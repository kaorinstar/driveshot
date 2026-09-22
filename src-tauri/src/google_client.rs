//! Which OAuth client Driveshot signs in with, and where that answer comes from.
//!
//! There are two places, and the order matters. A file the user wrote wins over the one built
//! into the application, because that file is the escape hatch: it is what lets somebody keep
//! using Driveshot when the client it ships with has stopped working. Design rule 6 in
//! `CLAUDE.md`, and the three routes it gives are written out in `docs/architecture.md`.
//!
//! **Nothing here is a secret.** The identifier and the secret of a desktop client both sit
//! inside the installer, where anyone who wants them can read them. Google's own documentation
//! says so, and RFC 8252 says so of every native application. What makes the sign-in safe is
//! PKCE, in `driveshot_core::oauth`, not the secrecy of these two strings. They are kept out of
//! this repository anyway - there is no reason to publish the project's own quota to a search
//! engine - which is why the built-in one arrives through the environment at build time rather
//! than as a literal below.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// The file a user writes their own client into, in Driveshot's configuration folder.
///
/// A file rather than a setting in the window, because settings that persist are #9 and this
/// cannot wait for them: an escape hatch that only exists after the next feature is not an escape
/// hatch. #9 absorbs this, and reads the same file.
const CLIENT_FILE: &str = "google-client.json";

/// The client identifier baked in at build time, if the build was given one.
///
/// `option_env!` rather than `env!` so that a checkout with nothing configured still compiles.
/// A build without it produces an application that can do everything except sign in with the
/// built-in client, and says so rather than failing at the first request.
const BUILT_IN_ID: Option<&str> = option_env!("DRIVESHOT_GOOGLE_CLIENT_ID");

/// The client secret baked in at build time, if the build was given one.
const BUILT_IN_SECRET: Option<&str> = option_env!("DRIVESHOT_GOOGLE_CLIENT_SECRET");

/// An OAuth client: who Driveshot says it is when it asks a user for access.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    /// The client identifier registered with Google.
    pub client_id: String,
    /// The client secret, when the client was issued one.
    ///
    /// Optional because Google accepts the token exchange without it for a desktop client, and
    /// because a user creating their own may reasonably not want to copy it about.
    #[serde(default)]
    pub client_secret: Option<String>,
}

impl OAuthClient {
    /// The secret as the token request wants it: absent rather than empty.
    pub fn secret(&self) -> Option<&str> {
        self.client_secret
            .as_deref()
            .filter(|secret| !secret.is_empty())
    }
}

/// Where the client in use came from, which is what the settings window tells the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// The one built into this copy of Driveshot.
    BuiltIn,
    /// One the user wrote into [`CLIENT_FILE`].
    UserSupplied,
}

/// The client Driveshot will sign in with, and where it came from.
///
/// # Errors
///
/// Returns a sentence for the user when there is no usable client: either the file is there and
/// cannot be read or understood, or there is no file and this build carries no built-in client.
pub fn current(app: &AppHandle) -> Result<(OAuthClient, Source), String> {
    if let Some(path) = client_file(app) {
        if path.exists() {
            let text = std::fs::read_to_string(&path).map_err(|error| {
                crate::strings::client_file_unreadable(&path, &error.to_string())
            })?;
            let client: OAuthClient = serde_json::from_str(&text).map_err(|error| {
                crate::strings::client_file_unreadable(&path, &error.to_string())
            })?;
            if client.client_id.trim().is_empty() {
                return Err(crate::strings::client_file_has_no_id(&path));
            }
            return Ok((client, Source::UserSupplied));
        }
    }

    match BUILT_IN_ID.map(str::trim).filter(|id| !id.is_empty()) {
        Some(client_id) => Ok((
            OAuthClient {
                client_id: client_id.to_owned(),
                client_secret: BUILT_IN_SECRET.map(str::to_owned),
            },
            Source::BuiltIn,
        )),
        None => Err(crate::strings::no_client_at_all(
            &client_file(app)
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| CLIENT_FILE.to_owned()),
        )),
    }
}

/// Where [`CLIENT_FILE`] is, or `None` if the platform will not say where configuration goes.
pub fn client_file(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|dir| dir.join(CLIENT_FILE))
}
