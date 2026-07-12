//! `#[tauri::command]` shim over `crate::send`.

use super::CommandError;

/// Sends an SMS via the broker's tracked-outbox path.
///
/// # Errors
///
/// Returns [`CommandError`] if the broker can't be reached or rejects the request.
#[tauri::command]
#[specta::specta]
pub async fn send(addr: String, number: String, message: String) -> Result<String, CommandError> {
    Ok(crate::send::send(&addr, number, message).await?)
}

#[cfg(test)]
mod tests;
