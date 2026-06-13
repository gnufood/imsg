//! Display formatting helpers shared across subcommand renderers.

/// Converts a MAP basic-ISO datetime (`YYYYMMDDTHHMMSS`) to `YYYY-MM-DD HH:MM`.
///
/// Returns `s` unchanged if the string is shorter than 13 bytes, the separator
/// at position 8 is not `T`, or any expected digit positions contain non-ASCII-digit
/// bytes. Does not validate calendar correctness — only structural format.
pub(crate) fn fmt_datetime(s: &str) -> String {
    let Some(y) = s.get(0..4) else { return s.to_owned() };
    let Some(mo) = s.get(4..6) else { return s.to_owned() };
    let Some(d) = s.get(6..8) else { return s.to_owned() };
    let Some(h) = s.get(9..11) else { return s.to_owned() };
    let Some(mi) = s.get(11..13) else { return s.to_owned() };
    if s.get(8..9) != Some("T")
        || !y.bytes().all(|b| b.is_ascii_digit())
        || !mo.bytes().all(|b| b.is_ascii_digit())
        || !d.bytes().all(|b| b.is_ascii_digit())
        || !h.bytes().all(|b| b.is_ascii_digit())
        || !mi.bytes().all(|b| b.is_ascii_digit())
    {
        return s.to_owned();
    }
    format!("{y}-{mo}-{d} {h}:{mi}")
}
