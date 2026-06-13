//! bMessage encoder and parser (MAP spec appendix B).
//!
//! Parse with [`BMessage::parse`]; encode with [`BMessage::encode`].

mod encode;
mod parser;
mod types;

pub use types::{BBody, BEnvelope, BMessage, BMessageError, BVCard, MessageStatus, MessageType};

impl BMessage {
    /// Parses a bMessage from UTF-8 text, accepting CRLF and LF line endings.
    ///
    /// # Errors
    ///
    /// Returns [`BMessageError`] when a required field is absent, a section is
    /// unterminated, or STATUS/TYPE hold unrecognised values.
    pub fn parse(input: &str) -> Result<Self, BMessageError> {
        parser::parse_bmessage(input)
    }
}
