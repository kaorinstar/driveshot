// Hides the console window that Windows otherwise opens behind the application. Only in a release
// build: a debug build keeps the console, because that is where a panic is printed.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

//! The Driveshot application.
//!
//! It lives in the tray and waits for a key. Starting it puts nothing on the screen; the settings
//! window is opened from the tray menu, and closing that window hides it again rather than
//! exiting, because exiting is what the tray menu's last entry is for.
//!
//! What the hotkey does today is show that it fired. Capture itself is #5, the upload is #6, and
//! deletion is #7. Building it in this order means the tray, the hotkey and the window are known
//! to work before anything is written on top of them.
//!
//! The rule that shapes this file: a calculation belongs in `driveshot-core`, where it is tested
//! on every platform. What stays here is what genuinely needs a screen, a file or a network.

mod capture;
mod strings;

use chrono::{DateTime, SecondsFormat, Utc};
use driveshot_core::{Provider, Retention, Selection};
use serde::Serialize;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// The key combination Driveshot takes a shot with.
///
/// Fixed for now; making it editable is #9. Ctrl or Cmd with Shift and D, because the obvious
/// alternatives are taken: Windows itself holds Win+Shift+S for its own snipping tool, and macOS
/// holds Cmd+Shift+3, 4 and 5 for screenshots. This one is not free either - a browser in front
/// will not see Ctrl+Shift+D while Driveshot is running - which is precisely why a hotkey that
/// fails to register is reported rather than passed over in silence.
const DEFAULT_HOTKEY: &str = "CmdOrCtrl+Shift+D";

/// The label of the settings window, as `tauri.conf.json` names it.
const MAIN_WINDOW: &str = "main";

/// Whether the hotkey is Driveshot's, and when it last fired.
///
/// Held as state rather than worked out on demand because registering happens once, at startup,
/// and the answer has to survive until somebody opens the window to read it.
struct Hotkey {
    /// The combination that was asked for, whether or not it was granted.
    shortcut: String,
    /// Why it could not be registered, or `None` if it was.
    error: Option<String>,
    /// When it last fired. `None` until it has.
    last_fired: Mutex<Option<DateTime<Utc>>>,
}

/// Where the last shot was written, so the settings window can say.
///
/// A successful capture is deliberately quiet - no window, no sound - so this is how someone
/// checks that it worked. A failed one is not quiet: it opens the window and says why.
#[derive(Default)]
struct LastShot(Mutex<Option<String>>);

/// One cloud drive as the settings window lists it.
#[derive(Serialize)]
struct ProviderInfo {
    /// The stable identifier, which is what settings and records store.
    id: &'static str,
    /// The name shown to the user.
    name: &'static str,
    /// Whether uploading to this drive works yet.
    ///
    /// Every provider is listed from the start, including the ones not built yet, so the window
    /// shows the plan rather than hiding it. Set this to `true` in the same pull request that
    /// makes the provider work, not before.
    available: bool,
}

/// Every cloud drive Driveshot knows about, in the order the settings window offers them.
#[tauri::command]
fn providers() -> Vec<ProviderInfo> {
    Provider::ALL
        .into_iter()
        .map(|provider| ProviderInfo {
            id: provider.id(),
            name: provider.display_name(),
            available: false,
        })
        .collect()
}

/// When a file uploaded now would fall due for deletion.
#[derive(Serialize)]
struct ExpiryPreview {
    /// The moment the file falls due, as RFC 3339 in UTC, or `null` if it is never due.
    expires_at: Option<String>,
}

/// Answers "if I upload something now, when does it go away?" for a retention of `days` days.
///
/// `days` of `null` means the file is kept until the user deletes it themselves. Zero is refused
/// rather than treated as "delete at once", which is the rule [`Retention`] enforces.
#[tauri::command]
fn expiry_preview(days: Option<u32>) -> Result<ExpiryPreview, String> {
    let retention = match days {
        None => Retention::Forever,
        Some(days) => Retention::days(days)
            .ok_or_else(|| "A retention of zero days is not allowed.".to_owned())?,
    };

    Ok(ExpiryPreview {
        expires_at: retention
            .expires_at(Utc::now())
            .map(|due| due.to_rfc3339_opts(SecondsFormat::Secs, true)),
    })
}

/// Covers the screens so the user can draw a rectangle on one of them.
///
/// Called by the tray menu, the hotkey, and nothing else. A failure here is shown rather than
/// swallowed: the user pressed a key and is entitled to know that nothing happened.
fn start_capture(app: &AppHandle) {
    if let Err(error) = capture::begin(app) {
        eprintln!("{error}");
        report(app, &error);
    }
}

/// Takes the shot the user selected, or says why it could not.
///
/// The overlay calls this once, on the mouse button coming up. `monitor` is the index the overlay
/// was opened with, which is the monitor the selection is in.
///
/// It returns nothing, and returns before the shot has been taken. This runs on the main thread,
/// and the main thread is what has to close the overlays first; holding it here is what used to
/// put their dimming in the saved image (#34). So the outcome arrives at the closure below, on
/// another thread, and the overlay is told nothing - it is on its way out either way, and a
/// failure is shown in the settings window rather than on a window that is closing.
#[tauri::command]
fn finish_capture(app: AppHandle, monitor: usize, selection: Selection) {
    capture::finish(&app, monitor, selection, |app, outcome| match outcome {
        Ok(path) => {
            let path = path.display().to_string();
            println!("Shot saved to {path}");
            if let Some(last) = app.try_state::<LastShot>() {
                if let Ok(mut slot) = last.0.lock() {
                    *slot = Some(path);
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            report(app, &error);
        }
    });
}

/// Puts the overlays away without taking anything. Escape, or a click that was not a drag.
#[tauri::command]
fn cancel_capture(app: AppHandle) {
    capture::close_all(&app);
}

/// Told by an overlay that it has drawn itself and can be shown.
///
/// The overlays are created invisible so that the web view's white first frame never reaches the
/// screen (#20). `window` identifies which one is speaking.
#[tauri::command]
fn overlay_ready(app: AppHandle, window: tauri::Window) {
    capture::ready(&app, window.label());
}

/// Where the last shot went, or `null` if none has been taken since Driveshot started.
#[tauri::command]
fn last_shot(last: tauri::State<'_, LastShot>) -> Option<String> {
    last.0.lock().ok().and_then(|slot| slot.clone())
}

/// Puts a problem where the user will see it: in the settings window, which is brought up for it.
///
/// Printing alone is not enough. Driveshot has no window on the screen most of the time, so a
/// failure nobody is shown is a failure nobody knows about.
fn report(app: &AppHandle, problem: &str) {
    if let Some(last) = app.try_state::<LastShot>() {
        if let Ok(mut slot) = last.0.lock() {
            *slot = Some(problem.to_owned());
        }
    }
    show_settings(app);
}

/// The state of the hotkey, as the settings window shows it.
#[derive(Serialize)]
struct HotkeyStatus {
    /// The combination that was asked for.
    shortcut: String,
    /// Whether Driveshot holds it.
    registered: bool,
    /// Why it does not, in a sentence, or `null` if it does.
    error: Option<String>,
    /// When it last fired, as RFC 3339 in UTC, or `null` if it has not.
    last_fired: Option<String>,
}

/// Whether the hotkey is Driveshot's, and when it last fired.
#[tauri::command]
fn hotkey_status(hotkey: tauri::State<'_, Hotkey>) -> HotkeyStatus {
    let last_fired = hotkey
        .last_fired
        .lock()
        .ok()
        .and_then(|fired| *fired)
        .map(|at| at.to_rfc3339_opts(SecondsFormat::Secs, true));

    HotkeyStatus {
        shortcut: hotkey.shortcut.clone(),
        registered: hotkey.error.is_none(),
        error: hotkey.error.clone(),
        last_fired,
    }
}

/// Brings the settings window up, creating nothing: it exists from startup, hidden.
///
/// A window that will not show is not worth stopping the application over, so a failure here is
/// reported to whatever is watching the logs and otherwise passed over.
fn show_settings(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        eprintln!("the window labelled '{MAIN_WINDOW}' is missing from tauri.conf.json");
        return;
    };

    if let Err(error) = window.show() {
        eprintln!("the settings window would not show: {error}");
        return;
    }
    if let Err(error) = window.set_focus() {
        eprintln!("the settings window would not take focus: {error}");
    }
}

/// Notes that a capture was asked for, and asks for it.
fn capture_requested(app: &AppHandle) {
    if let Some(hotkey) = app.try_state::<Hotkey>() {
        if let Ok(mut last_fired) = hotkey.last_fired.lock() {
            *last_fired = Some(Utc::now());
        }
    }

    // For a settings window that happens to be open: it shows when the key last fired, and
    // nothing else would refresh it.
    if let Err(error) = app.emit("capture-requested", ()) {
        eprintln!("the window was not told about the capture request: {error}");
    }

    start_capture(app);
}

/// Registers the hotkey and reports what happened, which is the whole point: a combination
/// another application already holds leaves Driveshot with a key that does nothing.
fn register_hotkey(app: &AppHandle) -> Hotkey {
    let hotkey = |error: Option<String>| Hotkey {
        shortcut: DEFAULT_HOTKEY.to_owned(),
        error,
        last_fired: Mutex::new(None),
    };

    let shortcut = match Shortcut::from_str(DEFAULT_HOTKEY) {
        Ok(shortcut) => shortcut,
        Err(error) => {
            return hotkey(Some(strings::hotkey_not_understood(
                DEFAULT_HOTKEY,
                &error.to_string(),
            )))
        }
    };

    match app.global_shortcut().register(shortcut) {
        Ok(()) => hotkey(None),
        Err(error) => hotkey(Some(strings::hotkey_taken(
            DEFAULT_HOTKEY,
            &error.to_string(),
        ))),
    }
}

/// Builds the tray icon and its menu.
///
/// The icon is the one `tauri.conf.json` names, so the tray and the window cannot drift apart.
fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let capture = MenuItem::with_id(app, "capture", strings::TRAY_CAPTURE, true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", strings::TRAY_SETTINGS, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", strings::TRAY_QUIT, true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &capture,
            &PredefinedMenuItem::separator(app)?,
            &settings,
            &quit,
        ],
    )?;

    let mut tray = TrayIconBuilder::new()
        .tooltip(strings::TRAY_TOOLTIP)
        .menu(&menu)
        // The menu belongs to the right button. A left click opens the settings window, which is
        // what a single icon in the tray is expected to do.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "capture" => capture_requested(app),
            "settings" => show_settings(app),
            "quit" => app.exit(0),
            other => eprintln!("the tray menu sent an entry nothing handles: {other}"),
        })
        .on_tray_icon_event(|tray, event| {
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_settings(tray.app_handle());
            }
        });

    // Without an icon the tray entry is there but invisible, which reads as the application having
    // failed to start. Better to fail loudly at startup than to leave nothing on the screen.
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    } else {
        eprintln!("tauri.conf.json names no window icon, so the tray icon will be blank");
    }

    tray.build(app)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    // A key press is two events. Acting on the release alone means one capture per
                    // press rather than two.
                    if event.state() == ShortcutState::Released {
                        capture_requested(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            providers,
            expiry_preview,
            hotkey_status,
            finish_capture,
            cancel_capture,
            overlay_ready,
            last_shot
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // macOS shows an application with a window in the Dock and in the menu bar. Driveshot
            // is a tray application, so it asks not to be treated as one. Untested: nobody has
            // run the macOS build yet.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let hotkey = register_hotkey(&handle);
            // One line, at startup, saying whether the key is Driveshot's. The window shows the
            // same thing, but a log line is what someone has to hand when the application is
            // misbehaving on a machine that is not theirs.
            match &hotkey.error {
                None => println!("Driveshot holds {}.", hotkey.shortcut),
                Some(error) => eprintln!("{error}"),
            }
            app.manage(hotkey);
            app.manage(LastShot::default());

            build_tray(&handle)?;

            // Closing the settings window hides it. Driveshot keeps running, because the tray icon
            // is the application and the window is one way of looking at it. Quitting is the tray
            // menu's last entry.
            if let Some(window) = app.get_webview_window(MAIN_WINDOW) {
                let hidden = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Err(error) = hidden.hide() {
                            eprintln!("the settings window would not hide: {error}");
                        }
                    }
                });
            }

            // A hotkey nobody could take is the one failure a user has to be told about at once:
            // the application looks fine and its key does nothing. Everything else can wait until
            // the window is opened.
            if app.state::<Hotkey>().error.is_some() {
                show_settings(&handle);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Driveshot could not start");
}
