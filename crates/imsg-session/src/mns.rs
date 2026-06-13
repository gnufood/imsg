//! MNS accept loop — drives the MAP Message Notification Service server-side session.

use map_core::mns_server::MnsServer;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{mpsc, watch};
use transport::rfcomm::ProfileListener;

use crate::MnsEvent;

/// Perform OBEX MNS handshake, forward events to `event_tx`.
pub async fn run_mns_session(
    listener: ProfileListener,
    event_tx: mpsc::Sender<MnsEvent>,
    cancel: watch::Receiver<bool>,
) {
    let stream_source = listener_stream(listener, cancel.clone());
    run_mns_loop(Box::pin(stream_source), event_tx, cancel).await;
}

/// Stream of RFCOMM connections, honours `cancel`.
pub(crate) fn listener_stream(
    listener: ProfileListener,
    cancel: watch::Receiver<bool>,
) -> impl futures::Stream<Item = bluer::rfcomm::Stream> {
    futures::stream::unfold((listener, cancel), |(mut l, mut c)| async move {
        loop {
            if *c.borrow() {
                return None;
            }
            let req = tokio::select! {
                biased;
                result = c.changed() => {
                    if result.is_err() || *c.borrow_and_update() { return None; }
                    continue;
                }
                r = l.next() => match r {
                    Some(r) => r,
                    None => return None,
                }
            };
            match req.accept() {
                Ok(stream) => return Some((stream, (l, c))),
                Err(e) => tracing::warn!("MNS rfcomm accept failed: {e}"),
            }
        }
    })
}

/// Inner accept-and-serve loop. `stream_source` yields already-accepted transport streams.
pub(crate) async fn run_mns_loop<S, T>(
    stream_source: S,
    event_tx: mpsc::Sender<MnsEvent>,
    cancel: watch::Receiver<bool>,
) where
    S: futures::Stream<Item = T> + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
{
    crate::loop_util::run_accept_loop(stream_source, cancel, move |stream, _cancel| {
        let event_tx = event_tx.clone();
        async move {
            match MnsServer::accept(stream).await {
                Ok(mns) => drain_events(mns, &event_tx).await,
                Err(e) => {
                    tracing::warn!("MNS accept failed: {e}");
                    true
                }
            }
        }
    })
    .await;
}

/// Forward every event from one MNS connection to `event_tx`.
/// Returns `false` when `event_tx` closed (caller must stop), `true` when connection ended
/// (caller may accept next).
async fn drain_events<T>(mut mns: MnsServer<T>, event_tx: &mpsc::Sender<MnsEvent>) -> bool
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    loop {
        match mns.next_event().await {
            Ok(Some(e)) => {
                if event_tx.send(e).await.is_err() {
                    return false;
                }
            }
            Ok(None) => return true,
            Err(e) => {
                tracing::warn!("MNS event error: {e}");
                return true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{iphone_connect, iphone_disconnect, iphone_put, NEW_MESSAGE_XML};
    use futures::stream;
    use map_core::mns_event::{EventType, MnsError};
    use obex_core::wrap;
    use tokio::io::{duplex, DuplexStream};

    #[tokio::test]
    async fn exits_when_listener_ends() {
        let (tx, _rx) = mpsc::channel(1);
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        run_mns_loop(stream::empty::<DuplexStream>(), tx, cancel_rx).await;
    }

    #[tokio::test]
    async fn exits_on_cancel_before_accept() {
        let (tx, _rx) = mpsc::channel(1);
        let (_cancel_tx, cancel_rx) = watch::channel(true);
        run_mns_loop(stream::pending::<DuplexStream>(), tx, cancel_rx).await;
    }

    #[tokio::test]
    async fn event_forwarded_to_channel() -> Result<(), MnsError> {
        let (event_tx, mut event_rx) = mpsc::channel(4);
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (server_io, iphone_io) = duplex(4096);

        let ((), iphone_result) =
            futures::join!(run_mns_loop(stream::iter([server_io]), event_tx, cancel_rx), async {
                let mut t = wrap(iphone_io);
                iphone_connect(&mut t).await?;
                iphone_put(&mut t, NEW_MESSAGE_XML).await?;
                iphone_disconnect(&mut t).await?;
                Ok::<_, MnsError>(())
            });
        iphone_result?;

        let event = event_rx.recv().await.ok_or(MnsError::UnexpectedEof)?;
        assert_eq!(event.event_type(), EventType::NewMessage);
        assert_eq!(event.handle(), Some("ABC123"));
        assert_eq!(event.folder(), Some("TELECOM/MSG/INBOX"));
        Ok(())
    }

    #[tokio::test]
    async fn reconnects_on_second_connection() -> Result<(), MnsError> {
        let (event_tx, mut event_rx) = mpsc::channel(4);
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (s1_server, s1_iphone) = duplex(4096);
        let (s2_server, s2_iphone) = duplex(4096);

        let ((), iphone_result) = futures::join!(
            run_mns_loop(stream::iter([s1_server, s2_server]), event_tx, cancel_rx),
            async {
                let mut t1 = wrap(s1_iphone);
                iphone_connect(&mut t1).await?;
                iphone_put(&mut t1, NEW_MESSAGE_XML).await?;
                iphone_disconnect(&mut t1).await?;

                let mut t2 = wrap(s2_iphone);
                iphone_connect(&mut t2).await?;
                iphone_put(&mut t2, NEW_MESSAGE_XML).await?;
                iphone_disconnect(&mut t2).await?;
                Ok::<_, MnsError>(())
            }
        );
        iphone_result?;

        let e1 = event_rx.recv().await.ok_or(MnsError::UnexpectedEof)?;
        let e2 = event_rx.recv().await.ok_or(MnsError::UnexpectedEof)?;
        assert_eq!(e1.event_type(), EventType::NewMessage);
        assert_eq!(e2.event_type(), EventType::NewMessage);
        Ok(())
    }
}
