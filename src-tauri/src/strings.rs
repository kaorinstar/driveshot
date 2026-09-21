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
