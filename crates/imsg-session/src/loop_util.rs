//! Shared accept-loop driver used by MNS session and relay modules.

use std::future::Future;

use futures::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::watch;

/// `drain` returns `false` to halt early, `true` to accept next.
pub(crate) async fn run_accept_loop<S, T, F, Fut>(
    mut stream_source: S,
    mut cancel: watch::Receiver<bool>,
    mut drain: F,
) where
    S: futures::Stream<Item = T> + Unpin,
    T: AsyncRead + AsyncWrite + Unpin,
    F: FnMut(T, watch::Receiver<bool>) -> Fut,
    Fut: Future<Output = bool>,
{
    loop {
        if *cancel.borrow() {
            return;
        }
        let stream = tokio::select! {
            biased;
            result = cancel.changed() => {
                if result.is_err() || *cancel.borrow_and_update() { return; }
                continue;
            }
            maybe = stream_source.next() => match maybe {
                Some(s) => s,
                None => return,
            }
        };
        if !drain(stream, cancel.clone()).await {
            return;
        }
    }
}
