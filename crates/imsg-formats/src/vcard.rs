//! PBAP contact normalisation using calcard.
//!
//! Use [`Contact::from_vcard_str`] to extract display name and phone numbers
//! from a raw vCard 3.0 string pulled via PBAP.

use calcard::vcard::{VCardProperty, VCardValue};
use thiserror::Error;

/// Contact parsing errors — calcard cannot parse the vCard input.
#[derive(Debug, Error)]
pub enum ContactError {
    /// calcard could not parse the vCard; the input was not well-formed vCard.
    #[error("vCard parse failed")]
    ParseFailed,
}

/// Normalised contact extracted from a PBAP vCard.
///
/// Phone numbers are whitespace-stripped but otherwise preserved as-is; no
/// E.164 normalisation is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    /// Value of the FN property; `None` if absent.
    pub display_name: Option<String>,
    phones: Vec<String>,
}

impl Contact {
    /// Whitespace-stripped, non-empty TEL values in vCard order.
    #[inline]
    #[must_use]
    pub fn phones(&self) -> &[String] {
        &self.phones
    }
}

impl Contact {
    /// Extracts FN and TEL; input must be a single vCard.
    ///
    /// # Errors
    ///
    /// Returns [`ContactError::ParseFailed`] if calcard cannot parse the input
    /// as a valid vCard.
    pub fn from_vcard_str(input: &str) -> Result<Self, ContactError> {
        let vcard = calcard::vcard::VCard::parse(input).map_err(|_| ContactError::ParseFailed)?;

        let display_name = vcard
            .property(&VCardProperty::Fn)
            .and_then(|e| e.values.first())
            .and_then(VCardValue::as_text)
            .map(str::to_owned);

        let phones = vcard
            .properties(&VCardProperty::Tel)
            .flat_map(|e| e.values.iter())
            .filter_map(VCardValue::as_text)
            .map(|s| s.split_whitespace().collect::<String>())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self { display_name, phones })
    }
}
