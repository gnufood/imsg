//! PBAP protocol logic — contact pull and vCard retrieval.

pub mod client;
pub mod contacts;
pub mod params;
pub mod phonebook;

pub use contacts::normalize_number;
pub use contacts::CardEntry;
pub use contacts::CardListingError;
pub use formats::vcard::{Contact, ContactError};
pub use obex_core::client::ObexError;
pub use obex_core::TransportError;

use thiserror::Error;

/// PBAP client errors — OBEX, transport, server response, vCard parse, XML listing parse, and input validation.
#[derive(Debug, Error)]
pub enum PbapError {
    /// Connection state violation, packet codec failure, or server rejection.
    #[error("OBEX: {0}")]
    Obex(#[from] ObexError),
    /// Framing error or OS-level socket failure.
    #[error("transport: {0}")]
    Transport(#[from] TransportError),
    /// Remote device returned a non-OK response opcode; inner byte is the opcode.
    #[error("server returned error opcode {0:#04x}")]
    ServerError(u8),
    /// Transport stream ended before a complete packet was received.
    #[error("unexpected end of stream")]
    UnexpectedEof,
    /// Response body exceeded the 4 MiB safety ceiling.
    #[error("response body too large")]
    ResponseTooLarge,
    /// Response body is not valid UTF-8; cannot parse as vCard text.
    #[error("response body is not UTF-8")]
    InvalidEncoding,
    /// calcard failed to parse a vCard in the response body.
    #[error("vCard parse error: {0}")]
    Contact(#[from] ContactError),
    /// Remote vCard-listing XML was malformed or a `<card>` element lacked the required `handle` attribute.
    #[error("card listing: {0}")]
    CardListing(#[from] CardListingError),
    /// A user-supplied parameter failed wire-format validation; inner message names the violated constraint.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
}
