//! Unit tests for MAP-event-report XML parsing.

use imsg_map::mns_event::{parse_event_report, EventType, MnsError};

const MNS_NEW_MESSAGE_XML: &[u8] = br#"<?xml version="1.0"?>
<MAP-event-report version="1.0">
<event type="NewMessage" handle="20000" folder="TELECOM/MSG/INBOX" msg_type="SMS_GSM" datetime="20260604T120000"/>
</MAP-event-report>"#;

const MNS_MESSAGE_SHIFT_XML: &[u8] = br#"<?xml version="1.0"?>
<MAP-event-report version="1.0">
<event type="MessageShift" handle="20001" folder="TELECOM/MSG/SENT" old_folder="TELECOM/MSG/OUTBOX" msg_type="SMS_GSM"/>
</MAP-event-report>"#;

const MNS_MEMORY_FULL_XML: &[u8] = br#"<?xml version="1.0"?>
<MAP-event-report version="1.0">
<event type="MemoryFull"/>
</MAP-event-report>"#;

const MNS_READ_STATUS_CHANGED_XML: &[u8] = br#"<?xml version="1.0"?>
<MAP-event-report version="1.0">
<event type="ReadStatusChanged" handle="20002" folder="TELECOM/MSG/INBOX" msg_type="SMS_GSM"/>
</MAP-event-report>"#;

const MNS_UNKNOWN_TYPE_XML: &[u8] = br#"<?xml version="1.0"?>
<MAP-event-report version="1.0">
<event type="NewFax" handle="30000" folder="TELECOM/MSG/INBOX"/>
</MAP-event-report>"#;

const MNS_NO_EVENT_XML: &[u8] = br#"<?xml version="1.0"?>
<MAP-event-report version="1.0">
</MAP-event-report>"#;

#[test]
fn mns_new_message_event() -> Result<(), MnsError> {
    let ev = parse_event_report(MNS_NEW_MESSAGE_XML)?;
    assert_eq!(ev.event_type(), EventType::NewMessage);
    assert_eq!(ev.handle(), Some("20000"));
    assert_eq!(ev.folder(), Some("TELECOM/MSG/INBOX"));
    assert_eq!(ev.msg_type(), Some("SMS_GSM"));
    assert_eq!(ev.datetime(), Some("20260604T120000"));
    assert!(ev.old_folder().is_none());
    Ok(())
}

#[test]
fn mns_message_shift_event() -> Result<(), MnsError> {
    let ev = parse_event_report(MNS_MESSAGE_SHIFT_XML)?;
    assert_eq!(ev.event_type(), EventType::MessageShift);
    assert_eq!(ev.handle(), Some("20001"));
    assert_eq!(ev.folder(), Some("TELECOM/MSG/SENT"));
    assert_eq!(ev.old_folder(), Some("TELECOM/MSG/OUTBOX"));
    assert_eq!(ev.msg_type(), Some("SMS_GSM"));
    Ok(())
}

#[test]
fn mns_memory_full_event() -> Result<(), MnsError> {
    let ev = parse_event_report(MNS_MEMORY_FULL_XML)?;
    assert_eq!(ev.event_type(), EventType::MemoryFull);
    assert!(ev.handle().is_none());
    assert!(ev.folder().is_none());
    assert!(ev.msg_type().is_none());
    assert!(ev.datetime().is_none());
    Ok(())
}

#[test]
fn mns_read_status_changed_event() -> Result<(), MnsError> {
    let ev = parse_event_report(MNS_READ_STATUS_CHANGED_XML)?;
    assert_eq!(ev.event_type(), EventType::ReadStatusChanged);
    assert_eq!(ev.handle(), Some("20002"));
    assert_eq!(ev.folder(), Some("TELECOM/MSG/INBOX"));
    assert_eq!(ev.msg_type(), Some("SMS_GSM"));
    Ok(())
}

#[test]
fn mns_unknown_event_type_returns_error() {
    let result = parse_event_report(MNS_UNKNOWN_TYPE_XML);
    assert!(matches!(result, Err(MnsError::UnknownEventType(_))));
}

#[test]
fn mns_missing_event_element_returns_error() {
    let result = parse_event_report(MNS_NO_EVENT_XML);
    assert!(matches!(result, Err(MnsError::MissingEvent)));
}
