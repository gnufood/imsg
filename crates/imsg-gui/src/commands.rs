//! `#[tauri::command]` shims — the invoke-handler boundary a `tauri_specta::Builder` wraps.
//!
//! Split by domain.
//!
//! `String`/`PathBuf` args carry `#[allow(clippy::needless_pass_by_value)]`:
//! `#[tauri::command]` arguments are deserialized from IPC and must be owned.

pub mod config;
pub mod contacts;
pub mod daemon;
pub mod delete;
pub mod discover;
pub mod gate;
pub mod reads;
pub mod send;

use serde::Serialize;
use specta::Type;

/// Failure surfaced to the frontend from any `#[tauri::command]` in this crate.
///
/// Wraps the source error's `Display` text. No converted source leaks key material or
/// `SQLite`/service-manager internals.
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

impl From<crate::daemon::provision::ProvisionError> for CommandError {
    fn from(err: crate::daemon::provision::ProvisionError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<broker_client::WriteError> for CommandError {
    fn from(err: broker_client::WriteError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<broker_client::ContactsError> for CommandError {
    fn from(err: broker_client::ContactsError) -> Self {
        Self { message: err.to_string() }
    }
}

impl From<transport::discover::DiscoverError> for CommandError {
    fn from(err: transport::discover::DiscoverError) -> Self {
        Self { message: err.to_string() }
    }
}

/// Registers every `#[tauri::command]` in this crate on a fresh `tauri_specta::Builder`.
///
/// Shared by the `bindings.ts` export step and `main.rs`, so the list is never hand-duplicated.
///
/// `i64`/`u64` fields export as JS `bigint` via semantic types; `specta_typescript` otherwise
/// forbids exporting them (precision loss past 2^53). Only the generated `bindings.ts` wrappers
/// apply the conversion, so raw `invoke()` bypasses it.
#[must_use]
pub fn builder<R: tauri::Runtime>() -> tauri_specta::Builder<R> {
    tauri_specta::Builder::<R>::new()
        .commands(tauri_specta::collect_commands![
            reads::get_by_handle,
            reads::list_messages,
            reads::mark_read,
            reads::threads,
            contacts::list_contacts,
            contacts::get_contact,
            contacts::lookup_contact,
            contacts::sync_contacts_now,
            config::config_show,
            config::config_set_device,
            config::config_set_channels,
            config::config_is_device_configured,
            config::config_set_device_and_channels,
            config::config_set_broker_security_level,
            discover::discover_list_paired_devices,
            discover::discover_resolve_channels,
            gate::gate_status,
            gate::gate_proceed,
            daemon::daemon_install,
            daemon::daemon_uninstall,
            daemon::daemon_status,
            daemon::daemon_service_status,
            daemon::daemon_stop,
            daemon::daemon_restart,
            daemon::broker_status,
            send::send,
            delete::delete,
        ])
        .semantic_types(
            specta_typescript::semantic::Configuration::default().enable_lossless_bigints(),
        )
}

#[cfg(test)]
mod tests;
