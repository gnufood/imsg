//! Shared datetime utilities: MAP timestamp parsing and epoch-millisecond display.

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

/// Converts a MAP basic-ISO datetime string (`YYYYMMDDTHHMMSS[±HHMM]`) to epoch milliseconds.
///
/// Parses only the first 15 characters; timezone suffix is ignored and the value is treated
/// as UTC. Returns `None` if the string is shorter than 15 bytes or fails to parse.
#[must_use]
pub fn datetime_to_ms(s: &str) -> Option<i64> {
    let truncated = s.get(..15)?;
    let naive = NaiveDateTime::parse_from_str(truncated, "%Y%m%dT%H%M%S").ok()?;
    Some(Utc.from_utc_datetime(&naive).timestamp_millis())
}

/// Formats epoch milliseconds as `YYYY-MM-DD HH:MM` in local time.
///
/// Returns `"?"` for timestamps outside the representable range.
#[must_use]
pub fn ms_to_display(ms: i64) -> String {
    DateTime::from_timestamp_millis(ms).map_or_else(
        || "?".to_owned(),
        |dt| dt.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string(),
    )
}

/// Current epoch milliseconds via `chrono::Utc`.
#[must_use]
pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}
