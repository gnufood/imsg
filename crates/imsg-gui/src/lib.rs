//! Tauri backend for the imsg GUI.
//!
//! Adapter only: mirrors the CLI's local-store read path (own `Store` connection, bypassing
//! the broker) and proxies device/write operations through `imsg-broker-client`. No domain
//! logic lives here.

pub mod commands;
pub mod config;
pub mod contacts;
pub mod daemon;
pub mod delete;
pub mod discover;
pub mod dto;
pub mod gate;
pub mod reads;
pub mod send;
pub mod sync;

#[cfg(test)]
pub mod test_support;
