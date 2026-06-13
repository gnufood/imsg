//! MAP XML parser for message listings (`MAP-msg-listing`).

use quick_xml::{events::Event, Reader, XmlVersion};
use thiserror::Error;

use crate::messages::MessageEntry;

/// quick-xml parse, attribute decode, and u32 invalid int.
#[derive(Debug, Error)]
pub enum MessageListingError {
    /// Underlying quick-xml reader error; also covers entity-decoding failures.
    #[error("XML error: {0}")]
    Parse(#[from] quick_xml::Error),
    /// Attribute encoding error from quick-xml.
    #[error("attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),
    /// Numeric attribute (`size`) could not be parsed as a `u32`.
    #[error("invalid integer attribute: {0}")]
    InvalidInt(#[from] std::num::ParseIntError),
}

/// Parses a `<MAP-msg-listing>` document from raw bytes.
///
/// Returns one [`MessageEntry`] per `<msg>` element in document order. Attributes absent from
/// an element default to their zero/false/empty value.
///
/// # Errors
///
/// Returns [`MessageListingError`] on malformed XML, undecodable attributes, non-UTF-8
/// attribute values, or a non-numeric `size` field.
pub fn parse_message_listing(xml: &[u8]) -> Result<Vec<MessageEntry>, MessageListingError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut messages = Vec::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"msg" => {
                let mut handle = String::new();
                let mut subject = String::new();
                let mut datetime = String::new();
                let mut sender_name = String::new();
                let mut sender_addressing = String::new();
                let mut recipient_name = String::new();
                let mut recipient_addressing = String::new();
                let mut msg_type = String::new();
                let mut size = 0u32;
                let mut read = false;
                let mut sent = false;
                for attr in e.attributes() {
                    let a = attr?;
                    let val = a.normalized_value(XmlVersion::Implicit1_0)?.into_owned();
                    match a.key.as_ref() {
                        b"handle" => handle = val,
                        b"subject" => subject = val,
                        b"datetime" => datetime = val,
                        b"sender_name" => sender_name = val,
                        b"sender_addressing" => sender_addressing = val,
                        b"recipient_name" => recipient_name = val,
                        b"recipient_addressing" => recipient_addressing = val,
                        b"type" => msg_type = val,
                        b"size" => size = val.parse()?,
                        b"read" => read = val == "yes",
                        b"sent" => sent = val == "yes",
                        _ => {}
                    }
                }
                messages.push(MessageEntry {
                    handle,
                    subject,
                    datetime,
                    sender_name,
                    sender_addressing,
                    recipient_name,
                    recipient_addressing,
                    msg_type,
                    size,
                    read,
                    sent,
                });
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(messages)
}
