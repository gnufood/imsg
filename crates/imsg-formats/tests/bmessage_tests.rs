//! Integration tests for bMessage parsing and encoding.

use imsg_formats::bmessage::{BMessage, BMessageError, MessageStatus};

fn expected() -> BMessage {
    BMessage::outbound_sms("+15559876543", "Hello from Linux")
}

#[test]
fn encode_decode_roundtrip() -> Result<(), BMessageError> {
    let msg = expected();
    let encoded = msg.encode();
    let decoded = BMessage::parse(&encoded)?;
    assert_eq!(decoded, msg);
    Ok(())
}

#[test]
fn encode_uses_crlf_line_endings() {
    let encoded = expected().encode();
    assert!(encoded.contains("\r\n"));
}

#[test]
fn encode_type_field_uses_underscore() {
    let encoded = expected().encode();
    assert!(encoded.contains("TYPE:SMS_GSM"));
    assert!(!encoded.contains("SMS-GSM"));
}

#[test]
fn encode_length_matches_crlf_block() -> Result<(), BMessageError> {
    let encoded = expected().encode();
    let length_line = encoded
        .lines()
        .find(|l| l.starts_with("LENGTH:"))
        .ok_or(BMessageError::MissingField("LENGTH"))?;
    let declared: usize = length_line
        .trim_start_matches("LENGTH:")
        .parse()
        .map_err(|_| BMessageError::MissingField("LENGTH"))?;
    // "BEGIN:MSG\r\n" + "Hello from Linux\r\n" + "END:MSG\r\n" = 38
    assert_eq!(declared, 38);
    Ok(())
}

#[test]
fn encode_sender_before_benv() -> Result<(), BMessageError> {
    let encoded = expected().encode();
    let orig_pos = encoded.find("BEGIN:VCARD").ok_or(BMessageError::MissingField("VCARD"))?;
    let benv_pos = encoded.find("BEGIN:BENV").ok_or(BMessageError::MissingField("BENV"))?;
    assert!(orig_pos < benv_pos);
    Ok(())
}

#[test]
fn encode_recipient_inside_benv() -> Result<(), BMessageError> {
    let encoded = expected().encode();
    let benv_pos = encoded.find("BEGIN:BENV").ok_or(BMessageError::MissingField("BENV"))?;
    assert!(encoded.get(benv_pos..).is_some_and(|s| s.contains("+15559876543")));
    Ok(())
}

#[test]
fn parse_status_read() -> Result<(), BMessageError> {
    let src = "BEGIN:BMSG\r\nVERSION:1.0\r\nSTATUS:READ\r\nTYPE:SMS_GSM\r\nFOLDER:telecom/msg/inbox\r\nBEGIN:BENV\r\nBEGIN:BBODY\r\nENCODING:8BIT\r\nCHARSET:UTF-8\r\nLANGUAGE:UNKNOWN\r\nLENGTH:24\r\nBEGIN:MSG\r\nhi\r\nEND:MSG\r\nEND:BBODY\r\nEND:BENV\r\nEND:BMSG\r\n";
    let msg = BMessage::parse(src)?;
    assert_eq!(*msg.status(), MessageStatus::Read);
    Ok(())
}

#[test]
fn parse_missing_status_returns_error() {
    let src = "BEGIN:BMSG\r\nVERSION:1.0\r\nTYPE:SMS_GSM\r\nFOLDER:telecom/msg/inbox\r\nBEGIN:BENV\r\nBEGIN:BBODY\r\nENCODING:8BIT\r\nCHARSET:UTF-8\r\nLANGUAGE:UNKNOWN\r\nLENGTH:9\r\nBEGIN:MSG\r\nEND:MSG\r\nEND:BBODY\r\nEND:BENV\r\nEND:BMSG\r\n";
    assert!(matches!(BMessage::parse(src), Err(BMessageError::MissingField("STATUS"))));
}

#[test]
fn parse_unknown_type_returns_error() {
    let src = "BEGIN:BMSG\r\nVERSION:1.0\r\nSTATUS:UNREAD\r\nTYPE:SMS-GSM\r\nFOLDER:telecom/msg/inbox\r\nBEGIN:BENV\r\nBEGIN:BBODY\r\nENCODING:8BIT\r\nCHARSET:UTF-8\r\nLANGUAGE:UNKNOWN\r\nLENGTH:9\r\nBEGIN:MSG\r\nEND:MSG\r\nEND:BBODY\r\nEND:BENV\r\nEND:BMSG\r\n";
    assert!(matches!(BMessage::parse(src), Err(BMessageError::UnknownType(_))));
}

#[test]
fn parse_unterminated_returns_error() {
    assert!(matches!(
        BMessage::parse("BEGIN:BMSG\r\n"),
        Err(BMessageError::UnterminatedSection("BMSG"))
    ));
}

#[test]
fn parse_accepts_lf_line_endings() -> Result<(), BMessageError> {
    let lf = expected().encode().replace("\r\n", "\n");
    let msg = BMessage::parse(&lf)?;
    assert_eq!(*msg.status(), MessageStatus::Unread);
    Ok(())
}

#[test]
fn parse_body_containing_end_msg_sentinel() -> Result<(), BMessageError> {
    // LENGTH:41 = BEGIN:MSG\r\n(11) + END:MSG\r\n(9) + "actual end"\r\n(12) + END:MSG\r\n(9)
    let src ="BEGIN:BMSG\r\nVERSION:1.0\r\nSTATUS:UNREAD\r\nTYPE:SMS_GSM\r\nFOLDER:telecom/msg/inbox\r\nBEGIN:BENV\r\nBEGIN:BBODY\r\nENCODING:8BIT\r\nCHARSET:UTF-8\r\nLANGUAGE:UNKNOWN\r\nLENGTH:41\r\nBEGIN:MSG\r\nEND:MSG\r\nactual end\r\nEND:MSG\r\nEND:BBODY\r\nEND:BENV\r\nEND:BMSG\r\n";
    let msg = BMessage::parse(src)?;
    assert_eq!(msg.envelope().body.text, "END:MSG\nactual end");
    Ok(())
}
