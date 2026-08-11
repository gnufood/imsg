# bMessage Format Reference

The bMessage format is defined in the Bluetooth Message Access Profile (MAP) specification, Appendix B. It is used to exchange SMS messages between devices over Bluetooth, notably between iOS devices and external clients.

## Overview

bMessage is a text-based wire format with a hierarchical structure. It encodes message metadata (status, type, folder), sender information (vCard), recipients (vCard), and the message body (BBODY). All data is UTF-8 encoded text.

## Document Structure

A bMessage document consists of nested sections, each delimited by `BEGIN:` and `END:` tags:

```
BEGIN:BMSG
VERSION:1.0
STATUS:<status>
TYPE:<type>
FOLDER:<folder>
[BEGIN:VCARD ... END:VCARD]   // originator (optional)
BEGIN:BENV
    [BEGIN:VCARD ... END:VCARD]   // recipients (one or more)
    BEGIN:BBODY
        ENCODING:<encoding>
        CHARSET:<charset>
        LANGUAGE:<language>
        LENGTH:<byte-count>
        BEGIN:MSG
        <message-text>
        END:MSG
    END:BBODY
END:BENV
END:BMSG
```

## Top-Level Fields

| Field | Required | Description |
|-------|----------|-------------|
| `VERSION` | Yes | Must be `1.0`. Present in all valid documents. |
| `STATUS` | Yes | `READ` or `UNREAD`. Indicates the message read state. |
| `TYPE` | Yes | Message type. iOS devices always emit `SMS_GSM`. |
| `FOLDER` | Yes | Folder path, e.g., `telecom/msg/inbox`, `telecom/msg/outbox`. |
| `BEGIN:VCARD` | No | Originator vCard, placed before `BEGIN:BENV`. |

## STATUS Values

| Wire Value | Meaning |
|------------|---------|
| `READ` | Message has been read. |
| `UNREAD` | Message is unread. |

The parser rejects any STATUS value other than these two, returning `BMessageError::UnknownStatus`.

## TYPE Values

| Wire Value | Meaning |
|------------|---------|
| `SMS_GSM` | GSM SMS message. The underscore form is required; iOS rejects `SMS-GSM`. |

The parser accepts only `SMS_GSM`, returning `BMessageError::UnknownType` for any other value.

## FOLDER Values

Common folder paths used by iOS:

| Folder Path | Direction |
|-------------|-----------|
| `telecom/msg/inbox` | Incoming messages |
| `telecom/msg/outbox` | Outgoing/queued messages |
| `telecom/msg/sent` | Sent messages |
| `telecom/msg/draft` | Draft messages |

## vCard Structure (BVCard)

Each vCard within a bMessage contains only two properties:

| Property | Description |
|----------|-------------|
| `N` | Display name or empty string. |
| `TEL` | Phone number in raw form. |

All other vCard properties are discarded on parse. The `BVCard` struct exposes `name: String` and `tel: String` fields.

## BENV Section

The BENV (Envelope) section contains:

1. **Recipients**: One or more vCard blocks. For outbound messages, the recipient appears inside BENV; the originator appears outside BMSG.
2. **BBODY**: The message body and its metadata.

## BBODY Section

| Field | Required | Description |
|-------|----------|-------------|
| `ENCODING` | Yes | Encoding type, e.g., `8BIT`. |
| `CHARSET` | Yes | Character set, e.g., `UTF-8`. |
| `LANGUAGE` | Yes | Language tag, e.g., `UNKNOWN`. |
| `LENGTH` | Yes | Byte count of the `BEGIN:MSG…END:MSG` block including CRLF line endings. |
| `BEGIN:MSG` | Yes | Delimits the message text. |
| `END:MSG` | Yes | Terminates the message text. |

## MSG Block

The content between `BEGIN:MSG` and `END:MSG` is the message body. Line endings within the MSG block are normalized to LF (`\n`) on parse. The parser uses the `LENGTH` field to determine how many bytes belong to the MSG block, allowing the text to contain the literal string `END:MSG` without prematurely terminating the block.

## Encoding Rules

### Line Endings

All lines in the wire format use CRLF (`\r\n`) line endings. The encoder produces CRLF throughout; the parser accepts both CRLF and LF.

### LENGTH Calculation

The `LENGTH` field in BBODY specifies the byte count of the CRLF-terminated `BEGIN:MSG…END:MSG` block. This includes:

- `BEGIN:MSG\r\n` (12 bytes)
- Each text line plus `\r\n`
- `END:MSG\r\n` (10 bytes)

The encoder computes this value by iterating over text lines and adding 2 bytes per line for the CRLF terminator. The parser validates the LENGTH field is present and uses it to correctly extract the message text even when it contains sentinel strings.

### TYPE Field Format

The TYPE field must use an underscore (`SMS_GSM`), not a hyphen (`SMS-GSM`). iOS rejects the hyphen form.

## Parsing

### Required Fields

The parser requires the following fields to be present:

- `STATUS`
- `TYPE`
- `FOLDER`
- `ENCODING`, `CHARSET`, `LANGUAGE`, `LENGTH` within BBODY
- `BEGIN:BENV` and `END:BENV`
- `BEGIN:BBODY` and `END:BBODY`
- `BEGIN:MSG` and `END:MSG`

Missing any required field returns `BMessageError::MissingField`.

### Section Termination

Each `BEGIN:X` must be matched by a corresponding `END:X` before EOF or before the next section of the same nesting level. Unterminated sections return `BMessageError::UnterminatedSection`.

### Line Ending Handling

The parser accepts both CRLF and LF line endings in input. However, the LENGTH field is always interpreted as a CRLF-based count per the MAP specification.

## Error Types

| Error | Condition |
|-------|-----------|
| `MissingField(&'static str)` | A required top-level field was absent. |
| `UnknownStatus(String)` | STATUS value was neither `READ` nor `UNREAD`. |
| `UnknownType(String)` | TYPE value was not `SMS_GSM`. |
| `UnterminatedSection(&'static str)` | A `BEGIN:X` was not followed by a matching `END:X` before EOF. |

## Data Types

### `MessageStatus`

```rust
pub enum MessageStatus {
    Read,   // Wire value: "READ"
    Unread, // Wire value: "UNREAD"
}
```

### `MessageType`

```rust
pub enum MessageType {
    SmsGsm, // Wire value: "SMS_GSM"
}
```

### `BVCard`

```rust
pub struct BVCard {
    pub name: String, // N property value
    pub tel: String,  // TEL property value
}
```

### `BBody`

```rust
pub struct BBody {
    pub encoding: String,  // ENCODING field
    pub charset: String,   // CHARSET field
    pub language: String,  // LANGUAGE field
    pub text: String,      // Message text (LF-normalized)
}
```

### `BEnvelope`

```rust
pub struct BEnvelope {
    pub recipients: Vec<BVCard>, // Recipient vCards
    pub body: BBody,             // Message body
}
```

### `BMessage`

```rust
pub struct BMessage {
    // Accessors:
    pub fn status(&self) -> &MessageStatus
    pub fn message_type(&self) -> &MessageType
    pub fn folder(&self) -> &str
    pub fn originator(&self) -> Option<&BVCard>
    pub fn envelope(&self) -> &BEnvelope
}
```

## Builder Method

`BMessage::outbound_sms(phone: &str, text: &str) -> Self` constructs an outbound SMS bMessage with iOS-mandated defaults:

- `STATUS:UNREAD`
- `TYPE:SMS_GSM`
- `FOLDER:telecom/msg/outbox`
- Empty originator vCard outside BENV
- Recipient vCard inside BENV
- `ENCODING:8BIT`, `CHARSET:UTF-8`, `LANGUAGE:UNKNOWN`

## Encoding Example

Input: `BMessage::outbound_sms("+15559876543", "Hello from Linux")`

Output:
```
BEGIN:BMSG
VERSION:1.0
STATUS:UNREAD
TYPE:SMS_GSM
FOLDER:telecom/msg/outbox
BEGIN:VCARD
VERSION:3.0
N:
TEL:+15559876543
END:VCARD
BEGIN:BENV
BEGIN:VCARD
VERSION:3.0
N:
TEL:+15559876543
END:VCARD
BEGIN:BBODY
ENCODING:8BIT
CHARSET:UTF-8
LANGUAGE:UNKNOWN
LENGTH:38
BEGIN:MSG
Hello from Linux
END:MSG
END:BBODY
END:BENV
END:BMSG
```

## See Also

- [Phone Number Normalization Reference](phone-number-normalization-reference.html) — E.164 normalization for addresses in bMessage
- [vCard Format Reference](vcard-format-reference.html) — Contact structure parsing
- [XML Folder Listing Reference](xml-folder-listing-reference.html) — MAP folder listing responses