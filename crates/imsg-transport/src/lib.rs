//! RFCOMM and TCP raw stream connectors. Wrap the returned stream with
//! [`obex_core::wrap`] to obtain an OBEX-framed transport.

/// iroh QUIC hub/spoke connectors — replaces the TCP bridge for remote machines.
pub mod iroh;
/// RFCOMM Bluetooth Classic stream connector via [`bluer`].
pub mod rfcomm;
/// TCP stream connector for the TCP bridge dev proxy.
pub mod tcp;

pub use obex_core::TransportError;
