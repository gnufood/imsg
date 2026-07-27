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

/// A phone number in both its raw device-reported form and its canonical E.164 form.
///
/// `e164` is `Some` only when [`normalize_number`] resolved the number; otherwise the number is
/// kept solely as `raw`. `raw` is always retained so nothing device-reported is lost and so
/// callers that must show the original form (e.g. `contacts --raw`) can.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhoneField {
    raw: String,
    e164: Option<String>,
}

impl PhoneField {
    /// Parses `raw` via [`normalize_number`], retaining the original. `region` is consulted only
    /// when `raw` carries no country code of its own; pass `None` to resolve only self-describing
    /// numbers.
    #[must_use]
    pub fn new(raw: &str, region: Option<country::Id>) -> Self {
        let e164 = match normalize_number(raw, region) {
            Normalization::E164(number) => Some(number),
            _ => None,
        };
        Self { raw: raw.to_owned(), e164 }
    }

    /// Reconstructs a field from previously stored columns without re-parsing. Trust the caller:
    /// `e164`, if present, is assumed already canonical — avoids re-running `phonenumber` on read
    /// and the metadata-drift reclassification that would cause.
    #[must_use]
    pub const fn from_parts(raw: String, e164: Option<String>) -> Self {
        Self { raw, e164 }
    }

    /// The original device-reported value, always present.
    #[must_use]
    pub fn raw(&self) -> &str {
        &self.raw
    }

    /// The canonical E.164 form, or `None` if the number could not be resolved.
    #[must_use]
    pub fn e164(&self) -> Option<&str> {
        self.e164.as_deref()
    }

    /// The value to compare on: E.164 when resolved, else the raw form. Two fields for the same
    /// real number compare equal here regardless of source formatting.
    #[must_use]
    pub fn canonical(&self) -> &str {
        self.e164.as_deref().unwrap_or(&self.raw)
    }

    /// The value to render by default: E.164 when resolved, else the raw form.
    #[must_use]
    pub fn display(&self) -> &str {
        self.canonical()
    }
}

/// Distinguishes an explicit-but-unassigned country code (e.g. `+999...`) from a national-format
/// number with no country code at all — both surface as [`ParseError::InvalidCountryCode`].
fn looks_international(raw: &str) -> bool {
    let value = raw.trim_start();
    value.starts_with('+') || value.starts_with("00")
}
