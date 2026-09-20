//! Logic for Driveshot that depends on neither the user interface nor an operating system API.
//!
//! The application captures part of the screen, uploads it to a cloud drive, publishes a share
//! link, and deletes the file once its retention has run out. The decisions behind those steps
//! live here, separated from the code that draws a window or makes a network call, so they can be
//! tested anywhere.
//!
//! What this crate holds:
//!
//! - [`Provider`]: which cloud drive a file went to.
//! - [`Retention`]: how long a file is kept, and when it falls due for deletion.
//! - [`ShotRecord`] and [`ShotIndex`]: what was uploaded, where it went, and when.
//!
//! What it deliberately does not hold: the capture itself, the OAuth exchange, the HTTP calls,
//! and the tray icon. Those need a screen, a browser or a network, and live in `src-tauri`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod provider;
mod record;
mod retention;

pub use error::{Error, Result};
pub use provider::Provider;
pub use record::{ShotIndex, ShotRecord};
pub use retention::Retention;
