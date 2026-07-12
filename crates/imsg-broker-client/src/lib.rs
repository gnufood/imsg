//! Shared IPC client for the imsg broker/daemon abstract socket.
//!
//! Transport, reachability, and response-interpretation primitives reused by every
//! consumer of the broker protocol (`imsg-ipc`) — the CLI today, the GUI eventually.
//! Presentation formatting (human-readable display strings) and process-spawn mechanics
//! (the ephemeral broker's `current_exe()` self-spawn) stay in each binary; this crate
//! only returns structured data.

// `pub mod` (not `mod`) is required: `transport` has `pub(crate)` items, and `pub(crate)`
// in a private module trips `redundant_pub_crate` while `pub` trips `unreachable_pub`.
pub mod probe;
pub mod query;
pub mod response;
pub mod transport;
pub mod write;

pub use probe::{connect_retry, probe};
pub use query::{query_persistent, query_state};
pub use response::{text_result, CallError};
pub use transport::send_request;
pub use write::{delete, send, WriteError};
