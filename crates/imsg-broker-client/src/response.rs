//! Interprets a [`BrokerResponse`] as its expected success payload or a typed failure.
//!
//! Shared by every consumer (`Send`, `SendLive`, `Delete`, `SyncContacts`, ...) instead of
//! each one hand-rolling the same match over response variants.

use ipc::{BrokerResponse, Reason};

/// A text-returning broker request answered with something other than
/// [`BrokerResponse::Text`].
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    /// The device/session rejected the operation.
    #[error("{0}")]
    Failed(Reason),
    /// IPC-plumbing failure reported by the broker (malformed frame, broker shutting down).
    #[error("{0}")]
    Error(String),
    /// A response shape no text-returning request should produce (e.g. `Messages`, `StatusInfo`).
    #[error("unexpected broker response: {0:?}")]
    Unexpected(Box<BrokerResponse>),
}

/// Extracts the success text from a [`BrokerResponse`] to a text-returning request, or the typed
/// [`CallError`] describing why it didn't succeed.
///
/// # Errors
///
/// Returns [`CallError::Failed`] for a device/session rejection, [`CallError::Error`] for an
/// IPC-plumbing failure, or [`CallError::Unexpected`] for any other response shape.
pub fn text_result(resp: BrokerResponse) -> Result<String, CallError> {
    match resp {
        BrokerResponse::Text(s) => Ok(s),
        BrokerResponse::Failed(reason) => Err(CallError::Failed(reason)),
        BrokerResponse::Error(e) => Err(CallError::Error(e)),
        other => Err(CallError::Unexpected(Box::new(other))),
    }
}

/// Extracts the upserted-row count from a [`BrokerResponse::ContactsSynced`], or the typed
/// [`CallError`] describing why it didn't succeed.
///
/// # Errors
///
/// Returns [`CallError::Failed`] for a device/session rejection, [`CallError::Error`] for an
/// IPC-plumbing failure, or [`CallError::Unexpected`] for any other response shape.
pub fn contacts_synced_result(resp: BrokerResponse) -> Result<usize, CallError> {
    match resp {
        BrokerResponse::ContactsSynced { count } => Ok(count),
        BrokerResponse::Failed(reason) => Err(CallError::Failed(reason)),
        BrokerResponse::Error(e) => Err(CallError::Error(e)),
        other => Err(CallError::Unexpected(Box::new(other))),
    }
}

#[cfg(test)]
mod tests;
