//! Integration tests for the OBEX codec and client.

use bytes::Bytes;
use imsg_obex::{
    client::{ObexClient, ObexError},
    headers::Header,
    packet::{OpCode, Packet, PacketError, PacketExtra},
};
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use rstest::rstest;

const MAP_UUID: [u8; 16] = [
    0xbb, 0x58, 0x2b, 0x40, 0x42, 0x0c, 0x11, 0xdb, 0xb0, 0xde, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

const CONN_ID: u32 = 0xD0A0_6130;

fn connected_client() -> Result<ObexClient, ObexError> {
    let mut client = ObexClient::new();
    client.handle_connect_response(include_bytes!("fixtures/connect_rsp.bin"))?;
    Ok(client)
}

#[test]
fn encode_connect_request_matches_fixture() -> Result<(), ObexError> {
    let encoded = ObexClient::connect_request(&MAP_UUID)?;
    let fixture = include_bytes!("fixtures/connect_req.bin");
    assert_eq!(encoded.as_ref(), &fixture[..]);
    Ok(())
}

#[test]
fn decode_connect_request() -> Result<(), PacketError> {
    let fixture = include_bytes!("fixtures/connect_req.bin");
    let packet = Packet::decode(fixture)?;
    assert_eq!(packet.opcode, OpCode::Connect);
    assert_eq!(
        packet.extra,
        PacketExtra::Connect { version: 0x10, flags: 0x00, max_packet: 0xFFFF }
    );
    let target = packet.header_target().ok_or(PacketError::InvalidHeader)?;
    assert_eq!(target, &MAP_UUID);
    Ok(())
}

#[test]
fn decode_connect_response() -> Result<(), PacketError> {
    let fixture = include_bytes!("fixtures/connect_rsp.bin");
    let packet = Packet::decode_connect_response(fixture)?;
    assert_eq!(packet.opcode, OpCode::Ok);
    assert_eq!(packet.header_connection_id(), Some(CONN_ID));
    Ok(())
}

#[test]
fn handle_connect_response_transitions_state() -> Result<(), ObexError> {
    let mut client = ObexClient::new();
    assert!(!client.is_connected());
    let conn_id = client.handle_connect_response(include_bytes!("fixtures/connect_rsp.bin"))?;
    assert!(client.is_connected());
    assert_eq!(conn_id, CONN_ID);
    Ok(())
}

// ── SETPATH ───────────────────────────────────────────────────────────────────

#[rstest]
#[case("telecom", include_bytes!("fixtures/setpath_telecom_req.bin"))]
#[case("msg", include_bytes!("fixtures/setpath_msg_req.bin"))]
#[case("outbox", include_bytes!("fixtures/setpath_outbox_req.bin"))]
fn encode_setpath_matches_fixture(
    #[case] folder: &str,
    #[case] fixture: &[u8],
) -> Result<(), ObexError> {
    let encoded = connected_client()?.setpath_request(folder)?;
    assert_eq!(encoded.as_ref(), fixture);
    Ok(())
}

const SETPATH_BACKUP_REQ: &[u8] = include_bytes!("fixtures/setpath_backup_req.bin");

#[test]
fn encode_setpath_backup_matches_fixture() -> Result<(), ObexError> {
    let encoded = connected_client()?.setpath_backup_request()?;
    assert_eq!(encoded.as_ref(), SETPATH_BACKUP_REQ);
    Ok(())
}

#[test]
fn decode_setpath_telecom_request() -> Result<(), PacketError> {
    let fixture = include_bytes!("fixtures/setpath_telecom_req.bin");
    let packet = Packet::decode(fixture)?;
    assert_eq!(packet.opcode, OpCode::SetPath);
    assert_eq!(packet.extra, PacketExtra::SetPath { flags: 0x02, constants: 0x00 });
    assert_eq!(packet.header_connection_id(), Some(CONN_ID));
    let name = packet.headers.iter().find_map(|h| {
        if let Header::Name(s) = h {
            Some(s.as_str())
        } else {
            None
        }
    });
    assert_eq!(name, Some("telecom"));
    Ok(())
}

// ── GET ───────────────────────────────────────────────────────────────────────

#[test]
fn encode_get_folder_listing_matches_fixture() -> Result<(), ObexError> {
    let encoded = connected_client()?.get_request(b"x-obex/folder-listing\x00", None, None)?;
    let fixture = include_bytes!("fixtures/get_folder_listing_000_req.bin");
    assert_eq!(encoded.as_ref(), &fixture[..]);
    Ok(())
}

#[test]
fn decode_get_folder_listing_response_is_ok_with_body() -> Result<(), PacketError> {
    let fixture = include_bytes!("fixtures/get_folder_listing_000_rsp.bin");
    let packet = Packet::decode(fixture)?;
    assert!(packet.opcode.is_ok());
    let body = packet.body_payload().ok_or(PacketError::InvalidHeader)?;
    assert!(body.starts_with(b"<?xml"));
    let xml = std::str::from_utf8(body).map_err(|_| PacketError::InvalidHeader)?;
    assert!(xml.contains("inbox"));
    assert!(xml.contains("sent"));
    assert!(xml.contains("outbox"));
    assert!(xml.contains("deleted"));
    Ok(())
}

// ── DISCONNECT ────────────────────────────────────────────────────────────────

#[test]
fn encode_disconnect_matches_fixture() -> Result<(), ObexError> {
    let encoded = connected_client()?.disconnect_request()?;
    let fixture = include_bytes!("fixtures/disconnect_req.bin");
    assert_eq!(encoded.as_ref(), &fixture[..]);
    Ok(())
}

#[test]
fn decode_disconnect_response() -> Result<(), PacketError> {
    let fixture = include_bytes!("fixtures/disconnect_rsp.bin");
    let packet = Packet::decode(fixture)?;
    assert!(packet.opcode.is_ok());
    Ok(())
}

// ── PUT ───────────────────────────────────────────────────────────────────────

#[test]
fn encode_put_final_matches_push_message_fixture() -> Result<(), ObexError> {
    let fixture_req = include_bytes!("fixtures/put_push_message_req.bin");
    let packet = ObexClient::parse_response(fixture_req)?;
    let body = packet.body_payload().ok_or(ObexError::Packet(PacketError::InvalidHeader))?;
    let len = u32::try_from(body.len()).map_err(|_| ObexError::BodyTooLarge)?;
    let encoded = connected_client()?.put_final_request(
        b"x-bt/message\x00",
        vec![
            Header::Name(String::new()),
            Header::Length(len),
            Header::AppParams(Bytes::from_static(b"\x14\x01\x01")),
            Header::EndOfBody(Bytes::copy_from_slice(body)),
        ],
    )?;
    assert_eq!(encoded.as_ref(), &fixture_req[..]);
    Ok(())
}

#[test]
fn decode_push_message_response_contains_handle() -> Result<(), PacketError> {
    let fixture = include_bytes!("fixtures/put_push_message_rsp.bin");
    let packet = Packet::decode(fixture)?;
    assert!(packet.opcode.is_ok());
    let name = packet
        .headers
        .iter()
        .find_map(|h| if let Header::Name(s) = h { Some(s.clone()) } else { None })
        .ok_or(PacketError::InvalidHeader)?;
    assert!(!name.is_empty());
    Ok(())
}

// ── ERROR CASES ───────────────────────────────────────────────────────────────

#[test]
fn decode_empty_input_returns_error() {
    assert!(Packet::decode(&[]).is_err());
}

#[test]
fn decode_truncated_input_returns_error() {
    assert!(Packet::decode(&[0x80, 0x00]).is_err());
}

#[test]
fn setpath_without_connect_returns_not_connected() {
    assert!(ObexClient::new().setpath_request("telecom").is_err());
}

#[test]
fn disconnect_without_connect_returns_not_connected() {
    assert!(ObexClient::new().disconnect_request().is_err());
}

// ── ROUNDTRIP ─────────────────────────────────────────────────────────────────

fn arb_header() -> impl Strategy<Value = Header> {
    prop_oneof![
        any::<u32>().prop_map(Header::ConnectionId),
        any::<u32>().prop_map(Header::Length),
        "[a-zA-Z0-9 ]{0,20}".prop_map(Header::Name),
    ]
}

fn arb_packet() -> impl Strategy<Value = Packet> {
    (
        prop_oneof![
            Just(OpCode::GetFinal),
            Just(OpCode::Disconnect),
            Just(OpCode::Continue),
            Just(OpCode::Ok),
        ],
        proptest::collection::vec(arb_header(), 0..4),
    )
        .prop_map(|(opcode, headers)| Packet { opcode, extra: PacketExtra::None, headers })
}

// ── NEW ERROR CASES ───────────────────────────────────────────────────────────

#[test]
fn decode_connect_response_with_bad_headers_returns_error() {
    assert!(Packet::decode_connect_response(&[
        0xA0, 0x00, 0x0A, 0x10, 0x00, 0xFF, 0xFF, 0x01, 0x00, 0x01
    ])
    .is_err());
}

#[test]
fn handle_connect_response_missing_conn_id_returns_error() {
    let result =
        ObexClient::new().handle_connect_response(&[0xA0, 0x00, 0x07, 0x10, 0x00, 0xFF, 0xFF]);
    assert!(matches!(result, Err(ObexError::MissingConnectionId)));
}

#[rstest]
#[case(&[0x83, 0x00, 0x0A, 0x01, 0x00, 0x07, 0xD8, 0x00, 0x00, 0x00])]
#[case(&[0x83, 0x00, 0x09, 0x01, 0x00, 0x06, 0x00, 0x74, 0x00])]
#[case(&[0x83, 0x00, 0x06, 0x01, 0x00, 0x02])]
fn decode_name_header_errors(#[case] data: &[u8]) {
    assert!(Packet::decode(data).is_err());
}

proptest! {
    #[test]
    fn encode_decode_roundtrip(packet in arb_packet()) {
        let encoded = packet.encode()
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        let decoded = Packet::decode(&encoded)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        prop_assert_eq!(decoded, packet);
    }
}
