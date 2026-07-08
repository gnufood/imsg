//! Shared IPC client for the imsg broker/daemon abstract socket.
//!
//! Pure transport and reachability primitives reused by every consumer of the broker
//! protocol (`imsg-ipc`) — the CLI today, the GUI eventually. Presentation formatting
//! and process-spawn mechanics (the ephemeral broker's `current_exe()` self-spawn) stay
//! in each binary; this crate only returns structured data.

// `pub mod` (not `mod`) is required: `transport` has `pub(crate)` items, and `pub(crate)`
// in a private module trips `redundant_pub_crate` while `pub` trips `unreachable_pub`.
pub mod probe;
pub mod query;
pub mod transport;

pub use probe::{connect_retry, probe};
pub use query::{query_persistent, query_state};
pub use transport::send_request;
