//! Shared fake-PBAP-server harness for `sync_contacts` tests, driven over a `tokio::io::duplex`
//! pair against a real (temp-dir) `Store`.

mod meta;
mod refresh;

use bytes::Bytes;
use obex_core::headers::Header;
use obex_core::packet::{OpCode, Packet, PacketExtra};
use secrecy::SecretBox;
use store::Store;

use super::{hex16, sync_contacts};

const CONNECT_RSP: &[u8] = &[
    0xa0, 0x00, 0x1f, 0x10, 0x00, 0x0f, 0xa0, 0xcb, 0xdd, 0x20, 0x40, 0xd0, 0x4a, 0x00, 0x13, 0x79,
    0x61, 0x35, 0xf0, 0xf0, 0xc5, 0x11, 0xd8, 0x09, 0x66, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

const A: [u8; 16] = [0xAA; 16];
const B: [u8; 16] = [0xBB; 16];
const C: [u8; 16] = [0xCC; 16];
const D: [u8; 16] = [0xDD; 16];

async fn fake_store() -> anyhow::Result<(Store, tempfile::TempDir)> {
    let dir = tempfile::tempdir()?;
    let key: SecretBox<[u8; 32]> = SecretBox::new(Box::new([0u8; 32]));
    let s = Store::open(dir.path().join("test.db"), key).await?;
    Ok((s, dir))
}

fn tlv(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = vec![tag, u8::try_from(value.len()).unwrap_or(u8::MAX)];
    out.extend_from_slice(value);
    out
}

fn metadata_rsp(db_id: [u8; 16], primary: [u8; 16], secondary: [u8; 16]) -> anyhow::Result<Bytes> {
    let mut app_params = tlv(0x0D, &db_id);
    app_params.extend(tlv(0x0A, &primary));
    app_params.extend(tlv(0x0B, &secondary));
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::AppParams(Bytes::from(app_params))],
    }
    .encode()?)
}

fn body_rsp(body: &[u8]) -> anyhow::Result<Bytes> {
    Ok(Packet {
        opcode: OpCode::Ok,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()?)
}

fn list_body(handles: &[&str]) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut body = String::from("<vCard-listing>");
    for h in handles {
        let _ = write!(body, "<card handle=\"{h}\" name=\"x\"/>");
    }
    body.push_str("</vCard-listing>");
    body.into_bytes()
}

fn vcard(uid: Option<&str>, name: &str, tel: &str) -> Vec<u8> {
    let uid_line = uid.map(|u| format!("UID:{u}\r\n")).unwrap_or_default();
    format!("BEGIN:VCARD\r\nVERSION:3.0\r\nFN:{name}\r\n{uid_line}TEL:{tel}\r\nEND:VCARD\r\n")
        .into_bytes()
}
