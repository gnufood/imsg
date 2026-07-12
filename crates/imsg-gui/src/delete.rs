//! `delete` — forwards to `imsg-broker-client`'s shared write path.
//!
//! The daemon holds the sole live MAP session, so this never touches the GUI's own `Store`
//! directly — the broker's own store loses the row, and the GUI's poll loop picks up the
//! change from there.
//!
//! `--undelete` has no GUI equivalent: iOS does not respect MAP `SetMessageStatus` with
//! `StatusIndicator=DELETED`/`StatusValue=0`, so the CLI treats it as a local no-op — nothing
//! for this module to forward.

/// Sets the deleted flag on the device and removes the message from the broker's store.
///
/// # Errors
///
/// Returns [`broker_client::WriteError`] if the broker can't be reached or rejects the request.
pub async fn delete(
    addr: &str,
    handle: String,
    folder: String,
) -> Result<String, broker_client::WriteError> {
    broker_client::delete(addr, handle, folder).await
}

#[cfg(test)]
mod tests;
