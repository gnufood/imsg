//! MAP protocol logic — commands, folder navigation, MNS server, and iOS quirks.

pub mod client;
pub mod folders;
pub mod messages;
pub mod mns_event;
pub mod mns_server;
pub mod params;
pub mod quirks;
pub mod xml;

pub use formats::bmessage::{BMessage, BMessageError, MessageStatus, MessageType};
pub use formats::xml::{FolderEntry, FolderListing, XmlError};

pub use obex_core::client::ObexError;
pub use obex_core::TransportError;
use thiserror::Error;

/// MAP client/MNS errors — OBEX, transport, server response, parse, input validation, and encoding.
#[derive(Debug, Error)]
pub enum MapError {
    /// Wraps [`ObexError`]; packet decode, invalid OBEX state, or PUT missing handle.
    #[error("OBEX: {0}")]
    Obex(#[from] ObexError),
    /// Wraps [`TransportError`]; stream closed, framing error, or Bluetooth I/O fault.
    #[error("transport: {0}")]
    Transport(#[from] TransportError),
    /// Remote device returned a non-OK response opcode.
    #[error("server returned error opcode {0:#04x}")]
    ServerError(u8),
    /// Transport stream ended before a complete packet was received.
    #[error("unexpected end of stream")]
    UnexpectedEof,
    /// Message listing XML response could not be parsed.
    #[error("message listing parse error: {0}")]
    MessageListing(#[from] xml::MessageListingError),
    /// Response body exceeded the 4 MiB safety ceiling.
    #[error("response body too large")]
    ResponseTooLarge,
    /// Response body is not valid UTF-8.
    #[error("response body is not UTF-8")]
    InvalidEncoding,
    /// bMessage body could not be parsed.
    #[error("invalid bMessage: {0}")]
    InvalidMessage(#[from] BMessageError),
    /// PUT response was OK but contained no Name header; remote did not assign a handle.
    #[error("push message response missing handle")]
    MissingHandle,
    /// Caller-supplied input violated a MAP protocol constraint.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
    /// Folder listing XML response could not be parsed.
    #[error("folder listing parse error: {0}")]
    FolderListing(#[from] XmlError),
}
