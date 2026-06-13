//! Transport layer tests — no Bluetooth required.

use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use obex_core::{wrap, TransportError};

#[tokio::test]
async fn send_recv_connect_req() -> Result<(), TransportError> {
    let (a, b) = tokio::io::duplex(4096);
    let mut sender = wrap(a);
    let mut rx = wrap(b);

    let packet =
        Bytes::from_static(include_bytes!("../../imsg-obex/tests/fixtures/connect_req.bin"));
    sender.send(packet.clone()).await?;

    let received = rx.next().await.ok_or(TransportError::UnexpectedEof)??;
    assert_eq!(received, packet);
    Ok(())
}

#[tokio::test]
async fn send_recv_large_packet() -> Result<(), TransportError> {
    let (a, b) = tokio::io::duplex(8192);
    let mut sender = wrap(a);
    let mut rx = wrap(b);

    let packet = Bytes::from_static(include_bytes!(
        "../../imsg-obex/tests/fixtures/put_push_message_req.bin"
    ));
    sender.send(packet.clone()).await?;

    let received = rx.next().await.ok_or(TransportError::UnexpectedEof)??;
    assert_eq!(received, packet);
    Ok(())
}

#[tokio::test]
async fn fragmented_write_reassembled() -> Result<(), TransportError> {
    use tokio::io::AsyncWriteExt;

    let (mut raw, server_half) = tokio::io::duplex(4096);
    let mut rx = wrap(server_half);

    let fixture: &[u8] = include_bytes!("../../imsg-obex/tests/fixtures/connect_req.bin");
    let (first, second) = fixture.split_at(fixture.len() / 2);

    raw.write_all(first).await?;
    raw.write_all(second).await?;

    let received = rx.next().await.ok_or(TransportError::UnexpectedEof)??;
    assert_eq!(received.as_ref(), fixture);
    Ok(())
}

#[tokio::test]
async fn two_packets_arrive_in_order() -> Result<(), TransportError> {
    let (a, b) = tokio::io::duplex(4096);
    let mut sender = wrap(a);
    let mut rx = wrap(b);

    let req = Bytes::from_static(include_bytes!("../../imsg-obex/tests/fixtures/connect_req.bin"));
    let rsp = Bytes::from_static(include_bytes!("../../imsg-obex/tests/fixtures/connect_rsp.bin"));

    sender.send(req.clone()).await?;
    sender.send(rsp.clone()).await?;

    let first = rx.next().await.ok_or(TransportError::UnexpectedEof)??;
    let second = rx.next().await.ok_or(TransportError::UnexpectedEof)??;
    assert_eq!(first, req);
    assert_eq!(second, rsp);
    Ok(())
}

#[tokio::test]
async fn sender_drop_closes_rx() -> Result<(), TransportError> {
    let (a, b) = tokio::io::duplex(4096);
    let sender = wrap(a);
    let mut rx = wrap(b);

    drop(sender);

    let result = rx.next().await;
    assert!(result.is_none());
    Ok(())
}
