//! `#[tauri::command]` shim over `crate::delete`.

use super::CommandError;

/// Sets the deleted flag on the device and removes the message from the broker's store.
///
/// # Errors
///
/// Returns [`CommandError`] if the broker can't be reached or rejects the request.
#[tauri::command]
#[specta::specta]
pub async fn delete(addr: String, handle: String, folder: String) -> Result<String, CommandError> {
    Ok(crate::delete::delete(&addr, handle, folder).await?)
}

#[cfg(test)]
mod tests;
