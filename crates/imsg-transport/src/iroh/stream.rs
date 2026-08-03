use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use iroh::endpoint::VarInt;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use super::{Connection, RecvStream, SpokeStream};

/// Hub-side OBEX stream that keeps its [`Connection`] alive for the session duration.
///
/// Wraps [`SpokeStream`] and owns the underlying [`Connection`] as a drop guard. Dropping
/// `Connection` tears down all QUIC streams through it immediately — this wrapper prevents
/// that until the OBEX session completes.
pub struct HubStream {
    inner: SpokeStream,
    conn: Connection,
}

impl HubStream {
    /// Pairs `inner` with `conn` so both are dropped together.
    #[must_use]
    pub const fn new(inner: SpokeStream, conn: Connection) -> Self {
        Self { inner, conn }
    }
}

impl Drop for HubStream {
    fn drop(&mut self) {
        self.conn.close(VarInt::from_u32(0), b"");
    }
}

impl AsyncRead for HubStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for HubStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// MNS subscription stream that keeps its [`Connection`] alive for the session duration.
///
/// Wraps [`RecvStream`] and owns the underlying [`Connection`] as a drop guard. Dropping
/// `Connection` tears down all QUIC streams through it immediately — this wrapper prevents
/// that until the event loop completes. Both `RecvStream` and `Connection` are [`Unpin`], so
/// this type is [`Unpin`] and can be passed directly to [`tokio::io::AsyncReadExt`] methods.
pub struct HubRecvStream {
    inner: RecvStream,
    conn: Connection,
}

impl HubRecvStream {
    /// Pairs `inner` with `conn` so both are dropped together.
    #[must_use]
    pub const fn new(inner: RecvStream, conn: Connection) -> Self {
        Self { inner, conn }
    }
}

impl Drop for HubRecvStream {
    fn drop(&mut self) {
        self.conn.close(VarInt::from_u32(0), b"");
    }
}

impl AsyncRead for HubRecvStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}
