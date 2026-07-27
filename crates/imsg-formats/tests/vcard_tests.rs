//! Integration tests for PBAP contact normalisation.

use imsg_formats::vcard::{Contact, ContactError};

#[test]
fn extracts_display_name_and_phone() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:John Doe\r\nTEL:+15559876543\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.display_name.as_deref(), Some("John Doe"));
    let phone = contact.phones().first().ok_or(ContactError::ParseFailed)?;
    assert_eq!(phone.raw(), "+15559876543");
    Ok(())
}

#[test]
fn strips_whitespace_from_phone() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Jane\r\nTEL:+1 555 987 6543\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    let phone = contact.phones().first().ok_or(ContactError::ParseFailed)?;
    assert_eq!(phone.raw(), "+15559876543");
    Ok(())
}

#[test]
fn normalises_to_e164_at_parse_time_retaining_raw() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Ada\r\nTEL:+44 (0)1753 866488\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    let phone = contact.phones().first().ok_or(ContactError::ParseFailed)?;
    assert_eq!(phone.raw(), "+44(0)1753866488");
    assert_eq!(phone.e164(), Some("+441753866488"));
    Ok(())
}

#[test]
fn multiple_tel_properties() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Multi\r\nTEL:+1111\r\nTEL:+2222\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.phones().len(), 2);
    assert!(contact.phones().iter().any(|p| p.raw() == "+1111"));
    assert!(contact.phones().iter().any(|p| p.raw() == "+2222"));
    Ok(())
}

#[test]
fn missing_fn_gives_none() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nTEL:+1111\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.display_name, None);
    Ok(())
}

#[test]
fn empty_tel_is_filtered_out() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Empty\r\nTEL:\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert!(contact.phones().is_empty());
    Ok(())
}

#[test]
fn invalid_input_returns_error() {
    assert!(matches!(Contact::from_vcard_str("not a vcard"), Err(ContactError::ParseFailed)));
}

#[test]
fn extracts_uid() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:John Doe\r\nUID:abc-123\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.uid.as_deref(), Some("abc-123"));
    Ok(())
}

#[test]
fn missing_uid_gives_none() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Jane\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.uid, None);
    Ok(())
}
