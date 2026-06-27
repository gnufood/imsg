//! Display formatting helpers shared across subcommand renderers.
//!
//! MAP datetime string conversion (`fmt_datetime`) was removed when `list` and `threads`
//! became store-backed: they now receive `timestamp_ms: i64` and call
//! [`session::sync::ms_to_display`] directly.
