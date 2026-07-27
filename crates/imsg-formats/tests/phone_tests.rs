//! Integration tests for phone-number normalisation.

use imsg_formats::phone::{normalize_number, Normalization, PhoneField};
use phonenumber::country;

#[test]
fn strips_uk_trunk_prefix_when_country_code_present() {
    let result = normalize_number("+44(0)1753866488", None);
    assert!(matches!(result, Normalization::E164(ref n) if n == "+441753866488"));
}

#[test]
fn already_corrupted_value_self_heals() {
    // What the old naive digit-strip normalize_number used to produce for the case above.
    let result = normalize_number("+4401753866488", None);
    assert!(matches!(result, Normalization::E164(ref n) if n == "+441753866488"));
}

#[test]
fn rejects_unassigned_country_code() {
    let result = normalize_number("+999123456789", None);
    assert!(matches!(result, Normalization::InvalidCountryCode));
}

#[test]
fn bare_national_number_without_region_is_unresolved() {
    let result = normalize_number("01753 866488", None);
    assert!(matches!(result, Normalization::MissingRegion));
}

#[test]
fn bare_national_number_resolves_with_configured_region() {
    let result = normalize_number("01753 866488", Some(country::Id::GB));
    assert!(matches!(result, Normalization::E164(ref n) if n == "+441753866488"));
}

#[test]
fn too_short_number_is_reported_as_such() {
    let result = normalize_number("+441", None);
    assert!(matches!(result, Normalization::TooShort));
}

#[test]
fn structurally_parsed_but_unassigned_number_is_invalid_not_too_short() {
    // Parses fine (country code 1, national "234") but too short to be a real US number —
    // distinct from a ParseError::TooShort*, which fires before a PhoneNumber is even built.
    let result = normalize_number("+1234", None);
    assert!(matches!(result, Normalization::InvalidNumber));
}

#[test]
fn garbage_input_is_not_a_number() {
    let result = normalize_number("not a number", None);
    assert!(matches!(result, Normalization::NotANumber));
}

#[test]
fn chinese_mobile_number_resolves_with_configured_region() {
    let result = normalize_number("13812345678", Some(country::Id::CN));
    assert!(matches!(result, Normalization::E164(ref n) if n == "+8613812345678"));
}

#[test]
fn chinese_landline_strips_trunk_prefix_with_configured_region() {
    let result = normalize_number("01012345678", Some(country::Id::CN));
    assert!(matches!(result, Normalization::E164(ref n) if n == "+861012345678"));
}

#[test]
fn phone_field_new_resolves_and_retains_raw() {
    let field = PhoneField::new("+44(0)1753866488", None);
    assert_eq!(field.raw(), "+44(0)1753866488");
    assert_eq!(field.e164(), Some("+441753866488"));
    assert_eq!(field.canonical(), "+441753866488");
    assert_eq!(field.display(), "+441753866488");
}

#[test]
fn phone_field_new_falls_back_to_raw_when_unresolved() {
    let field = PhoneField::new("01753 866488", None);
    assert_eq!(field.raw(), "01753 866488");
    assert_eq!(field.e164(), None);
    assert_eq!(field.canonical(), "01753 866488");
    assert_eq!(field.display(), "01753 866488");
}

#[test]
fn phone_field_new_resolves_with_configured_region() {
    let field = PhoneField::new("01753 866488", Some(country::Id::GB));
    assert_eq!(field.raw(), "01753 866488");
    assert_eq!(field.e164(), Some("+441753866488"));
}

#[test]
fn phone_field_from_parts_trusts_stored_columns() {
    let field =
        PhoneField::from_parts("+44(0)1753866488".to_owned(), Some("+441753866488".to_owned()));
    assert_eq!(field.raw(), "+44(0)1753866488");
    assert_eq!(field.e164(), Some("+441753866488"));
    assert_eq!(field.canonical(), "+441753866488");
}

#[test]
fn phone_field_from_parts_without_e164_uses_raw_as_canonical() {
    let field = PhoneField::from_parts("01753 866488".to_owned(), None);
    assert_eq!(field.e164(), None);
    assert_eq!(field.canonical(), "01753 866488");
    assert_eq!(field.display(), "01753 866488");
}

#[test]
fn phone_fields_with_same_parts_are_equal() {
    let a = PhoneField::new("+44(0)1753866488", None);
    let b = PhoneField::from_parts("+44(0)1753866488".to_owned(), Some("+441753866488".to_owned()));
    assert_eq!(a, b);
}
