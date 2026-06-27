//! Session lifecycle — SDP lookup, RFCOMM connect, OBEX handshake, retry policy, MNS relay, and store sync.

pub mod conn;
pub mod fetch;
pub mod lifecycle;
pub mod live;
pub mod loop_util;
pub mod mns;
pub mod outbox;
pub mod relay;
pub mod retry;
pub mod sync;
pub mod util;
pub mod watch;

// `pub mod` under cfg(test): shared across mns/relay test modules; must be `pub` to escape
// `redundant_pub_crate`. cfg(test)-gating keeps it out of the real crate API.
#[cfg(test)]
pub mod test_support;

pub use map_core::mns_event::{EventType, MnsEvent};
pub use map_core::MapError;
pub use pbap_core::PbapError;
pub use relay::run_mns_relay;
pub use retry::{classify, Disposition};
pub use transport::TransportError;

/// MAP/PBAP/transport layer errors.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// OBEX MAP layer; either CONNECT failed or a command was rejected.
    #[error("MAP session error: {0}")]
    Map(#[from] MapError),
    /// OBEX PBAP layer; CONNECT failed or command rejected.
    #[error("PBAP session error: {0}")]
    Pbap(#[from] PbapError),
    /// RFCOMM or pre-OBEX I/O failure.
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
}
