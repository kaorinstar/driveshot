//! Signing in to Google Drive: the half of the flow that needs a browser and a network.
//!
//! The other half is `driveshot_core::oauth`, which builds the authorization URL, derives the
//! PKCE challenge and reads the redirect back. Nothing in this file decides what those strings
//! say; it opens a browser, listens on a port, and makes one HTTPS request.
//!
//! # Why a port on this machine rather than a page on a web site
//!
//! A desktop application has nowhere to receive a redirect. RFC 8252 answers that by having the
//! application listen on the loopback address for the length of the sign-in and register the
//! redirect as `http://127.0.0.1` with whatever port turned out to be free. The port is asked for
//! at sign-in time (`127.0.0.1:0` lets the operating system choose) rather than fixed, because a
//! fixed one is a port that is sometimes already taken.
//!
//! That port is reachable by anything else running as this user, which is the reason for two
//! checks that would otherwise look like paranoia. The `state` is compared before the redirect's
//! contents are read at all, so a request somebody else aimed at the port cannot have its code
//! exchanged. And the PKCE verifier never leaves this process until the exchange, so a code read
//! off the port by something watching cannot be exchanged by it either.
//!
//! # What is not here yet
//!
//! Tokens are held in memory and go when Driveshot does, so signing in has to be done once per
//! start. Keeping them across restarts means choosing where they live and what protects them,
//! which is written down in `SECURITY.md` before the change that does it.

use crate::google_client::{self, Source};
use crate::strings;
use chrono::{DateTime, SecondsFormat, Utc};
use driveshot_core::oauth::{
    self, AuthorizationRequest, Pkce, TokenExchange, TokenResponse, GOOGLE_DRIVE_FILE_SCOPE,
    GOOGLE_TOKEN_ENDPOINT,
};
use serde::Serialize;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;

/// How long the user has to finish signing in before the port is given up.
///
/// Long enough to find a password and a second factor; short enough that a sign-in somebody
/// walked away from does not leave a port open for the rest of the day.
const SIGN_IN_DEADLINE: Duration = Duration::from_secs(180);

/// How often the listener is asked whether the browser has come back.
const POLL_INTERVAL: Duration = Duration::from_millis(100);

/// The most of a request line that is read before it is refused.
///
/// A redirect's query is a few hundred characters. Anything sending more than this to the port is
/// not the browser coming back, and is not worth reading into memory.
const MAX_REQUEST_LINE: usize = 8 * 1024;

/// What Driveshot holds after a successful sign-in.
///
/// In memory only, for now. The access token expires within the hour and the refresh token is
/// what buys the next one.
#[derive(Debug, Clone)]
pub struct Tokens {
    /// The token that authorises a call to the Drive API.
    ///
    /// Nothing reads it yet, which is why it is allowed to be dead: this change signs in, and the
    /// change that uploads is what first sends it. Holding it now rather than later is the point
    /// of signing in at all, so it is kept rather than dropped and re-fetched.
    #[allow(dead_code, reason = "read by the upload, which is the rest of #6")]
    pub access_token: String,
    /// When that token stops being usable, already a minute short of the real expiry.
    pub expires_at: DateTime<Utc>,
    /// The token that buys a new access token.
    ///
    /// `None` when Google answered without one. It is never overwritten with `None`: losing it
    /// means being unable to delete what has already been published, once the access token goes.
    pub refresh_token: Option<String>,
}

/// Whether Driveshot is signed in, and to which client.
#[derive(Default)]
pub struct Session {
    tokens: Mutex<Option<Tokens>>,
    /// Set while a sign-in is running, so a second press of the button does not open a second
    /// browser window and a second listener.
    running: AtomicBool,
}

impl Session {
    /// The tokens, if there are any.
    pub fn tokens(&self) -> Option<Tokens> {
        self.tokens.lock().ok().and_then(|held| held.clone())
    }

    /// Keeps what a sign-in or a refresh returned.
    ///
    /// A response without a refresh token keeps the one already held rather than clearing it.
    fn keep(&self, response: &TokenResponse, answered_at: DateTime<Utc>) {
        let Ok(mut held) = self.tokens.lock() else {
            return;
        };
        let refresh_token = response.refresh_token.clone().or_else(|| {
            held.as_ref()
                .and_then(|tokens| tokens.refresh_token.clone())
        });
        *held = Some(Tokens {
            access_token: response.access_token.clone(),
            expires_at: response.expires_at(answered_at),
            refresh_token,
        });
    }

    /// Forgets everything. Signing out.
    fn clear(&self) {
        if let Ok(mut held) = self.tokens.lock() {
            *held = None;
        }
    }
}

/// What the settings window shows about the sign-in.
#[derive(Serialize)]
pub struct Status {
    /// Whether Driveshot holds a usable access token.
    pub signed_in: bool,
    /// When that token expires, as RFC 3339 in UTC, or `null` when not signed in.
    pub expires_at: Option<String>,
    /// Whether a refresh token was issued, which is what makes the sign-in outlast the hour.
    pub can_refresh: bool,
    /// Whether the client in use is the built-in one or the user's own, or `null` when there is
    /// no usable client at all.
    pub client_source: Option<Source>,
    /// Why there is no usable client, or `null` when there is one.
    pub client_problem: Option<String>,
    /// Where a client of one's own is kept, so the window can say it without knowing the path.
    pub client_file: Option<String>,
    /// The identifier of the user's own client, so the window can put it back in its field.
    ///
    /// `null` when there is none. The secret is never returned: the window has no use for it, and
    /// an empty secret field means "keep what is stored" rather than "clear it".
    pub saved_client_id: Option<String>,
}

/// Whether Driveshot is signed in, and what client it would use.
#[tauri::command]
pub fn sign_in_status(app: AppHandle, session: tauri::State<'_, Session>) -> Status {
    let tokens = session.tokens();
    let (client_source, client_problem) = match google_client::current(&app) {
        Ok((_, source)) => (Some(source), None),
        Err(problem) => (None, Some(problem)),
    };

    Status {
        signed_in: tokens
            .as_ref()
            .is_some_and(|tokens| tokens.expires_at > Utc::now()),
        expires_at: tokens
            .as_ref()
            .map(|tokens| tokens.expires_at.to_rfc3339_opts(SecondsFormat::Secs, true)),
        can_refresh: tokens
            .as_ref()
            .is_some_and(|tokens| tokens.refresh_token.is_some()),
        client_source,
        client_problem,
        client_file: google_client::client_file(&app).map(|path| path.display().to_string()),
        saved_client_id: google_client::saved_client_id(&app),
    }
}

/// Saves a Google client of the user's own, and answers with the state that leaves.
///
/// Signing in is not undone by this. A client that has changed makes the tokens held against the
/// old one worthless, so they are dropped: leaving them would leave the window claiming a
/// connection that the next request would find broken.
///
/// # Errors
///
/// Returns a sentence for the user: an empty identifier, or a file that could not be written.
#[tauri::command]
pub fn save_google_client(
    app: AppHandle,
    client_id: String,
    client_secret: String,
) -> Result<Status, String> {
    google_client::save(&app, &client_id, &client_secret)?;
    app.state::<Session>().clear();
    Ok(sign_in_status(app.clone(), app.state::<Session>()))
}

/// Removes the user's own client, so the built-in one is used again.
///
/// As with saving, whatever was signed in belonged to the client being dropped, so it goes too.
///
/// # Errors
///
/// Returns a sentence for the user when the file is there and will not go.
#[tauri::command]
pub fn forget_google_client(app: AppHandle) -> Result<Status, String> {
    google_client::forget(&app)?;
    app.state::<Session>().clear();
    Ok(sign_in_status(app.clone(), app.state::<Session>()))
}

/// Forgets the tokens held for this run.
///
/// This is not a revocation: the grant at Google stays until the user withdraws it there. Saying
/// so is `SECURITY.md`'s job, and withdrawing it properly is worth its own change.
#[tauri::command]
pub fn sign_out(app: AppHandle, session: tauri::State<'_, Session>) -> Status {
    session.clear();
    sign_in_status(app.clone(), app.state::<Session>())
}

/// Opens the browser, waits for the redirect, and exchanges the code for tokens.
///
/// # Errors
///
/// Returns a sentence for the user. Every failure here is one they can act on: no client
/// configured, a consent screen they cancelled, a browser that would not open, a wait that ran
/// out, or a scope they declined.
#[tauri::command]
pub async fn sign_in(app: AppHandle) -> Result<Status, String> {
    let session = app.state::<Session>();
    if session.running.swap(true, Ordering::SeqCst) {
        return Err(strings::SIGN_IN_ALREADY_RUNNING.to_owned());
    }
    let outcome = run_sign_in(&app).await;
    app.state::<Session>()
        .running
        .store(false, Ordering::SeqCst);
    outcome?;
    Ok(sign_in_status(app.clone(), app.state::<Session>()))
}

/// The sign-in itself, with the "only one at a time" flag handled by its caller.
async fn run_sign_in(app: &AppHandle) -> Result<(), String> {
    let (client, _) = google_client::current(app)?;

    // The port is taken before the browser is opened. A listener that will not bind has to be
    // reported before the user is sent somewhere to type a password.
    let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
        .map_err(|error| strings::sign_in_no_port(&error.to_string()))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| strings::sign_in_no_port(&error.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|error| strings::sign_in_no_port(&error.to_string()))?
        .port();
    let redirect_uri = oauth::loopback_redirect_uri(port);

    let pkce = Pkce::from_entropy(&entropy::<32>()?);
    let state = oauth::state_from_entropy(&entropy::<16>()?);

    let url = AuthorizationRequest {
        client_id: &client.client_id,
        redirect_uri: &redirect_uri,
        scope: GOOGLE_DRIVE_FILE_SCOPE,
        state: &state,
        challenge: pkce.challenge(),
    }
    .url();

    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| strings::sign_in_no_browser(&error.to_string()))?;

    // Waiting on a socket is blocking work, so it does not belong on a runtime thread.
    let awaited_state = state.clone();
    let code =
        tauri::async_runtime::spawn_blocking(move || wait_for_redirect(&listener, &awaited_state))
            .await
            .map_err(|error| strings::sign_in_failed(&error.to_string()))??;

    let body = TokenExchange {
        code: &code,
        client_id: &client.client_id,
        client_secret: client.secret(),
        redirect_uri: &redirect_uri,
        verifier: pkce.verifier(),
    }
    .form_body();

    let answered_at = Utc::now();
    let answer = post_form(GOOGLE_TOKEN_ENDPOINT, body).await?;
    let response = oauth::read_token_response(&answer)
        .map_err(|error| strings::sign_in_refused(&error.to_string()))?;

    // A user can grant less than was asked for. Finding that out here, rather than at the first
    // upload, is the difference between a sentence about the consent screen and a failure nobody
    // can place.
    if !response.granted(GOOGLE_DRIVE_FILE_SCOPE) {
        return Err(strings::SIGN_IN_SCOPE_REFUSED.to_owned());
    }

    app.state::<Session>().keep(&response, answered_at);
    Ok(())
}

/// Sends a form-encoded body and returns whatever came back, error status or not.
///
/// The status is deliberately not checked. The token endpoint answers a refusal with a 400 and a
/// JSON body naming what was wrong, and that body is worth more to the user than "400".
async fn post_form(endpoint: &str, body: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| strings::sign_in_failed(&error.to_string()))?;

    let response = client
        .post(endpoint)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|error| strings::sign_in_no_network(&error.to_string()))?;

    response
        .text()
        .await
        .map_err(|error| strings::sign_in_no_network(&error.to_string()))
}

/// Waits for the browser to come back to the loopback port, and returns the authorization code.
///
/// Anything arriving that is not the redirect - a favicon request, a browser probing the port,
/// somebody else entirely - is answered and ignored rather than ending the wait. Only a request
/// whose query carries `code` or `error` is treated as the answer, and its `state` decides whether
/// it is this sign-in's answer at all.
fn wait_for_redirect(listener: &TcpListener, state: &str) -> Result<String, String> {
    let deadline = Instant::now() + SIGN_IN_DEADLINE;

    while Instant::now() < deadline {
        let stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(POLL_INTERVAL);
                continue;
            }
            Err(error) => return Err(strings::sign_in_failed(&error.to_string())),
        };

        let Some(query) = read_query(&stream) else {
            answer(&stream, strings::BROWSER_PAGE_IGNORED);
            continue;
        };
        if !query.contains("code=") && !query.contains("error=") {
            answer(&stream, strings::BROWSER_PAGE_IGNORED);
            continue;
        }

        return match oauth::authorization_code(&query, state) {
            Ok(code) => {
                answer(&stream, strings::BROWSER_PAGE_DONE);
                Ok(code)
            }
            Err(error) => {
                answer(&stream, strings::BROWSER_PAGE_FAILED);
                Err(strings::sign_in_refused(&error.to_string()))
            }
        };
    }

    Err(strings::SIGN_IN_TIMED_OUT.to_owned())
}

/// Reads the query string out of an HTTP request line, or `None` if there is not one.
fn read_query(mut stream: &TcpStream) -> Option<String> {
    if stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .is_err()
    {
        return None;
    }

    let mut buffer = Vec::new();
    let mut byte = [0_u8; 1];
    while buffer.len() < MAX_REQUEST_LINE {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) if byte[0] == b'\n' => break,
            Ok(_) => buffer.push(byte[0]),
            Err(_) => return None,
        }
    }

    // "GET /?code=...&state=... HTTP/1.1"
    let line = String::from_utf8_lossy(&buffer);
    let target = line.split(' ').nth(1)?;
    target.split_once('?').map(|(_, query)| query.to_owned())
}

/// Answers the browser with a page, so the user is not left looking at a failed request.
///
/// A failure to write is not reported anywhere: the sign-in itself has already succeeded or
/// failed by this point, and the user's answer is in Driveshot rather than in the browser.
fn answer(mut stream: &TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

/// Random bytes from the operating system, for the PKCE verifier and the `state`.
///
/// A failure here stops the sign-in rather than falling back to something predictable. Both
/// values defend against nothing if they can be guessed.
fn entropy<const N: usize>() -> Result<[u8; N], String> {
    let mut bytes = [0_u8; N];
    getrandom::fill(&mut bytes).map_err(|error| strings::sign_in_no_entropy(&error.to_string()))?;
    Ok(bytes)
}
