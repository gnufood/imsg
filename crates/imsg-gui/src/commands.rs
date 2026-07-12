//! `#[tauri::command]` shims — the invoke-handler boundary a `tauri_specta::Builder` wraps.
//!
//! Split by domain (`reads`, `config`, `daemon`, `send`, `delete`) to stay under the
//! 250-line module ceiling as the command surface grows; `CommandError` (shared across every
//! domain) lives here.

pub mod config;
pub mod daemon;
pub mod delete;
pub mod reads;
pub mod send;

use serde::Serialize;
use specta::Type;

/// Failure surfaced to the frontend from any `#[tauri::command]` in this crate.
///
/// Wraps the underlying error's `Display` text; none of the sources converted below leak
/// secrets (no raw key material, no `SQLite`/service-manager internals beyond a driver-level
/// description).
#[derive(Debug, Serialize, Type, thiserror::Error)]
#[error("{message}")]
pub struct CommandError {
    message: String,
}

impl From<store::Error> for CommandError {
    fn from(err: store::Error) -> Self {
        Self { message: err.to_string() }
    }
}

// Fully qualified: this module's own `pub mod config;` shadows the `config` crate name for
// unqualified references within this file.
impl From<::config::ConfigError> for CommandError {
    fn from(err: ::config::ConfigError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<service::Error> for CommandError {
    fn from(err: service::Error) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<crate::daemon::StopError> for CommandError {
    fn from(err: crate::daemon::StopError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<broker_client::WriteError> for CommandError {
    fn from(err: broker_client::WriteError) -> Self {
        Self { message: err.to_string() }
    }
}

#[cfg(test)]
mod tests;
