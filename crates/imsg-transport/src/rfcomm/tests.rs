use std::io;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use socket2::{Domain, Socket, Type};

use super::await_bt_connected;
use crate::TransportError;

// --- OS-invariant tests ---
// Prove the kernel behaviour await_bt_connected relies on.
// Use standard sockets — no RFCOMM hardware needed.
//
// `connect`'s `security` branch (Socket::new/set_security/connect) needs a real RFCOMM
// socket and is untested here, matching this crate's existing convention for
// hardware-backed calls (see discover.rs's note on `connect`/`listen_mns`).

// mirrors the BT_CONNECT state — if this ever breaks, peer_addr().is_ok() loses its meaning
#[test]
fn unconnected_socket_peer_addr_returns_enotconn() -> io::Result<()> {
    let sock = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    let result = sock.peer_addr();
    assert!(
        matches!(&result, Err(e) if e.kind() == io::ErrorKind::NotConnected),
        "expected ENOTCONN on unconnected socket, got: {result:?}"
    );
    Ok(())
}

// mirrors BT_CONNECTED — proves peer_addr().is_ok() is the correct loop-exit condition
#[test]
fn connected_socket_peer_addr_succeeds() -> io::Result<()> {
    use std::net::{TcpListener, TcpStream};
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let client = TcpStream::connect(addr)?;
    client.peer_addr()?;
    Ok(())
}

// --- Logic tests for await_bt_connected ---

#[tokio::test]
async fn resolves_immediately_when_already_connected() -> Result<(), TransportError> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    await_bt_connected(|| true, deadline).await
}

// 100ms deadline keeps the test fast
#[tokio::test]
async fn times_out_when_link_never_establishes() {
    let deadline = tokio::time::Instant::now() + Duration::from_millis(100);
    let result = await_bt_connected(|| false, deadline).await;
    assert!(
        matches!(&result, Err(TransportError::Io(e)) if e.kind() == io::ErrorKind::TimedOut),
        "expected TimedOut, got: {result:?}"
    );
}

#[tokio::test]
async fn resolves_after_transient_not_connected() -> Result<(), TransportError> {
    let polls = Arc::new(AtomicU32::new(0));
    let polls2 = polls.clone();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    await_bt_connected(
        move || {
            let n = polls2.fetch_add(1, Ordering::Relaxed);
            n >= 2 // false for polls 0 and 1, true from poll 2 onward
        },
        deadline,
    )
    .await?;
    assert!(polls.load(Ordering::Relaxed) >= 3);
    Ok(())
}
