//! `send` — forwards to `imsg-broker-client`'s shared write path.
//!
//! The daemon holds the sole live MAP session, so this never touches the GUI's own `Store`
//! directly — the outbox row lives on the daemon's side, picked up by the GUI's poll loop.

/// Sends an SMS via the broker's tracked-outbox path.
///
/// # Errors
///
/// Returns [`broker_client::WriteError`] if the broker can't be reached or rejects the request.
pub async fn send(
    addr: &str,
    number: String,
    message: String,
) -> Result<String, broker_client::WriteError> {
    broker_client::send(addr, number, message).await
}

#[cfg(test)]
mod tests;
