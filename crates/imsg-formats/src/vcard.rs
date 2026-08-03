//! PBAP contact normalisation using calcard.
//!
//! Use [`crate::vcard::Contact::from_vcard_str`] to extract display name and phone numbers
//! from a raw vCard 3.0 string pulled via PBAP.

use calcard::vcard::{VCardProperty, VCardValue};
use thiserror::Error;

use crate::phone::PhoneField;

/// Contact parsing errors — calcard cannot parse the vCard input.
#[derive(Debug, Error)]
pub enum ContactError {
    /// The input was not well-formed vCard.
    #[error("vCard parse failed")]
    ParseFailed,
}

/// Normalised contact extracted from a PBAP vCard.
///
/// Phone numbers are normalised to E.164 at parse time (the PBAP ingress chokepoint), retaining
/// the whitespace-stripped raw form; see [`PhoneField`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    /// Value of the FN property; `None` if absent.
    pub display_name: Option<String>,
    /// Value of the UID property; `None` if absent. A durable per-contact key — unlike PBAP
    /// handles, which are volatile and re-resolved on every sync.
    pub uid: Option<String>,
    phones: Vec<PhoneField>,
}

impl Contact {
    /// Non-empty TEL values in vCard order, each carrying its whitespace-stripped raw form and,
    /// when resolvable, its E.164 canonical form.
    #[inline]
    #[must_use]
    pub fn phones(&self) -> &[PhoneField] {
        &self.phones
    }
}

impl Contact {
    /// Extracts FN, TEL, and UID; input must be a single vCard.
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

        let uid = vcard
            .property(&VCardProperty::Uid)
            .and_then(|e| e.values.first())
            .and_then(VCardValue::as_text)
            .map(str::to_owned);

        let phones = vcard
            .properties(&VCardProperty::Tel)
            .flat_map(|e| e.values.iter())
            .filter_map(VCardValue::as_text)
            .map(|s| s.split_whitespace().collect::<String>())
            .filter(|s| !s.is_empty())
            .map(|s| PhoneField::new(&s, None))
            .collect();

        Ok(Self { display_name, uid, phones })
    }
}
