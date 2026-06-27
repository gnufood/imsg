//! MAP MNS event types, event struct, parser error, and XML event-report parser.

use std::fmt;

use obex_core::client::ObexError;
use obex_core::TransportError;
use quick_xml::{events::Event, Reader, XmlVersion};
use thiserror::Error;

/// Reported in `MAP-event-report` `<event type=...>` attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// New message in store; `handle` and `folder` are always present.
    NewMessage,
    /// An outbound message was delivered to its recipient.
    DeliverySuccess,
    /// An outbound message was accepted by the network for delivery.
    SendingSuccess,
    /// Delivery of an outbound message permanently failed.
    DeliveryFailure,
    /// The network rejected an outbound message.
    SendingFailure,
    /// Message removed from store; `handle` present, `folder` is the folder it was in.
    MessageDeleted,
    /// A message was moved to a different folder; `old_folder` carries the previous location.
    MessageShift,
    /// Message store is full; no new messages can be received until space is freed.
    MemoryFull,
    /// Message store has space available again after a `MemoryFull` event.
    MemoryAvailable,
    /// Read/unread flag toggled on device; `handle` present.
    ReadStatusChanged,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Canonical MAP 1.4 event-type strings — must stay in sync with `event_type_from_str`.
        f.write_str(match self {
            Self::NewMessage => "NewMessage",
            Self::DeliverySuccess => "DeliverySuccess",
            Self::SendingSuccess => "SendingSuccess",
            Self::DeliveryFailure => "DeliveryFailure",
            Self::SendingFailure => "SendingFailure",
            Self::MessageDeleted => "MessageDeleted",
            Self::MessageShift => "MessageShift",
            Self::MemoryFull => "MemoryFull",
            Self::MemoryAvailable => "MemoryAvailable",
            Self::ReadStatusChanged => "ReadStatusChanged",
        })
    }
}

/// Fields absent in the `<event>` element are `None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MnsEvent {
    event_type: EventType,
    handle: Option<String>,
    folder: Option<String>,
    old_folder: Option<String>,
    msg_type: Option<String>,
    datetime: Option<String>,
}

impl MnsEvent {
    /// Governs which optional fields are present.
    #[must_use]
    pub const fn event_type(&self) -> EventType {
        self.event_type
    }

    /// Opaque message handle assigned by the device; absent for `MemoryFull`/`MemoryAvailable`.
    #[must_use]
    pub fn handle(&self) -> Option<&str> {
        self.handle.as_deref()
    }

    /// Current folder path, e.g. `TELECOM/MSG/INBOX`; absent for memory events.
    #[must_use]
    pub fn folder(&self) -> Option<&str> {
        self.folder.as_deref()
    }

    /// Previous folder path; present only for `MessageShift`.
    #[must_use]
    pub fn old_folder(&self) -> Option<&str> {
        self.old_folder.as_deref()
    }

    /// Message type string, e.g. `SMS_GSM`; absent for memory events.
    #[must_use]
    pub fn msg_type(&self) -> Option<&str> {
        self.msg_type.as_deref()
    }

    /// ISO 8601 basic datetime string, e.g. `20260604T120000`; present for `NewMessage`.
    #[must_use]
    pub fn datetime(&self) -> Option<&str> {
        self.datetime.as_deref()
    }
}

/// MNS server and event report XML parsing errors.
#[derive(Debug, Error)]
pub enum MnsError {
    /// quick-xml reader error encountered while parsing the event report body.
    #[error("XML error: {0}")]
    Parse(#[from] quick_xml::Error),
    /// Attribute value in the `<event>` element could not be decoded.
    #[error("attribute error: {0}")]
    Attr(#[from] quick_xml::events::attributes::AttrError),
    /// `type` attribute value did not match any MAP 1.4 event type; inner string is the raw value.
    #[error("unknown event type: {0}")]
    UnknownEventType(String),
    /// `<event>` element present but `type` attribute is absent.
    #[error("<event> element has no type attribute")]
    MissingEventType,
    /// Document contained no `<event>` element; body may be empty or malformed.
    #[error("MAP-event-report contained no <event> element")]
    MissingEvent,
    /// OBEX protocol-level error during MNS server operation.
    #[error("OBEX: {0}")]
    Obex(#[from] ObexError),
    /// Transport I/O error during MNS server operation.
    #[error("transport: {0}")]
    Transport(#[from] TransportError),
    /// Transport stream closed before the device sent a clean OBEX DISCONNECT.
    #[error("unexpected end of stream")]
    UnexpectedEof,
    /// Device sent an opcode other than PUT, `PUT_FINAL`, or DISCONNECT after CONNECT.
    #[error("unexpected OBEX opcode {0:#04x} from MNS client")]
    UnexpectedOpcode(u8),
    /// CONNECT request Target header is absent or does not match the MNS service UUID.
    #[error("CONNECT Target UUID does not match MNS service UUID")]
    InvalidTarget,
}

/// Parses an OBEX PUT body into an [`MnsEvent`].
///
/// The document must contain at least one `<event>` element; only the first is returned. The
/// `type` attribute is required and must match a known [`EventType`] value. All other attributes
/// are optional and default to `None` when absent.
///
/// # Errors
///
/// Returns [`MnsError::Parse`] on malformed XML, [`MnsError::Attr`] on undecodable attribute
/// values, [`MnsError::UnknownEventType`] if the `type` attribute is not a defined MAP 1.4 event
/// type, or [`MnsError::MissingEvent`] if no `<event>` element is found.
pub fn parse_event_report(xml: &[u8]) -> Result<MnsEvent, MnsError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::with_capacity(256);
    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Empty(e) | Event::Start(e) if e.name().as_ref() == b"event" => {
                let mut raw_type: Option<String> = None;
                let mut handle = None;
                let mut folder = None;
                let mut old_folder = None;
                let mut msg_type = None;
                let mut datetime = None;
                for attr in e.attributes() {
                    let a = attr?;
                    let val = a.normalized_value(XmlVersion::Implicit1_0)?.into_owned();
                    match a.key.as_ref() {
                        b"type" => raw_type = Some(val),
                        b"handle" => handle = Some(val),
                        b"folder" => folder = Some(val),
                        b"old_folder" => old_folder = Some(val),
                        b"msg_type" => msg_type = Some(val),
                        b"datetime" => datetime = Some(val),
                        _ => {}
                    }
                }
                let event_type = match raw_type {
                    Some(ref s) => event_type_from_str(s)?,
                    None => return Err(MnsError::MissingEventType),
                };
                return Ok(MnsEvent { event_type, handle, folder, old_folder, msg_type, datetime });
            }
            Event::Eof => return Err(MnsError::MissingEvent),
            _ => {}
        }
        buf.clear();
    }
}

fn event_type_from_str(s: &str) -> Result<EventType, MnsError> {
    match s {
        "NewMessage" => Ok(EventType::NewMessage),
        "DeliverySuccess" => Ok(EventType::DeliverySuccess),
        "SendingSuccess" => Ok(EventType::SendingSuccess),
        "DeliveryFailure" => Ok(EventType::DeliveryFailure),
        "SendingFailure" => Ok(EventType::SendingFailure),
        "MessageDeleted" => Ok(EventType::MessageDeleted),
        "MessageShift" => Ok(EventType::MessageShift),
        "MemoryFull" => Ok(EventType::MemoryFull),
        "MemoryAvailable" => Ok(EventType::MemoryAvailable),
        "ReadStatusChanged" => Ok(EventType::ReadStatusChanged),
        other => Err(MnsError::UnknownEventType(other.to_owned())),
    }
}
