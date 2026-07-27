//! Unit tests for the `store` contact row -> GUI DTO conversions.

use super::*;

#[test]
fn contact_entry_dto_preserves_uid_and_name() {
    let row = store::ContactEntryRow { uid: "U1".into(), display_name: Some("Jane Doe".into()) };
    let dto = ContactEntryDto::from(&row);
    assert_eq!(dto.uid, "U1");
    assert_eq!(dto.display_name.as_deref(), Some("Jane Doe"));
}

#[test]
fn contact_entry_dto_keeps_missing_name_absent() {
    let row = store::ContactEntryRow { uid: "U1".into(), display_name: None };
    assert_eq!(ContactEntryDto::from(&row).display_name, None);
}

#[test]
fn contact_dto_preserves_uid_name_and_phones() {
    let row = store::ContactRow {
        uid: "U1".into(),
        display_name: Some("Jane Doe".into()),
        phones: vec![
            store::PhoneField::new("+15550001", None),
            store::PhoneField::new("+15550002", None),
        ],
    };
    let dto = ContactDto::from(&row);
    assert_eq!(dto.uid, "U1");
    assert_eq!(dto.display_name.as_deref(), Some("Jane Doe"));
    assert_eq!(
        dto.phones,
        vec![
            PhoneDto { raw: "+15550001".to_owned(), e164: None },
            PhoneDto { raw: "+15550002".to_owned(), e164: None },
        ]
    );
}

#[test]
fn contact_dto_keeps_empty_phones_and_missing_name() {
    let row = store::ContactRow { uid: "U1".into(), display_name: None, phones: vec![] };
    let dto = ContactDto::from(&row);
    assert_eq!(dto.display_name, None);
    assert!(dto.phones.is_empty());
}
