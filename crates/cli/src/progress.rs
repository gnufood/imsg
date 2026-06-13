//! Top-level progress UI: a spinner shown during network-bound commands.

use std::future::Future;
use std::time::Duration;

use indicatif::ProgressBar;

/// The spinner is auto-hidden when stderr is not a terminal. Returns `fut`'s output
/// unchanged.
pub(crate) async fn with_spinner<T>(label: &str, fut: impl Future<Output = T>) -> T {
    let pb = ProgressBar::new_spinner();
    pb.set_message(format!("{label}…"));
    pb.enable_steady_tick(Duration::from_millis(100));
    let out = fut.await;
    pb.finish_and_clear();
    out
}
