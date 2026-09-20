// Hides the console window that Windows otherwise opens behind the application. Only in a release
// build: a debug build keeps the console, because that is where a panic is printed.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![forbid(unsafe_code)]

//! The Driveshot application.
//!
//! What is here today is the skeleton: the window opens, and it asks the Rust side for the two
//! things `driveshot-core` already decides - which cloud drives exist, and when a file uploaded
//! now would fall due for deletion. Capture, upload, sharing and deletion are not implemented
//! yet; the roadmap in README.md says in which order they arrive.
//!
//! The rule that shapes this file: a calculation belongs in `driveshot-core`, where it is tested
//! on every platform. What stays here is what genuinely needs a screen, a file or a network.

use chrono::{SecondsFormat, Utc};
use driveshot_core::{Provider, Retention};
use serde::Serialize;

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

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![providers, expiry_preview])
        .run(tauri::generate_context!())
        .expect("the Driveshot window could not be created");
}
