//! imsg CLI — library surface backing the `imsg` binary, plus the doc/completion generators
//! under `examples/` that need `Cli::command()` from outside the binary crate.

// `pub mod` (not `mod`) is required: items inside are `pub(crate)`, and `pub(crate)`
// in a private module trips `redundant_pub_crate` while `pub` trips `unreachable_pub`.
pub mod cli;
pub mod commands;
pub mod fmt;
pub mod output;
pub mod progress;
