//! Phone-number normalisation shared by PBAP contact and MAP message address handling.
//!
//! Backed by `phonenumber`'s regional numbering-plan metadata rather than punctuation
//! stripping — national trunk prefixes (e.g. UK's parenthesised `(0)`) vary by region and
//! sometimes by number type within a region, so there is no single fixed rule to hand-roll.

use phonenumber::{country, Mode, ParseError};

/// Outcome of normalising a raw phone-number string.
///
/// Never a hard error: every variant besides [`Normalization::E164`] means the caller should
/// keep using the original raw value — normalisation is best-effort, not a gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Normalization {
    /// Successfully resolved to a valid, canonical E.164 number.
    E164(String),
    /// Structurally parsed under the given/inferred region but not a valid, assignable number.
    InvalidNumber,
    /// No country code present and no `fallback_region` supplied to resolve one.
    MissingRegion,
    /// An explicit `+`/`00` country calling code was present but isn't a real, assigned one.
    InvalidCountryCode,
    /// Fewer digits than any region's numbering plan allows.
    TooShort,
    /// More digits than any region's numbering plan allows.
    TooLong,
    /// Input isn't recognisable as a phone number at all.
    NotANumber,
}

/// Normalises `raw` to E.164 using `phonenumber`'s regional metadata.
///
/// `fallback_region` is only consulted when `raw` carries no country calling code of its own
/// (no leading `+`/`00`) — a number that already has one derives its region from that code
/// regardless of `fallback_region`. Pass `None` to only resolve numbers that are already
/// self-describing.
#[must_use]
pub fn normalize_number(raw: &str, fallback_region: Option<country::Id>) -> Normalization {
    match phonenumber::parse(fallback_region, raw) {
        Ok(number) => {
            if !number.is_valid() {
                return Normalization::InvalidNumber;
            }
            Normalization::E164(number.format().mode(Mode::E164).to_string())
        }
        Err(ParseError::InvalidCountryCode) => {
            if looks_international(raw) {
                Normalization::InvalidCountryCode
            } else {
                Normalization::MissingRegion
            }
        }
        Err(ParseError::TooShortAfterIdd | ParseError::TooShortNsn) => Normalization::TooShort,
        Err(ParseError::TooLong) => Normalization::TooLong,
        Err(ParseError::NoNumber | ParseError::MalformedInteger(_)) => Normalization::NotANumber,
    }
}

/// Distinguishes an explicit-but-unassigned country code (e.g. `+999...`) from a national-format
/// number with no country code at all — both surface as [`ParseError::InvalidCountryCode`].
fn looks_international(raw: &str) -> bool {
    let value = raw.trim_start();
    value.starts_with('+') || value.starts_with("00")
}
