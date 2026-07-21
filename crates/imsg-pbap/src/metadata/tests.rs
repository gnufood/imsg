use super::PhonebookMetadata;

fn tlv(tag: u8, value: &[u8]) -> Vec<u8> {
    let mut out = vec![tag, u8::try_from(value.len()).unwrap_or(u8::MAX)];
    out.extend_from_slice(value);
    out
}

#[test]
fn parse_all_recognised_fields() {
    let db_id = [0xAA; 16];
    let primary = [0xBB; 16];
    let secondary = [0xCC; 16];
    let mut bytes = tlv(0x08, &[0x00, 0x05]);
    bytes.extend(tlv(0x0D, &db_id));
    bytes.extend(tlv(0x0A, &primary));
    bytes.extend(tlv(0x0B, &secondary));

    let meta = PhonebookMetadata::parse(&bytes);
    assert_eq!(meta.size, Some(5));
    assert_eq!(meta.database_id, Some(db_id));
    assert_eq!(meta.primary_version, Some(primary));
    assert_eq!(meta.secondary_version, Some(secondary));
}

#[test]
fn parse_empty_bytes_yields_all_none() {
    let meta = PhonebookMetadata::parse(&[]);
    assert_eq!(meta, PhonebookMetadata::default());
}

#[test]
fn parse_ignores_unknown_tags() {
    let mut bytes = tlv(0x99, &[0x01, 0x02, 0x03]);
    bytes.extend(tlv(0x08, &[0x00, 0x07]));
    let meta = PhonebookMetadata::parse(&bytes);
    assert_eq!(meta.size, Some(7));
}

#[test]
fn parse_drops_truncated_trailing_tlv() {
    // A well-formed size TLV followed by a tag/len header whose declared value length exceeds
    // the remaining bytes.
    let mut bytes = tlv(0x08, &[0x00, 0x01]);
    bytes.extend([0x0D, 0x10, 0xAA, 0xAA]);
    let meta = PhonebookMetadata::parse(&bytes);
    assert_eq!(meta.size, Some(1));
    assert_eq!(meta.database_id, None);
}

#[test]
fn parse_wrong_length_field_is_ignored() {
    // size TLV with a 1-byte value instead of the expected 2 bytes.
    let bytes = tlv(0x08, &[0x05]);
    let meta = PhonebookMetadata::parse(&bytes);
    assert_eq!(meta.size, None);
}
