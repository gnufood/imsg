//! Unit tests for live normalization, filtering, and aggregation helpers.

use std::collections::HashMap;

use map_core::messages::MessageEntry;
use map_core::BMessage;

use super::models::{Direction, LiveMessage};
use super::{accumulate, direction_of, keep, peer_address, to_live_body, window, ListFilter};
use crate::util::datetime_to_ms;

fn entry(sent: bool, read: bool, datetime: &str, sender: &str, recipient: &str) -> MessageEntry {
    MessageEntry {
        handle: "h".to_owned(),
        subject: String::new(),
        datetime: datetime.to_owned(),
        sender_name: String::new(),
        sender_addressing: sender.to_owned(),
        recipient_name: String::new(),
        recipient_addressing: recipient.to_owned(),
        msg_type: "SMS_GSM".to_owned(),
        size: 2,
        read,
        sent,
    }
}

fn msg(ts: i64, address: &str, read: bool) -> LiveMessage {
    LiveMessage {
        handle: "h".to_owned(),
        timestamp_ms: ts,
        address: address.to_owned(),
        folder: "inbox".to_owned(),
        read,
        text: "t".to_owned(),
    }
}

#[test]
fn direction_sent_for_sent_and_outbox_folders() {
    assert_eq!(direction_of("telecom/msg/sent"), Direction::Sent);
    assert_eq!(direction_of("telecom/msg/outbox"), Direction::Sent);
}

#[test]
fn direction_received_for_inbox_and_deleted() {
    assert_eq!(direction_of("telecom/msg/inbox"), Direction::Received);
    assert_eq!(direction_of("telecom/msg/deleted"), Direction::Received);
}

#[test]
fn keep_drops_read_when_unread_filter_set() {
    let f = ListFilter { unread: true, ..Default::default() };
    assert!(!keep(&msg(0, "a", true), &f));
    assert!(keep(&msg(0, "a", false), &f));
}

#[test]
fn keep_matches_from_address_exactly() {
    let f = ListFilter { from: Some("+1".to_owned()), ..Default::default() };
    assert!(keep(&msg(0, "+1", false), &f));
    assert!(!keep(&msg(0, "+2", false), &f));
}

#[test]
fn keep_drops_messages_before_since() {
    let f = ListFilter { since_ms: Some(100), ..Default::default() };
    assert!(!keep(&msg(99, "a", false), &f));
    assert!(keep(&msg(100, "a", false), &f));
}

#[test]
fn window_applies_offset_then_limit() -> anyhow::Result<()> {
    let msgs = vec![msg(3, "a", false), msg(2, "a", false), msg(1, "a", false)];
    let f = ListFilter { offset: 1, limit: Some(1), ..Default::default() };
    let out = window(msgs, &f);
    assert_eq!(out.len(), 1);
    let first = out.first().ok_or_else(|| anyhow::anyhow!("empty window"))?;
    assert_eq!(first.timestamp_ms, 2);
    Ok(())
}

#[test]
fn window_no_limit_keeps_rest_after_offset() {
    let msgs = vec![msg(2, "a", false), msg(1, "a", false)];
    let f = ListFilter { offset: 1, ..Default::default() };
    assert_eq!(window(msgs, &f).len(), 1);
}

#[test]
fn peer_address_resolves_sender_for_received_recipient_for_sent() {
    assert_eq!(peer_address(&entry(false, true, "", "+1", "+2")), "+1");
    assert_eq!(peer_address(&entry(true, true, "", "+1", "+2")), "+2");
}

#[test]
fn accumulate_groups_peer_counts_total_unread_and_latest() -> anyhow::Result<()> {
    let mut acc: HashMap<String, _> = HashMap::new();
    accumulate(&mut acc, &entry(false, false, "20260101T000000", "+1", ""));
    accumulate(&mut acc, &entry(true, true, "20260102T000000", "", "+1"));
    let t = acc.get("+1").ok_or_else(|| anyhow::anyhow!("missing thread"))?;
    assert_eq!(t.total, 2);
    assert_eq!(t.unread, 1);
    assert_eq!(Some(t.latest_ms), datetime_to_ms("20260102T000000"));
    Ok(())
}

#[test]
fn accumulate_skips_empty_address() {
    let mut acc = HashMap::new();
    accumulate(&mut acc, &entry(false, false, "20260101T000000", "", ""));
    assert!(acc.is_empty());
}

#[test]
fn to_live_body_sent_uses_recipient_and_outbox_direction() {
    let bmsg = BMessage::outbound_sms("+15550002", "hi");
    let body = to_live_body("hh".to_owned(), &bmsg);
    assert_eq!(body.handle, "hh");
    assert_eq!(body.direction, Direction::Sent);
    assert_eq!(body.address, "+15550002");
    assert!(!body.read);
    assert_eq!(body.text, "hi");
}

#[test]
fn to_live_body_received_uses_originator_and_inbox_direction() -> anyhow::Result<()> {
    // Transform a valid outbound encoding into a received one so LENGTH stays consistent.
    let wire = BMessage::outbound_sms("+15550002", "hi")
        .encode()
        .replace("FOLDER:telecom/msg/outbox", "FOLDER:telecom/msg/inbox")
        .replace("STATUS:UNREAD", "STATUS:READ")
        .replace("TEL:\r\n", "TEL:+15550001\r\n");
    let bmsg = BMessage::parse(&wire)?;
    let body = to_live_body("hh".to_owned(), &bmsg);
    assert_eq!(body.direction, Direction::Received);
    assert_eq!(body.address, "+15550001");
    assert!(body.read);
    assert_eq!(body.text, "hi");
    Ok(())
}
