//! Shared IPC client for the imsg broker/daemon abstract socket.
//!
//! Transport, reachability, and response-interpretation primitives reused by every
//! consumer of the broker protocol (`imsg-ipc`) — the CLI and the GUI. Presentation
//! formatting (human-readable display strings) stays in each binary. Process-spawn
//! mechanics (the ephemeral broker's/daemon's/headless GUI's `current_exe()` self-spawn)
//! don't belong here either — that's a different concern from talking to an already-running
//! broker over IPC — but they are shared, in `imsg-proc`.

// `pub mod` (not `mod`) is required: `transport` has `pub(crate)` items, and `pub(crate)`
// in a private module trips `redundant_pub_crate` while `pub` trips `unreachable_pub`.
pub mod contacts;
pub mod probe;
pub mod query;
pub mod read;
pub mod response;
pub mod transport;
pub mod write;

pub use contacts::{sync_contacts, ContactsError};
pub use probe::{connect_retry, probe};
pub use query::{query_persistent, query_state};
pub use read::{folders, ReadError};
pub use response::{folders_result, text_result, CallError};
pub use transport::send_request;
pub use write::{delete, send, sync, WriteError};
