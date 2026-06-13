//! Integration tests for the MNS OBEX server.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use imsg_map::{
    mns_event::{EventType, MnsError},
    mns_server::MnsServer,
    ObexError,
};
use obex_core::wrap;
use obex_core::{
    client::ObexClient,
    headers::Header,
    packet::{OpCode, Packet, PacketExtra},
};
use tokio::io::duplex;

const MNS_TARGET: [u8; 16] = [
    0xbb, 0x58, 0x2b, 0x41, 0x42, 0x0c, 0x11, 0xdb, 0xb0, 0xde, 0x08, 0x00, 0x20, 0x0c, 0x9a, 0x66,
];

const NEW_MESSAGE_XML: &[u8] = b"<?xml version='1.0'?>\
<MAP-event-report version='1.0'>\
<event type='NewMessage' handle='ABC123' folder='TELECOM/MSG/INBOX' msg_type='SMS_GSM'/>\
</MAP-event-report>";

/// Sends a CONNECT request and reads back the server's CONNECT OK response.
async fn iphone_connect(
    t: &mut obex_core::ObexTransport<tokio::io::DuplexStream>,
) -> Result<(), MnsError> {
    let req = ObexClient::connect_request(&MNS_TARGET).map_err(MnsError::Obex)?;
    t.send(req).await?;
    t.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)?;
    Ok(())
}

/// Sends a `PUT_FINAL` with the given body and reads back the `RSP_OK`.
async fn iphone_put(
    t: &mut obex_core::ObexTransport<tokio::io::DuplexStream>,
    body: &[u8],
) -> Result<(), MnsError> {
    let pkt = Packet {
        opcode: OpCode::PutFinal,
        extra: PacketExtra::None,
        headers: vec![Header::EndOfBody(Bytes::copy_from_slice(body))],
    }
    .encode()
    .map_err(ObexError::Packet)
    .map_err(MnsError::Obex)?;
    t.send(pkt).await?;
    t.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)?;
    Ok(())
}

/// Sends an OBEX DISCONNECT and reads back the `RSP_OK`.
async fn iphone_disconnect(
    t: &mut obex_core::ObexTransport<tokio::io::DuplexStream>,
) -> Result<(), MnsError> {
    let pkt = Packet { opcode: OpCode::Disconnect, extra: PacketExtra::None, headers: vec![] }
        .encode()
        .map_err(ObexError::Packet)
        .map_err(MnsError::Obex)?;
    t.send(pkt).await?;
    t.next().await.ok_or(MnsError::UnexpectedEof)?.map_err(MnsError::Transport)?;
    Ok(())
}

#[tokio::test]
async fn test_mns_accept_connect() -> Result<(), MnsError> {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, iphone_result) =
        futures::join!(async { MnsServer::accept(server_io).await.map(|_| ()) }, async {
            let mut t = wrap(iphone_io);
            iphone_connect(&mut t).await
        });
    iphone_result?;
    server_result
}

#[tokio::test]
async fn test_mns_next_event_new_message() -> Result<(), MnsError> {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, iphone_result) = futures::join!(
        async {
            let mut mns = MnsServer::accept(server_io).await?;
            mns.next_event().await
        },
        async {
            let mut t = wrap(iphone_io);
            iphone_connect(&mut t).await?;
            iphone_put(&mut t, NEW_MESSAGE_XML).await
        }
    );
    iphone_result?;
    let event = server_result?.ok_or(MnsError::MissingEvent)?;
    assert_eq!(event.event_type(), EventType::NewMessage);
    assert_eq!(event.handle(), Some("ABC123"));
    assert_eq!(event.folder(), Some("TELECOM/MSG/INBOX"));
    assert_eq!(event.msg_type(), Some("SMS_GSM"));
    Ok(())
}

#[tokio::test]
async fn test_mns_next_event_disconnect() -> Result<(), MnsError> {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, iphone_result) = futures::join!(
        async {
            let mut mns = MnsServer::accept(server_io).await?;
            mns.next_event().await
        },
        async {
            let mut t = wrap(iphone_io);
            iphone_connect(&mut t).await?;
            iphone_disconnect(&mut t).await
        }
    );
    iphone_result?;
    assert_eq!(server_result?, None);
    Ok(())
}

#[tokio::test]
async fn test_mns_two_events() -> Result<(), MnsError> {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, iphone_result) = futures::join!(
        async {
            let mut mns = MnsServer::accept(server_io).await?;
            let e1 = mns.next_event().await?;
            let e2 = mns.next_event().await?;
            Ok::<_, MnsError>((e1, e2))
        },
        async {
            let mut t = wrap(iphone_io);
            iphone_connect(&mut t).await?;
            iphone_put(&mut t, NEW_MESSAGE_XML).await?;
            iphone_put(&mut t, NEW_MESSAGE_XML).await
        }
    );
    iphone_result?;
    let (e1, e2) = server_result?;
    assert_eq!(e1.map(|e| e.event_type()), Some(EventType::NewMessage));
    assert_eq!(e2.map(|e| e.event_type()), Some(EventType::NewMessage));
    Ok(())
}

#[tokio::test]
async fn test_mns_invalid_target() {
    let (server_io, iphone_io) = duplex(4096);
    let wrong_target = [0u8; 16];
    let (server_result, ()) = futures::join!(MnsServer::accept(server_io), async {
        let mut t = wrap(iphone_io);
        if let Ok(req) = ObexClient::connect_request(&wrong_target) {
            let _ = t.send(req).await;
        }
    });
    assert!(matches!(server_result, Err(MnsError::InvalidTarget)));
}

#[tokio::test]
async fn test_mns_next_event_raw_returns_body() -> Result<(), MnsError> {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, iphone_result) = futures::join!(
        async {
            let mut mns = MnsServer::accept(server_io).await?;
            mns.next_event_raw().await
        },
        async {
            let mut t = wrap(iphone_io);
            iphone_connect(&mut t).await?;
            iphone_put(&mut t, NEW_MESSAGE_XML).await
        }
    );
    iphone_result?;
    let body = server_result?.ok_or(MnsError::MissingEvent)?;
    assert_eq!(body.as_ref(), NEW_MESSAGE_XML);
    Ok(())
}

#[tokio::test]
async fn test_mns_next_event_raw_disconnect() -> Result<(), MnsError> {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, iphone_result) = futures::join!(
        async {
            let mut mns = MnsServer::accept(server_io).await?;
            mns.next_event_raw().await
        },
        async {
            let mut t = wrap(iphone_io);
            iphone_connect(&mut t).await?;
            iphone_disconnect(&mut t).await
        }
    );
    iphone_result?;
    assert_eq!(server_result?, None);
    Ok(())
}

#[tokio::test]
async fn test_mns_unexpected_opcode() {
    let (server_io, iphone_io) = duplex(4096);
    let (server_result, ()) = futures::join!(
        async {
            let mut mns = MnsServer::accept(server_io).await?;
            mns.next_event().await
        },
        async {
            let mut t = wrap(iphone_io);
            if let Ok(req) = ObexClient::connect_request(&MNS_TARGET) {
                if t.send(req).await.is_ok() {
                    let _ = t.next().await;
                    let get = Packet {
                        opcode: OpCode::GetFinal,
                        extra: PacketExtra::None,
                        headers: vec![],
                    };
                    if let Ok(b) = get.encode() {
                        let _ = t.send(b).await;
                    }
                }
            }
        }
    );
    assert!(matches!(server_result, Err(MnsError::UnexpectedOpcode(_))));
}
