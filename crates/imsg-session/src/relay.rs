//! MNS relay loop — forward raw MAP event-report bytes from RFCOMM to a broadcast channel.
//!
//! The hub runs this beside the iroh hub: events pushed over RFCOMM are published verbatim
//! to every subscribed spoke. Parsing happens spoke-side — one malformed event degrades a
//! single spoke rather than the ingest.

use bytes::Bytes;
use map_core::mns_server::MnsServer;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{broadcast, watch};
use transport::rfcomm::ProfileListener;

use crate::mns::listener_stream;

/// Relay raw event bytes to `mns_tx`.
pub async fn run_mns_relay(
    listener: ProfileListener,
    mns_tx: broadcast::Sender<Bytes>,
    cancel: watch::Receiver<bool>,
) {
    let stream_source = listener_stream(listener, cancel.clone());
    relay_loop(Box::pin(stream_source), mns_tx, cancel).await;
}

/// Inner relay loop. `stream_source` yields already-accepted transport streams.
pub(crate) async fn relay_loop<S, T>(
    stream_source: S,
    mns_tx: broadcast::Sender<Bytes>,
    cancel: watch::Receiver<bool>,
) where
    S: futures::Stream<Item = T> + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
{
    crate::loop_util::run_accept_loop(stream_source, cancel, move |stream, mut cancel_clone| {
        let mns_tx = mns_tx.clone();
        async move { drain_to_broadcast(stream, &mns_tx, &mut cancel_clone).await }
    })
    .await;
}

/// Relay raw events from one MNS connection to `mns_tx`.
/// Returns `false` when cancel fires (stop), `true` when connection ends (accept next).
async fn drain_to_broadcast<T>(
    stream: T,
    mns_tx: &broadcast::Sender<Bytes>,
    cancel: &mut watch::Receiver<bool>,
) -> bool
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let mut mns = match MnsServer::accept(stream).await {
        Ok(mns) => mns,
        Err(e) => {
            tracing::warn!("MNS accept failed: {e}");
            return true;
        }
    };
    loop {
        tokio::select! {
            biased;
            result = cancel.changed() => {
                if result.is_err() || *cancel.borrow_and_update() { return false; }
            }
            event = mns.next_event_raw() => match event {
                Ok(Some(body)) => {
                    // No subscribers is normal — drop silently.
                    let _ = mns_tx.send(body);
                }
                Ok(None) => return true,
                Err(e) => {
                    tracing::warn!("MNS event error: {e}");
                    return true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{iphone_connect, iphone_disconnect, iphone_put, NEW_MESSAGE_XML};
    use futures::stream;
    use map_core::mns_event::MnsError;
    use obex_core::wrap;
    use tokio::io::{duplex, DuplexStream};

    #[tokio::test]
    async fn relay_broadcasts_raw_event() -> Result<(), MnsError> {
        let (mns_tx, mut mns_rx) = broadcast::channel::<Bytes>(8);
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (server_io, iphone_io) = duplex(4096);

        let ((), iphone_result) =
            futures::join!(relay_loop(stream::iter([server_io]), mns_tx, cancel_rx), async {
                let mut t = wrap(iphone_io);
                iphone_connect(&mut t).await?;
                iphone_put(&mut t, NEW_MESSAGE_XML).await?;
                iphone_disconnect(&mut t).await?;
                Ok::<_, MnsError>(())
            });
        iphone_result?;

        let body = mns_rx.try_recv().map_err(|_| MnsError::UnexpectedEof)?;
        assert_eq!(body.as_ref(), NEW_MESSAGE_XML);
        Ok(())
    }

    #[tokio::test]
    async fn relay_exits_on_cancel() {
        let (mns_tx, _rx) = broadcast::channel::<Bytes>(8);
        let (_cancel_tx, cancel_rx) = watch::channel(true);
        relay_loop(stream::pending::<DuplexStream>(), mns_tx, cancel_rx).await;
    }

    #[tokio::test]
    async fn relay_continues_after_disconnect() -> Result<(), MnsError> {
        let (mns_tx, mut mns_rx) = broadcast::channel::<Bytes>(8);
        let (_cancel_tx, cancel_rx) = watch::channel(false);
        let (s1_server, s1_iphone) = duplex(4096);
        let (s2_server, s2_iphone) = duplex(4096);

        let ((), iphone_result) = futures::join!(
            relay_loop(stream::iter([s1_server, s2_server]), mns_tx, cancel_rx),
            async {
                // First connection: connect then immediately disconnect, no event.
                let mut t1 = wrap(s1_iphone);
                iphone_connect(&mut t1).await?;
                iphone_disconnect(&mut t1).await?;

                // Second connection: a real event must still be relayed.
                let mut t2 = wrap(s2_iphone);
                iphone_connect(&mut t2).await?;
                iphone_put(&mut t2, NEW_MESSAGE_XML).await?;
                iphone_disconnect(&mut t2).await?;
                Ok::<_, MnsError>(())
            }
        );
        iphone_result?;

        let body = mns_rx.try_recv().map_err(|_| MnsError::UnexpectedEof)?;
        assert_eq!(body.as_ref(), NEW_MESSAGE_XML);
        Ok(())
    }
}
