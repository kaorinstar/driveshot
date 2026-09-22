//! The parts of the OAuth 2.0 authorization code flow that need neither a browser nor a network.
//!
//! Signing in to a cloud drive is mostly I/O: a browser is opened, a loopback server waits for the
//! redirect, and an HTTP request exchanges the code for a token. None of that is here. What is
//! here is everything around it that is a calculation - building the authorization URL, deriving
//! a PKCE challenge from a verifier, and reading the redirect the browser came back with - because
//! those are the parts that are wrong in ways a compiler does not catch, and they can be tested
//! on any machine without an account anywhere.
//!
//! Two of these functions take random bytes rather than generating their own. Randomness comes
//! from the operating system, which this crate does not call; `src-tauri` supplies the bytes. That
//! also makes every test below an exact expected value rather than a shape check.
//!
//! # What is shaped around Google Drive
//!
//! The endpoints and the scope named here are Google's, because Google Drive is the drive
//! Driveshot implements first and alone (#2). Nothing else is pushing against the shape of this
//! module yet, and pretending otherwise would be inventing an abstraction against a provider
//! nobody here can test. Reshaping it when OneDrive and Dropbox arrive is part of #8.

use crate::{Error, Result};
use sha2::{Digest, Sha256};

/// Google's authorization endpoint: where the user's browser is sent to sign in and consent.
pub const GOOGLE_AUTHORIZATION_ENDPOINT: &str = "https://accounts.google.com/o/oauth2/v2/auth";

/// Google's token endpoint: where an authorization code is exchanged for tokens.
pub const GOOGLE_TOKEN_ENDPOINT: &str = "https://oauth2.googleapis.com/token";

/// Google's revocation endpoint: where a token is withdrawn when the user signs out.
pub const GOOGLE_REVOCATION_ENDPOINT: &str = "https://oauth2.googleapis.com/revoke";

/// The only scope Driveshot asks Google for.
///
/// `drive.file` reaches the files this application itself created, and nothing else in the user's
/// Drive. That is the whole of what Driveshot needs: it uploads a screenshot, shares it, and
/// deletes it again. It is also the reason signing in stays simple - Google classes this scope as
/// non-sensitive, so the consent screen is not gated behind a security assessment.
///
/// **Do not widen this.** `drive` or `drive.readonly` would hand Driveshot the user's whole Drive
/// to satisfy a feature that only ever touches its own uploads, and would turn the sign-in into a
/// review process at the same time.
pub const GOOGLE_DRIVE_FILE_SCOPE: &str = "https://www.googleapis.com/auth/drive.file";

/// The loopback address the redirect comes back to.
///
/// A literal address rather than `localhost`: `localhost` can resolve to either IPv4 or IPv6, and
/// the redirect URI registered with Google has to match what the browser actually asks for, to
/// the character.
pub const LOOPBACK_HOST: &str = "127.0.0.1";

/// The redirect URI for a loopback listener on `port`.
///
/// Google's native-application flow expects the port to be chosen when the listener opens rather
/// than registered in advance, so this is built at sign-in time from whatever port was free.
#[must_use]
pub fn loopback_redirect_uri(port: u16) -> String {
    format!("http://{LOOPBACK_HOST}:{port}")
}

/// A PKCE verifier and the challenge derived from it.
///
/// PKCE is what makes an authorization code useless to anyone who intercepts it. The verifier is
/// kept in this process; only its hash - the challenge - goes out in the authorization URL, and
/// the verifier is sent later, on the direct request that exchanges the code. A code stolen from
/// the redirect cannot be exchanged without it.
///
/// This matters more here than it does on a server. The redirect arrives on a loopback port that
/// any other program on the machine could have been listening on, and the client secret of a
/// desktop application is inside the installer, so it is not a secret at all. The verifier is the
/// part an attacker genuinely cannot have.
#[derive(Debug, Clone)]
pub struct Pkce {
    verifier: String,
    challenge: String,
}

impl Pkce {
    /// The value of `code_challenge_method`: the challenge is the SHA-256 of the verifier.
    ///
    /// The specification also allows `plain`, where the challenge is the verifier itself. That
    /// defends against nothing, and Driveshot does not offer it.
    pub const CHALLENGE_METHOD: &'static str = "S256";

    /// Derives a verifier and its challenge from 32 random bytes.
    ///
    /// The verifier is those bytes in base64url, which is 43 characters - the shortest length the
    /// specification allows, and 256 bits of entropy. The bytes must come from a cryptographically
    /// secure source; this function cannot check that, and a predictable verifier defends against
    /// nothing.
    #[must_use]
    pub fn from_entropy(entropy: &[u8; 32]) -> Self {
        let verifier = base64url(entropy);
        let challenge = base64url(Sha256::digest(verifier.as_bytes()).as_slice());
        Self {
            verifier,
            challenge,
        }
    }

    /// The verifier, which stays in this process until the code is exchanged.
    #[must_use]
    pub fn verifier(&self) -> &str {
        &self.verifier
    }

    /// The challenge, which goes out in the authorization URL.
    #[must_use]
    pub fn challenge(&self) -> &str {
        &self.challenge
    }
}

/// Derives the `state` parameter from 16 random bytes.
///
/// `state` is echoed back on the redirect and compared with what went out. It is what tells a
/// redirect this application asked for from one somebody else aimed at the loopback port. As with
/// the PKCE verifier, the bytes must be unpredictable.
#[must_use]
pub fn state_from_entropy(entropy: &[u8; 16]) -> String {
    base64url(entropy)
}

/// Everything the authorization URL is built from.
///
/// The fields are borrowed rather than owned because every one of them already exists somewhere
/// else: the client identifier is a constant, the verifier belongs to a [`Pkce`], and the redirect
/// URI belongs to the listener that is already open.
#[derive(Debug, Clone, Copy)]
pub struct AuthorizationRequest<'a> {
    /// The OAuth client identifier registered with the provider.
    pub client_id: &'a str,
    /// Where the provider sends the browser back to, from [`loopback_redirect_uri`].
    pub redirect_uri: &'a str,
    /// The scope being asked for, normally [`GOOGLE_DRIVE_FILE_SCOPE`].
    pub scope: &'a str,
    /// The value from [`state_from_entropy`], to be checked when the redirect arrives.
    pub state: &'a str,
    /// The challenge from [`Pkce::challenge`].
    pub challenge: &'a str,
}

impl AuthorizationRequest<'_> {
    /// Builds the URL the user's browser is sent to.
    ///
    /// Two parameters here are not obviously needed and both are. `access_type=offline` is what
    /// asks for a refresh token; without it the sign-in lasts an hour and the user is sent back to
    /// a browser every time the token expires. `prompt=consent` forces the consent screen even
    /// when the user has approved this application before, which is the documented way to be given
    /// a refresh token rather than only an access token - a second sign-in that returns no refresh
    /// token leaves Driveshot unable to delete anything once the hour is up.
    #[must_use]
    pub fn url(&self) -> String {
        let mut url = String::from(GOOGLE_AUTHORIZATION_ENDPOINT);
        url.push('?');
        let parameters = [
            ("client_id", self.client_id),
            ("redirect_uri", self.redirect_uri),
            ("response_type", "code"),
            ("scope", self.scope),
            ("state", self.state),
            ("code_challenge", self.challenge),
            ("code_challenge_method", Pkce::CHALLENGE_METHOD),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ];
        for (index, (name, value)) in parameters.iter().enumerate() {
            if index > 0 {
                url.push('&');
            }
            url.push_str(name);
            url.push('=');
            percent_encode_into(value, &mut url);
        }
        url
    }
}

/// Reads the authorization code out of the query string the redirect arrived with.
///
/// `query` is the part after the `?` of the request line the loopback listener received, with no
/// leading `?`. `expected_state` is what [`state_from_entropy`] produced for this sign-in.
///
/// The state is compared before the code is looked at, and a mismatch is refused rather than
/// reported as a user error: a redirect carrying the wrong state was not asked for by this
/// sign-in, so whatever code it carries is not this application's to exchange.
///
/// # Errors
///
/// - [`Error::AuthorizationDenied`] if the provider reported an error, which includes the ordinary
///   case of the user pressing Cancel on the consent screen.
/// - [`Error::AuthorizationStateMismatch`] if `state` is absent or is not `expected_state`.
/// - [`Error::AuthorizationIncomplete`] if the redirect carries neither a code nor an error.
pub fn authorization_code(query: &str, expected_state: &str) -> Result<String> {
    let mut code = None;
    let mut state = None;
    let mut error = None;

    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (name, value) = match pair.split_once('=') {
            Some(split) => split,
            None => (pair, ""),
        };
        match name {
            "code" => code = Some(percent_decode(value)),
            "state" => state = Some(percent_decode(value)),
            "error" => error = Some(percent_decode(value)),
            _ => {}
        }
    }

    match state {
        Some(state) if constant_time_eq(&state, expected_state) => {}
        _ => return Err(Error::AuthorizationStateMismatch),
    }

    if let Some(error) = error {
        return Err(Error::AuthorizationDenied(error));
    }

    code.filter(|code| !code.is_empty())
        .ok_or(Error::AuthorizationIncomplete)
}

/// Encodes bytes as base64url with no padding, which is what PKCE and `state` both call for.
fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut buffer = [0_u8; 3];
        buffer[..chunk.len()].copy_from_slice(chunk);
        let packed = u32::from(buffer[0]) << 16 | u32::from(buffer[1]) << 8 | u32::from(buffer[2]);
        // Three bytes make four characters; two make three, and one makes two. The rest of the
        // group is padding, which base64url leaves off rather than writing as '='.
        for index in 0..chunk.len() + 1 {
            let six = (packed >> (18 - index * 6)) & 0b11_1111;
            encoded.push(char::from(ALPHABET[six as usize]));
        }
    }
    encoded
}

/// Appends `value` to `into`, percent-encoding everything RFC 3986 does not call unreserved.
fn percent_encode_into(value: &str, into: &mut String) {
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                into.push(char::from(byte));
            }
            _ => {
                into.push('%');
                into.push(hex_digit(byte >> 4));
                into.push(hex_digit(byte & 0x0f));
            }
        }
    }
}

fn hex_digit(nibble: u8) -> char {
    char::from(if nibble < 10 {
        b'0' + nibble
    } else {
        b'A' + nibble - 10
    })
}

/// Reverses [`percent_encode_into`] for one query-string value.
///
/// A `%` that is not followed by two hexadecimal digits is left as it is rather than refused. This
/// runs on whatever arrived at a loopback port, and a value this application does not recognise is
/// discarded by the caller anyway - the state check is what decides whether the redirect is ours.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                decoded.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                match (hex_value(bytes[index + 1]), hex_value(bytes[index + 2])) {
                    (Some(high), Some(low)) => {
                        decoded.push(high << 4 | low);
                        index += 3;
                    }
                    _ => {
                        decoded.push(b'%');
                        index += 1;
                    }
                }
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Compares two strings without returning early on the first difference.
///
/// The lengths still differ observably, which is unavoidable and does not matter: both values here
/// are a fixed length this application chose.
fn constant_time_eq(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one worked example in RFC 7636, which fixes both the encoding and the hash.
    #[test]
    fn pkce_matches_the_specification_example() {
        let entropy: [u8; 32] = [
            116, 24, 223, 180, 151, 153, 224, 37, 79, 250, 96, 125, 216, 173, 187, 186, 22, 212,
            37, 77, 105, 214, 191, 240, 91, 88, 5, 88, 83, 132, 141, 121,
        ];
        let pkce = Pkce::from_entropy(&entropy);
        assert_eq!(
            pkce.verifier(),
            "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"
        );
        assert_eq!(
            pkce.challenge(),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn a_verifier_is_the_shortest_length_the_specification_allows() {
        let pkce = Pkce::from_entropy(&[0; 32]);
        assert_eq!(pkce.verifier().len(), 43);
        assert!(pkce
            .verifier()
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'));
    }

    #[test]
    fn base64url_leaves_the_padding_off() {
        assert_eq!(base64url(b""), "");
        assert_eq!(base64url(b"f"), "Zg");
        assert_eq!(base64url(b"fo"), "Zm8");
        assert_eq!(base64url(b"foo"), "Zm9v");
        assert_eq!(base64url(b"foob"), "Zm9vYg");
        assert_eq!(base64url(b"fooba"), "Zm9vYmE");
        assert_eq!(base64url(b"foobar"), "Zm9vYmFy");
        // The two characters base64url differs from base64 in.
        assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
    }

    #[test]
    fn a_state_is_twenty_two_characters() {
        assert_eq!(state_from_entropy(&[0; 16]).len(), 22);
    }

    #[test]
    fn a_loopback_uri_names_an_address_rather_than_a_name() {
        assert_eq!(loopback_redirect_uri(49_152), "http://127.0.0.1:49152");
    }

    fn request<'a>(state: &'a str, challenge: &'a str) -> AuthorizationRequest<'a> {
        AuthorizationRequest {
            client_id: "1234.apps.googleusercontent.com",
            redirect_uri: "http://127.0.0.1:49152",
            scope: GOOGLE_DRIVE_FILE_SCOPE,
            state,
            challenge,
        }
    }

    #[test]
    fn the_authorization_url_encodes_every_value() {
        let url = request("st-ate", "chal-lenge").url();
        assert_eq!(
            url,
            "https://accounts.google.com/o/oauth2/v2/auth\
             ?client_id=1234.apps.googleusercontent.com\
             &redirect_uri=http%3A%2F%2F127.0.0.1%3A49152\
             &response_type=code\
             &scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fdrive.file\
             &state=st-ate\
             &code_challenge=chal-lenge\
             &code_challenge_method=S256\
             &access_type=offline\
             &prompt=consent"
        );
    }

    #[test]
    fn the_authorization_url_asks_for_a_refresh_token() {
        // Losing either of these leaves Driveshot signed in for an hour and unable to delete
        // anything afterwards, which is the failure this project cares about most.
        let url = request("state", "challenge").url();
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn a_granted_redirect_gives_up_its_code() {
        let code = authorization_code("state=abc&code=4%2F0AX4&scope=email", "abc").unwrap();
        assert_eq!(code, "4/0AX4");
    }

    #[test]
    fn the_order_of_the_parameters_does_not_matter() {
        let code = authorization_code("code=4%2F0AX4&state=abc", "abc").unwrap();
        assert_eq!(code, "4/0AX4");
    }

    #[test]
    fn a_cancelled_consent_screen_reports_what_it_said() {
        let error = authorization_code("error=access_denied&state=abc", "abc").unwrap_err();
        assert!(
            matches!(&error, Error::AuthorizationDenied(reported) if reported == "access_denied")
        );
    }

    #[test]
    fn a_redirect_with_the_wrong_state_is_refused_before_its_code_is_read() {
        let error = authorization_code("code=4%2F0AX4&state=somebody-else", "abc").unwrap_err();
        assert!(matches!(error, Error::AuthorizationStateMismatch));
    }

    #[test]
    fn a_redirect_with_no_state_is_refused() {
        let error = authorization_code("code=4%2F0AX4", "abc").unwrap_err();
        assert!(matches!(error, Error::AuthorizationStateMismatch));
    }

    #[test]
    fn an_error_with_the_wrong_state_is_a_mismatch_rather_than_a_denial() {
        // Otherwise anyone able to reach the loopback port could make Driveshot report whatever
        // they liked to the user.
        let error = authorization_code("error=access_denied&state=nope", "abc").unwrap_err();
        assert!(matches!(error, Error::AuthorizationStateMismatch));
    }

    #[test]
    fn a_redirect_with_neither_a_code_nor_an_error_is_incomplete() {
        let error = authorization_code("state=abc", "abc").unwrap_err();
        assert!(matches!(error, Error::AuthorizationIncomplete));
        let error = authorization_code("state=abc&code=", "abc").unwrap_err();
        assert!(matches!(error, Error::AuthorizationIncomplete));
    }

    #[test]
    fn percent_decoding_survives_what_a_loopback_port_can_be_sent() {
        assert_eq!(percent_decode("a+b"), "a b");
        assert_eq!(percent_decode("%2F"), "/");
        assert_eq!(percent_decode("%2f"), "/");
        // Truncated or malformed escapes are left alone rather than refused.
        assert_eq!(percent_decode("%2"), "%2");
        assert_eq!(percent_decode("%"), "%");
        assert_eq!(percent_decode("%zz"), "%zz");
    }

    #[test]
    fn constant_time_comparison_still_compares() {
        assert!(constant_time_eq("abc", "abc"));
        assert!(!constant_time_eq("abc", "abd"));
        assert!(!constant_time_eq("abc", "ab"));
        assert!(constant_time_eq("", ""));
    }
}
