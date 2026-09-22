//! Every word the application itself shows, named here rather than written where it is used.
//!
//! The settings window has its own table in `src/strings.ts`. This one covers what the window
//! cannot reach: the tray menu, the tray tooltip, and the sentences that explain a hotkey that
//! could not be registered.
//!
//! English alone today, as in the window. It is written this way from the start because adding a
//! second language then costs one more table and nothing else.

/// The tray menu entry that takes a shot. It does nothing yet; capture is #5.
pub const TRAY_CAPTURE: &str = "Capture";

/// The tray menu entry that opens the settings window.
pub const TRAY_SETTINGS: &str = "Settings";

/// The tray menu entry that exits.
pub const TRAY_QUIT: &str = "Quit Driveshot";

/// What the pointer shows when it rests on the tray icon.
pub const TRAY_TOOLTIP: &str = "Driveshot";

/// Said when a hotkey cannot be understood at all, rather than merely being taken.
pub fn hotkey_not_understood(shortcut: &str, reason: &str) -> String {
    format!("'{shortcut}' is not a key combination Driveshot understands: {reason}")
}

/// Said when a hotkey is understood but the operating system would not give it up.
///
/// This is the common case, and it is why registering a hotkey is reported rather than assumed:
/// another application already holding the combination leaves Driveshot with a hotkey that
/// silently does nothing, which is indistinguishable from a broken application.
pub fn hotkey_taken(shortcut: &str, reason: &str) -> String {
    format!("'{shortcut}' is already in use by something else, so Driveshot could not take it: {reason}")
}

/// Said when the windowing system lists no monitors at all, which should not happen on a machine
/// with a screen.
pub const CAPTURE_NO_MONITORS: &str = "Driveshot could not find a monitor to capture.";

/// Said when the monitor a selection was drawn on is no longer there: unplugged, or rearranged,
/// between the overlay opening and the drag finishing.
pub const CAPTURE_MONITOR_GONE: &str =
    "That monitor is no longer there, so there was nothing to capture.";

/// Said when the selection covered no pixels: a click rather than a drag, or a drag entirely off
/// the screen.
pub const CAPTURE_NOTHING_SELECTED: &str = "Nothing was selected, so no shot was taken.";

/// Said when the operating system will not say where the user's pictures go.
pub const CAPTURE_NO_PICTURES_FOLDER: &str =
    "Driveshot could not find your pictures folder, so it does not know where to put the shot.";

/// Said when the overlay that the selection is drawn on will not open.
pub fn capture_overlay_failed(reason: &str) -> String {
    format!("Driveshot could not cover the screen to take a shot: {reason}")
}

/// Said when the screen could not be read.
///
/// On macOS this is what a refused "Screen Recording" permission looks like, which is why the
/// reason is passed through rather than replaced with something tidier.
pub fn capture_failed(reason: &str) -> String {
    format!("The screen could not be captured: {reason}")
}

/// Said when the shot was taken but could not be written to disk.
pub fn capture_not_saved(path: &str, reason: &str) -> String {
    format!("The shot could not be saved to {path}: {reason}")
}

/// Said when the list of monitors could not be read.
pub fn capture_no_monitor_list(reason: &str) -> String {
    format!("Driveshot could not ask which monitors are attached: {reason}")
}

/// Said when the user asked to sign in while a sign-in is already waiting for a browser.
pub const SIGN_IN_ALREADY_RUNNING: &str =
    "Driveshot is already waiting for you to finish signing in. Check your browser.";

/// Said when nobody came back to the loopback port in time.
pub const SIGN_IN_TIMED_OUT: &str =
    "Signing in took too long, so Driveshot stopped waiting. Try again.";

/// Said when the sign-in succeeded but the permission Driveshot asked for was not granted.
///
/// Driveshot asks for one thing - access to the files it creates itself - and cannot upload
/// without it. There is nothing to fall back to, so this is refused rather than half-accepted.
pub const SIGN_IN_SCOPE_REFUSED: &str =
    "Driveshot was not given permission to add files to your Drive, so it cannot upload anything. \
     Sign in again and allow it.";

/// Said when no port on this machine could be opened to receive the sign-in.
pub fn sign_in_no_port(reason: &str) -> String {
    format!("Driveshot could not open a port on this computer to complete the sign-in: {reason}")
}

/// Said when the browser would not open.
pub fn sign_in_no_browser(reason: &str) -> String {
    format!("Driveshot could not open your browser to sign in: {reason}")
}

/// Said when the token endpoint could not be reached at all.
pub fn sign_in_no_network(reason: &str) -> String {
    format!("Driveshot could not reach Google to finish signing in: {reason}")
}

/// Said when Google answered the sign-in with a refusal.
///
/// The reason is passed through rather than tidied: 'access_denied' is the consent screen's
/// Cancel button and is not a fault, while 'invalid_client' means the client identifier no longer
/// names anything, and the two call for completely different things from the user.
pub fn sign_in_refused(reason: &str) -> String {
    format!("Google did not complete the sign-in: {reason}")
}

/// Said when the sign-in failed for a reason that is none of the above.
pub fn sign_in_failed(reason: &str) -> String {
    format!("Driveshot could not complete the sign-in: {reason}")
}

/// Said when the operating system would not produce random bytes.
pub fn sign_in_no_entropy(reason: &str) -> String {
    format!("Driveshot could not generate the random values a safe sign-in needs: {reason}")
}

/// Said when the file holding the user's own OAuth client cannot be read or understood.
pub fn client_file_unreadable(path: &std::path::Path, reason: &str) -> String {
    format!(
        "Driveshot could not read your own Google client from {}: {reason}",
        path.display()
    )
}

/// Said when that file is readable but names no client identifier.
pub fn client_file_has_no_id(path: &std::path::Path) -> String {
    format!(
        "{} does not contain a 'client_id', so Driveshot does not know which Google client to use.",
        path.display()
    )
}

/// Said when this build carries no client of its own and the user has not supplied one.
///
/// This is what a copy of Driveshot built from source without credentials looks like. It is a
/// normal state rather than a fault, so the sentence says what to do about it - and what to do is
/// in the window the sentence appears in, rather than in a file somebody has to go and find.
pub const NO_CLIENT_AT_ALL: &str =
    "This copy of Driveshot has no Google client built in. Create one in the Google Cloud console \
     as a 'Desktop app', then put it in the fields below.";

/// The page the browser is left on once the sign-in has been read.
pub const BROWSER_PAGE_DONE: &str = "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
    <title>Driveshot</title><body style=\"font-family:system-ui;margin:4rem;text-align:center\">\
    <h1>You are signed in</h1><p>You can close this tab and go back to Driveshot.</p></body></html>";

/// The page the browser is left on when the sign-in came back but could not be used.
pub const BROWSER_PAGE_FAILED: &str = "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
    <title>Driveshot</title><body style=\"font-family:system-ui;margin:4rem;text-align:center\">\
    <h1>That did not work</h1><p>Close this tab and go back to Driveshot, which says why.</p>\
    </body></html>";

/// The page anything other than the sign-in redirect is answered with.
pub const BROWSER_PAGE_IGNORED: &str = "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
    <title>Driveshot</title><body style=\"font-family:system-ui;margin:4rem;text-align:center\">\
    <p>Driveshot is waiting for a sign-in. There is nothing to see here.</p></body></html>";

/// Said when the client identifier field was left empty.
pub const CLIENT_NEEDS_AN_ID: &str =
    "A Google client needs a client ID. Copy it from the Google Cloud console.";

/// Said when the platform will not say where an application's configuration goes.
pub const CLIENT_NO_CONFIG_FOLDER: &str =
    "Driveshot could not find a folder to keep its settings in, so it cannot save a Google client.";

/// Said when the file holding the user's own OAuth client could not be written or removed.
pub fn client_file_unwritable(path: &std::path::Path, reason: &str) -> String {
    format!(
        "Driveshot could not save your Google client to {}: {reason}",
        path.display()
    )
}
