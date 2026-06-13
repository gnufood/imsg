//! Integration tests for PBAP contact normalisation.

use imsg_formats::vcard::{Contact, ContactError};

#[test]
fn extracts_display_name_and_phone() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:John Doe\r\nTEL:+15559876543\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.display_name.as_deref(), Some("John Doe"));
    assert_eq!(contact.phones(), ["+15559876543"]);
    Ok(())
}

#[test]
fn strips_whitespace_from_phone() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Jane\r\nTEL:+1 555 987 6543\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.phones(), ["+15559876543"]);
    Ok(())
}

#[test]
fn multiple_tel_properties() -> Result<(), ContactError> {
    let input = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Multi\r\nTEL:+1111\r\nTEL:+2222\r\nEND:VCARD\r\n";
    let contact = Contact::from_vcard_str(input)?;
    assert_eq!(contact.phones().len(), 2);
    assert!(contact.phones().iter().any(|p| p == "+1111"));
    assert!(contact.phones().iter().any(|p| p == "+2222"));
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
