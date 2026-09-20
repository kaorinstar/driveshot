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
