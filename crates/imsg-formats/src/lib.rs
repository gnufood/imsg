//! Parsed representations of MAP and PBAP wire formats, plus phone-number normalisation shared
//! by both.

/// bMessage encoder and parser.
pub mod bmessage;
/// Phone-number normalisation, shared by the PBAP and MAP address paths.
pub mod phone;
/// PBAP contact normalisation using calcard.
pub mod vcard;
/// MAP XML folder listing parser.
pub mod xml;
