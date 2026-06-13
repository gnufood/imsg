use thiserror::Error;

/// bMessage parse errors — missing required fields, unrecognised STATUS/TYPE values, and unterminated sections.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum BMessageError {
    /// A required top-level field was absent.
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    /// The STATUS value was neither `READ` nor `UNREAD`.
    #[error("unrecognised STATUS: {0}")]
    UnknownStatus(String),
    /// The TYPE value was not `SMS_GSM`.
    #[error("unrecognised TYPE: {0}")]
    UnknownType(String),
    /// A BEGIN:X was not followed by a matching END:X before EOF.
    #[error("unterminated section: {0}")]
    UnterminatedSection(&'static str),
}

/// Read/unread status from the bMessage STATUS field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageStatus {
    /// Wire value `READ`.
    Read,
    /// Wire value `UNREAD`.
    Unread,
}

impl MessageStatus {
    pub(super) const fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "READ",
            Self::Unread => "UNREAD",
        }
    }
}

/// Message type from the bMessage TYPE field.
///
/// iOS devices only report `SMS_GSM` (feature bitmask `0x02`). The underscore
/// form is required; iOS rejects `SMS-GSM`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    /// GSM SMS. Encoded as `SMS_GSM` — underscore, not hyphen.
    SmsGsm,
}

impl MessageType {
    pub(super) const fn as_str() -> &'static str {
        "SMS_GSM"
    }
}

/// Minimal vCard embedded in a bMessage originator or BENV section.
///
/// Only N and TEL are preserved; all other properties are discarded on parse.
/// Empty string means the property was absent or blank.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BVCard {
    /// Value of the N property.
    pub name: String,
    /// Value of the TEL property.
    pub tel: String,
}

/// BBODY section: transport metadata and decoded message text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BBody {
    /// ENCODING value, e.g. `8BIT`.
    pub encoding: String,
    /// CHARSET value, e.g. `UTF-8`.
    pub charset: String,
    /// LANGUAGE value, e.g. `UNKNOWN`.
    pub language: String,
    /// Text between BEGIN:MSG and END:MSG. CRLF normalised to LF on parse.
    pub text: String,
}

/// BENV section: recipients and the message body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BEnvelope {
    /// Recipient vCards from BEGIN:VCARD…END:VCARD blocks inside BENV.
    pub recipients: Vec<BVCard>,
    /// The BBODY section.
    pub body: BBody,
}

/// Single-level BENV structure; nested BENV not supported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BMessage {
    status: MessageStatus,
    type_: MessageType,
    folder: String,
    originator: Option<BVCard>,
    envelope: BEnvelope,
}

impl BMessage {
    /// Wire STATUS field: `READ` or `UNREAD`.
    #[must_use]
    #[inline]
    pub const fn status(&self) -> &MessageStatus {
        &self.status
    }

    /// FOLDER field value, e.g. `telecom/msg/outbox`.
    #[must_use]
    #[inline]
    pub fn folder(&self) -> &str {
        &self.folder
    }

    /// Sender vCard outside BENV; `None` if absent.
    #[must_use]
    #[inline]
    pub const fn originator(&self) -> Option<&BVCard> {
        self.originator.as_ref()
    }

    /// BENV section: recipient vCards and the message body.
    #[must_use]
    #[inline]
    pub const fn envelope(&self) -> &BEnvelope {
        &self.envelope
    }

    /// iOS-mandated structure for `PushMessage`: STATUS:UNREAD, `TYPE:SMS_GSM`, FOLDER:telecom/msg/outbox;
    /// empty originator outside BENV, recipient inside BENV.
    #[must_use]
    pub fn outbound_sms(phone: &str, text: &str) -> Self {
        Self {
            status: MessageStatus::Unread,
            type_: MessageType::SmsGsm,
            folder: "telecom/msg/outbox".to_owned(),
            originator: Some(BVCard::default()),
            envelope: BEnvelope {
                recipients: vec![BVCard { name: String::new(), tel: phone.to_owned() }],
                body: BBody {
                    encoding: "8BIT".to_owned(),
                    charset: "UTF-8".to_owned(),
                    language: "UNKNOWN".to_owned(),
                    text: text.to_owned(),
                },
            },
        }
    }

    pub(super) const fn from_parts(
        status: MessageStatus,
        type_: MessageType,
        folder: String,
        originator: Option<BVCard>,
        envelope: BEnvelope,
    ) -> Self {
        Self { status, type_, folder, originator, envelope }
    }
}
