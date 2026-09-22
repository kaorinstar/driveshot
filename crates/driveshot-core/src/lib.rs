//! Logic for Driveshot that depends on neither the user interface nor an operating system API.
//!
//! The application captures part of the screen, uploads it to a cloud drive, publishes a share
//! link, and deletes the file once its retention has run out. The decisions behind those steps
//! live here, separated from the code that draws a window or makes a network call, so they can be
//! tested anywhere.
//!
//! What this crate holds:
//!
//! - [`Selection`] and [`pixels_for`]: which pixels of a screenshot the user selected.
//! - [`Provider`]: which cloud drive a file went to.
//! - [`Retention`]: how long a file is kept, and when it falls due for deletion.
//! - [`ShotRecord`] and [`ShotIndex`]: what was uploaded, where it went, and when.
//! - [`oauth`]: the authorization URL, the PKCE challenge, and reading the redirect back.
//!
//! What it deliberately does not hold: the capture itself, the browser and the loopback listener
//! a sign-in needs, the HTTP calls, and the tray icon. Those need a screen, a browser or a
//! network, and live in `src-tauri`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod geometry;
pub mod oauth;
mod provider;
mod record;
mod retention;

pub use error::{Error, Result};
pub use geometry::{pixels_for, LogicalSize, PixelRect, PixelSize, Selection};
pub use provider::Provider;
pub use record::{ShotIndex, ShotRecord};
pub use retention::Retention;
