//! iroh QUIC hub/spoke transport — replaces the TCP bridge for remote machines.
//!
//! The hub owns the RFCOMM link and runs [`crate::iroh::run_hub`]; spokes reach it over QUIC
//! via the hub's [`crate::iroh::EndpointId`]. Each profile rides its own ALPN-tagged
//! bidirectional stream: MAP and PBAP
//! requests are proxied spoke→hub into RFCOMM, MNS events are fanned hub→spoke.

use obex_core::TransportError;
use tokio::io::Join;

pub use iroh::endpoint::{Connection, RecvStream, SendStream};
pub use iroh::{Endpoint, EndpointAddr, EndpointId, SecretKey};

mod hub;
mod key;
mod spoke;
mod stream;

pub use hub::run_hub;
pub use key::load_or_create_key;
pub use spoke::{bind_spoke, connect_map_hub, connect_mns_hub, connect_pbap_hub};
pub use stream::{HubRecvStream, HubStream};

/// ALPN for spoke→hub MAP request streams.
pub const MAP_ALPN: &[u8] = b"imsg-map/1";
/// ALPN for spoke→hub PBAP request streams.
pub const PBAP_ALPN: &[u8] = b"imsg-pbap/1";
/// ALPN for hub→spoke MNS event streams.
pub const MNS_ALPN: &[u8] = b"imsg-mns/1";

/// Bidirectional QUIC stream presented to OBEX framing as a single duplex I/O object.
pub type SpokeStream = Join<RecvStream, SendStream>;

pub(crate) fn iroh_err<E: std::fmt::Display>(e: E) -> TransportError {
    TransportError::External(e.to_string())
}
