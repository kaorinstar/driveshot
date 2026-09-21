//! Taking a shot of part of the screen.
//!
//! The sequence is: cover every monitor with an overlay, let the user drag a rectangle on one of
//! them, capture that monitor, cut the rectangle out, and write it to a file.
//!
//! # The coordinate systems, and why they are kept apart
//!
//! Three of them meet here, and mixing them up is how a screenshot ends up cropped to the wrong
//! place on a display that is not at 100%:
//!
//! - **Physical pixels**, which is where the monitors are and what a captured image is made of.
//!   Tauri reports monitor positions and sizes in these, but a window *builder* takes points - so
//!   an overlay is placed by `set_position` after it is built rather than by the builder (#22).
//! - **Points**, the overlay's own coordinates, which is what the pointer events the user
//!   generates are in. Physical divided by the scale factor.
//! - **Pixels within one captured image**, which is what a crop needs.
//!
//! Two rules keep it straight. Monitors are matched to what `xcap` captures by a physical point
//! inside them - `Monitor::from_point` is physical on every platform, while `Monitor::width` is
//! logical on Linux and physical on Windows, so the point is the only part that can be relied on.
//! And the conversion from the user's rectangle to pixels is done by `driveshot_core::pixels_for`,
//! which measures the scale from the captured image rather than asking any platform for it.

use std::path::PathBuf;

use driveshot_core::{LogicalSize, PixelSize, Selection};
use tauri::utils::config::Color;
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::strings;

/// The page every overlay window shows.
const OVERLAY_PAGE: &str = "overlay.html";

/// What an overlay window's label starts with, so that they can all be found and closed again.
const OVERLAY_PREFIX: &str = "overlay-";

/// Where a monitor is and how large, in the one coordinate system that means the same thing
/// everywhere.
#[derive(Debug, Clone, Copy)]
struct MonitorGeometry {
    /// Left edge, in physical pixels, in the desktop's coordinates.
    x: i32,
    /// Top edge, in physical pixels, in the desktop's coordinates.
    y: i32,
    /// Width in physical pixels.
    width: u32,
    /// Height in physical pixels.
    height: u32,
    /// Physical pixels per point.
    scale: f64,
}

impl MonitorGeometry {
    /// A point inside this monitor, in physical pixels, for matching it against what `xcap` sees.
    ///
    /// The centre rather than a corner: a corner is shared with the monitor next to it, and which
    /// of the two owns it is exactly the sort of thing platforms disagree about.
    fn centre(self) -> (i32, i32) {
        (
            self.x.saturating_add_unsigned(self.width / 2),
            self.y.saturating_add_unsigned(self.height / 2),
        )
    }

    /// The size of an overlay covering this monitor, in the points its pointer events use.
    fn overlay_size(self) -> LogicalSize {
        LogicalSize {
            width: f64::from(self.width) / self.scale,
            height: f64::from(self.height) / self.scale,
        }
    }
}

/// Covers every monitor with an overlay window and returns how many were opened.
///
/// Already showing an overlay means the user pressed the key twice; the second press is ignored
/// rather than stacking a second set of windows over the first.
pub fn begin(app: &AppHandle) -> Result<usize, String> {
    if !overlay_labels(app).is_empty() {
        return Ok(0);
    }

    let monitors = monitors(app)?;
    if monitors.is_empty() {
        return Err(strings::CAPTURE_NO_MONITORS.to_owned());
    }

    let mut opened = 0;
    for (index, monitor) in monitors.iter().enumerate() {
        match open_overlay(app, index, *monitor) {
            Ok(()) => opened += 1,
            Err(error) => {
                // One monitor failing must not leave the others covered: the user would be looking
                // at a screen they cannot dismiss.
                close_all(app);
                return Err(strings::capture_overlay_failed(&error.to_string()));
            }
        }
    }

    Ok(opened)
}

/// Closes every overlay. Safe to call when there are none.
pub fn close_all(app: &AppHandle) {
    for label in overlay_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            if let Err(error) = window.close() {
                eprintln!("an overlay window would not close: {error}");
            }
        }
    }
}

/// Captures what the user selected on `monitor_index` and writes it to a file.
///
/// Returns where it was written. The overlays are closed first, so that they are not in the shot.
pub fn finish(
    app: &AppHandle,
    monitor_index: usize,
    selection: Selection,
) -> Result<PathBuf, String> {
    close_all(app);

    let monitors = monitors(app)?;
    let geometry = *monitors
        .get(monitor_index)
        .ok_or_else(|| strings::CAPTURE_MONITOR_GONE.to_owned())?;

    // The overlays are windows like any other, and a window takes a moment to stop being on the
    // screen. Capturing immediately catches the overlay in the shot.
    std::thread::sleep(std::time::Duration::from_millis(120));

    let (x, y) = geometry.centre();
    let monitor = xcap::Monitor::from_point(x, y)
        .map_err(|error| strings::capture_failed(&error.to_string()))?;

    let image = monitor
        .capture_image()
        .map_err(|error| strings::capture_failed(&error.to_string()))?;

    let size = PixelSize {
        width: image.width(),
        height: image.height(),
    };
    let rect = driveshot_core::pixels_for(selection, geometry.overlay_size(), size)
        .ok_or_else(|| strings::CAPTURE_NOTHING_SELECTED.to_owned())?;

    let cropped =
        xcap::image::imageops::crop_imm(&image, rect.x, rect.y, rect.width, rect.height).to_image();

    let path = shot_path()?;
    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|error| {
            strings::capture_not_saved(&folder.display().to_string(), &error.to_string())
        })?;
    }
    cropped.save(&path).map_err(|error| {
        strings::capture_not_saved(&path.display().to_string(), &error.to_string())
    })?;

    Ok(path)
}

/// Where the next shot goes: a file named after the moment it was taken, in a folder of Driveshot's
/// own inside the pictures folder.
///
/// Local for now. Uploading it to a cloud drive is #6, and this folder is where the file waits
/// until that exists.
fn shot_path() -> Result<PathBuf, String> {
    let pictures = dirs::picture_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| strings::CAPTURE_NO_PICTURES_FOLDER.to_owned())?;

    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    Ok(pictures
        .join("Driveshot")
        .join(format!("driveshot-{stamp}.png")))
}

/// Every monitor, as Tauri describes it.
fn monitors(app: &AppHandle) -> Result<Vec<MonitorGeometry>, String> {
    let monitors = app
        .available_monitors()
        .map_err(|error| strings::capture_no_monitor_list(&error.to_string()))?;

    Ok(monitors
        .into_iter()
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            MonitorGeometry {
                x: position.x,
                y: position.y,
                width: size.width,
                height: size.height,
                scale: monitor.scale_factor(),
            }
        })
        .collect())
}

/// The labels of the overlay windows that are open.
fn overlay_labels(app: &AppHandle) -> Vec<String> {
    app.webview_windows()
        .into_keys()
        .filter(|label| label.starts_with(OVERLAY_PREFIX))
        .collect()
}

/// Opens one overlay, covering one monitor.
fn open_overlay(app: &AppHandle, index: usize, monitor: MonitorGeometry) -> tauri::Result<()> {
    let url = WebviewUrl::App(format!("{OVERLAY_PAGE}?monitor={index}").into());
    let label = format!("{OVERLAY_PREFIX}{index}");

    let window = WebviewWindowBuilder::new(app, label.clone(), url)
        .title("Driveshot")
        // The builder takes points, not physical pixels, and turns them into pixels with whichever
        // scale factor the window is created under - which is not necessarily the scale factor of
        // the monitor it is being sent to. So this is only where the window is born; the physical
        // rectangle it is meant to cover is set below, where nothing is converted (#22).
        .position(
            f64::from(monitor.x) / monitor.scale,
            f64::from(monitor.y) / monitor.scale,
        )
        .inner_size(
            f64::from(monitor.width) / monitor.scale,
            f64::from(monitor.height) / monitor.scale,
        )
        .decorations(false)
        // Windows gives an undecorated window its shadow by leaving the resize frame around it,
        // and then pulls the page inside in by that frame's width - eight pixels at 100%, ten at
        // 125% - and draws a one-pixel white border round the result. On an overlay meant to cover
        // a monitor exactly, that is an undimmed strip down each side and a white line around the
        // lot, and every selection read against a surface wider than the real one (#22).
        .shadow(false)
        .transparent(true)
        // A window appears before the page inside it has painted anything, and what shows in
        // those few frames is the web view's own background - white. So the window is created
        // hidden and shown by `ready` below, once the page says it has drawn itself (#20).
        .visible(false)
        // Insurance for a window manager that shows the window a frame early regardless. Tauri
        // documents this as ignored on Windows 8 and newer unless the alpha channel is 0, which
        // is exactly the value here.
        .background_color(Color(0, 0, 0, 0))
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        // Focus is what lets Escape reach the page.
        .focused(true)
        .build()?;

    // Physical pixels: the one coordinate system that means the same thing on every platform and
    // at every scaling. The window is still invisible here, so moving it shows nothing.
    window.set_position(PhysicalPosition::new(monitor.x, monitor.y))?;
    window.set_size(PhysicalSize::new(monitor.width, monitor.height))?;

    show_anyway_if_silent(app, label);
    Ok(())
}

/// How long an overlay is given to report that it has drawn itself.
///
/// Long enough that it never fires in practice, short enough that a user who pressed the key is
/// not left wondering.
const READY_DEADLINE: std::time::Duration = std::time::Duration::from_millis(1500);

/// Shows an overlay that never said it was ready.
///
/// The overlays start invisible and are shown by `ready`. If that message never arrives - a page
/// that failed to load, a script that threw - the key press would otherwise do nothing visible at
/// all, which is the worst way for this to fail: the user cannot tell Driveshot from a dead
/// keyboard. A late overlay is a far smaller problem than an absent one.
fn show_anyway_if_silent(app: &AppHandle, label: String) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(READY_DEADLINE);

        let Some(window) = app.get_webview_window(&label) else {
            return; // Cancelled, or already taken. Nothing to show.
        };
        if window.is_visible().unwrap_or(true) {
            return; // ready() got there first, which is the ordinary case.
        }

        eprintln!("{label} did not report itself ready; showing it anyway");
        if let Err(error) = window.show() {
            eprintln!("an overlay would not show: {error}");
        }
    });
}

/// Shows an overlay that has finished drawing itself.
///
/// Called by each overlay page once, as soon as it has painted. Until then the window exists but
/// is invisible, which is what keeps the white first frame off the screen (#20).
pub fn ready(app: &AppHandle, label: &str) {
    if !label.starts_with(OVERLAY_PREFIX) {
        eprintln!("something that is not an overlay reported itself ready: {label}");
        return;
    }

    let Some(window) = app.get_webview_window(label) else {
        // The user cancelled between the page loading and this arriving. Nothing to show.
        return;
    };

    if let Err(error) = window.show() {
        eprintln!("an overlay would not show: {error}");
        return;
    }
    // Showing a window does not always give it the keyboard, and without the keyboard Escape
    // cannot reach the page.
    if let Err(error) = window.set_focus() {
        eprintln!("an overlay would not take focus: {error}");
    }
}
