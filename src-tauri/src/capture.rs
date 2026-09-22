//! Taking a shot of part of the screen.
//!
//! The sequence is: cover every monitor with an overlay, let the user drag a rectangle on one of
//! them, capture that monitor, cut the rectangle out, and write it to a file.
//!
//! # The overlays outlive the capture
//!
//! There is one overlay window per monitor and they are **built once, at startup, and kept**.
//! A capture shows them and puts them away again; it does not create them. That is not an
//! optimisation of the tidy sort - building a web view and loading a page into it took about a
//! second, and that second was the whole delay between pressing the key and seeing the screen dim
//! (#23). Nothing else in a capture is slow.
//!
//! What it costs is that the page is the same page as last time, so it remembers the last
//! selection. Each capture therefore starts by telling the overlays it has started; they put
//! themselves back to how they began and say when they are ready, and only then are they shown.
//! That handshake is also what keeps the web view's white first frame off the screen (#20), and
//! it is now a round trip rather than a page load.
//!
//! Monitors are plugged in and unplugged while Driveshot sits in the tray, so the set of windows
//! is reconciled against the monitors at the start of every capture rather than trusted from
//! startup.
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
use std::sync::atomic::{AtomicBool, Ordering};

use driveshot_core::{LogicalSize, PixelSize, Selection};
use tauri::utils::config::Color;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder,
};

use crate::strings;

/// The page every overlay window shows.
const OVERLAY_PAGE: &str = "overlay.html";

/// What an overlay window's label starts with, so that they can all be found again.
const OVERLAY_PREFIX: &str = "overlay-";

/// The event that tells the overlays a capture has started and they should put themselves back to
/// how they began.
const CAPTURE_BEGIN: &str = "capture-begin";

/// Whether a capture is happening now.
///
/// One press covers the screen; a second press while it is covered is the user pressing twice, and
/// is ignored rather than starting again underneath. The overlay windows cannot answer this on
/// their own any more: they exist whether or not a capture is running, and between the key press
/// and the overlays reporting ready none of them is on the screen yet.
#[derive(Default)]
pub struct Capturing(AtomicBool);

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

/// Builds the overlay windows at startup, so that the first capture does not have to.
///
/// A failure here is not worth refusing to start over: the windows are built again at the start of
/// a capture, and the only cost of that is the second this exists to avoid. It is still said out
/// loud, because a machine where this fails every time is a machine where every capture is slow.
pub fn prepare(app: &AppHandle) {
    if let Err(error) = ensure_overlays(app) {
        eprintln!("the overlays could not be built at startup, so the first capture will build them: {error}");
    }
}

/// Covers every monitor and returns how many overlays were shown.
///
/// The windows already exist; this reconciles them against the monitors as they are now, tells
/// them a capture has started, and leaves them to show themselves once they say they are ready.
pub fn begin(app: &AppHandle) -> Result<usize, String> {
    if app.state::<Capturing>().0.swap(true, Ordering::SeqCst) {
        // Already covering the screen. The user pressed the key twice.
        return Ok(0);
    }

    let built = match ensure_overlays(app) {
        Ok(built) => built,
        Err(error) => {
            stop_capturing(app);
            return Err(error);
        }
    };

    // Every overlay hears this. Each one puts itself back to how it began - no selection drawn, no
    // shot already asked for, the hint showing - and calls `ready` when it has, which is what
    // shows it.
    if let Err(error) = app.emit(CAPTURE_BEGIN, ()) {
        stop_capturing(app);
        return Err(strings::capture_overlay_failed(&error.to_string()));
    }

    for (index, fresh) in built.iter().enumerate() {
        // An overlay built a moment ago is still loading its page, and it was never listening
        // when the event above went out. It answers when it has drawn itself for the first time
        // instead, which is the slow case this change exists to keep off the ordinary path.
        let deadline = if *fresh {
            FIRST_DRAW_DEADLINE
        } else {
            READY_DEADLINE
        };
        show_when_ready(app, overlay_label(index), deadline);
    }

    Ok(built.len())
}

/// Puts the overlays away. Safe to call when none is showing.
///
/// They are hidden rather than closed: closing them is what made the next capture slow.
fn hide_all(app: &AppHandle) {
    for label in overlay_labels(app) {
        if let Some(window) = app.get_webview_window(&label) {
            if let Err(error) = window.hide() {
                eprintln!("an overlay window would not hide: {error}");
            }
        }
    }
}

/// Puts the overlays away and ends the capture. Escape, a right click, or a click without a drag.
pub fn cancel(app: &AppHandle) {
    hide_all(app);
    stop_capturing(app);
}

/// Captures what the user selected on `monitor_index` and writes it to a file.
///
/// Returns where it was written. The overlays are hidden first, so that they are not in the shot.
pub fn finish(
    app: &AppHandle,
    monitor_index: usize,
    selection: Selection,
) -> Result<PathBuf, String> {
    hide_all(app);
    stop_capturing(app);

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

/// The label of the overlay covering the monitor at `index`.
fn overlay_label(index: usize) -> String {
    format!("{OVERLAY_PREFIX}{index}")
}

/// The labels of the overlay windows that exist, showing or not.
fn overlay_labels(app: &AppHandle) -> Vec<String> {
    app.webview_windows()
        .into_keys()
        .filter(|label| label.starts_with(OVERLAY_PREFIX))
        .collect()
}

/// Notes that the capture is over, whether it produced a shot or not.
fn stop_capturing(app: &AppHandle) {
    app.state::<Capturing>().0.store(false, Ordering::SeqCst);
}

/// Makes sure there is exactly one overlay window per monitor, each covering its own.
///
/// Called at startup and again at the start of every capture. A monitor can be plugged in,
/// unplugged, moved or rescaled while Driveshot sits in the tray, so the windows built last time
/// are checked against the monitors there are now rather than assumed to still fit.
///
/// Returns one answer per monitor: whether that overlay had to be built here rather than being
/// one that was already waiting. The two are given different amounts of time to report themselves
/// ready, because one of them has a page to load first.
fn ensure_overlays(app: &AppHandle) -> Result<Vec<bool>, String> {
    let monitors = monitors(app)?;
    if monitors.is_empty() {
        return Err(strings::CAPTURE_NO_MONITORS.to_owned());
    }

    // A monitor that has gone leaves an overlay with nothing to cover. Its label carries an index
    // past the end of the list, so it would never be shown again either.
    for label in overlay_labels(app) {
        let beyond = label
            .strip_prefix(OVERLAY_PREFIX)
            .and_then(|index| index.parse::<usize>().ok())
            .is_none_or(|index| index >= monitors.len());
        if beyond {
            if let Some(window) = app.get_webview_window(&label) {
                if let Err(error) = window.close() {
                    eprintln!("an overlay for a monitor that has gone would not close: {error}");
                }
            }
        }
    }

    let mut built = Vec::with_capacity(monitors.len());
    for (index, monitor) in monitors.iter().enumerate() {
        match place_overlay(app, index, *monitor) {
            Ok(fresh) => built.push(fresh),
            Err(error) => {
                // One monitor failing must not leave the others covered: the user would be looking
                // at a screen they cannot dismiss.
                hide_all(app);
                return Err(strings::capture_overlay_failed(&error.to_string()));
            }
        }
    }

    Ok(built)
}

/// Builds the overlay for one monitor if it is not there, and puts it over that monitor.
///
/// Returns whether it had to build it.
fn place_overlay(app: &AppHandle, index: usize, monitor: MonitorGeometry) -> tauri::Result<bool> {
    let label = overlay_label(index);

    let (window, fresh) = match app.get_webview_window(&label) {
        Some(window) => (window, false),
        None => (build_overlay(app, index, monitor)?, true),
    };

    // Physical pixels: the one coordinate system that means the same thing on every platform and
    // at every scaling. The window is invisible while this happens, so moving it shows nothing.
    window.set_position(PhysicalPosition::new(monitor.x, monitor.y))?;
    window.set_size(PhysicalSize::new(monitor.width, monitor.height))?;
    Ok(fresh)
}

/// Builds one overlay window, hidden, with its page loading.
fn build_overlay(
    app: &AppHandle,
    index: usize,
    monitor: MonitorGeometry,
) -> tauri::Result<tauri::WebviewWindow> {
    let url = WebviewUrl::App(format!("{OVERLAY_PAGE}?monitor={index}").into());

    WebviewWindowBuilder::new(app, overlay_label(index), url)
        .title("Driveshot")
        // The builder takes points, not physical pixels, and turns them into pixels with whichever
        // scale factor the window is created under - which is not necessarily the scale factor of
        // the monitor it is being sent to. So this is only where the window is born; the physical
        // rectangle it is meant to cover is set by `place_overlay` (#22).
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
        // Built invisible and shown by `ready` once its page says it has drawn itself. At startup
        // that is what keeps a web view's white first frame off the screen (#20); at every capture
        // after it, the page has been drawn for a long time and this is what keeps the *previous*
        // capture's selection off it.
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
        .build()
}

/// How long an overlay that was already waiting is given to answer that it is ready.
///
/// What is being waited for is a message out to the page and one back, and a repaint of a page
/// that is already loaded. That is a few milliseconds when it works at all, so half a second is
/// the point at which it is not going to.
const READY_DEADLINE: std::time::Duration = std::time::Duration::from_millis(500);

/// How long an overlay built during the capture is given instead.
///
/// It has a web view to start and a page to load, which is the second that #23 was about. It only
/// happens when a monitor appeared since the last capture, and the alternative to waiting is
/// showing a web view that has drawn nothing yet - which is #20.
const FIRST_DRAW_DEADLINE: std::time::Duration = std::time::Duration::from_millis(1500);

/// Waits for an overlay to say it is ready, and shows it regardless if it does not.
///
/// An overlay that never answers would leave the key press doing **nothing visible at all** - the
/// worst way for this to fail, because the user cannot tell Driveshot from a dead keyboard. A late
/// overlay, or one still showing the hint from a moment ago, is a far smaller problem than an
/// absent one.
fn show_when_ready(app: &AppHandle, label: String, deadline: std::time::Duration) {
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(deadline);

        if !app.state::<Capturing>().0.load(Ordering::SeqCst) {
            return; // Cancelled, or the shot has already been taken.
        }
        let Some(window) = app.get_webview_window(&label) else {
            return; // The monitor went away. Nothing to show.
        };
        if window.is_visible().unwrap_or(true) {
            return; // ready() got there first, which is the ordinary case.
        }

        eprintln!("{label} did not answer that it was ready; showing it anyway");
        show(&window, &label);
    });
}

/// Shows an overlay that has said it is ready.
///
/// Called by each overlay page once per capture, as soon as it has put itself back to how it
/// began. Until then the window exists but is invisible, so what reaches the screen is never a
/// blank web view (#20) nor the last capture's selection.
pub fn ready(app: &AppHandle, label: &str) {
    if !label.starts_with(OVERLAY_PREFIX) {
        eprintln!("something that is not an overlay reported itself ready: {label}");
        return;
    }
    if !app.state::<Capturing>().0.load(Ordering::SeqCst) {
        // No capture is running. The page says this once when it first loads, too.
        return;
    }

    let Some(window) = app.get_webview_window(label) else {
        return; // The monitor went away between the capture starting and this arriving.
    };
    show(&window, label);
}

/// Shows one overlay and gives it the keyboard.
fn show(window: &tauri::WebviewWindow, label: &str) {
    if let Err(error) = window.show() {
        eprintln!("{label} would not show: {error}");
        return;
    }
    // Showing a window does not always give it the keyboard, and without the keyboard Escape
    // cannot reach the page.
    if let Err(error) = window.set_focus() {
        eprintln!("{label} would not take focus: {error}");
    }
}
