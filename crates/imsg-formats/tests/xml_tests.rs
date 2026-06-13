//! Integration tests for MAP XML folder listing parsing.

use imsg_formats::xml::{FolderEntry, FolderListing};

const FIXTURE_RSP: &[u8] =
    include_bytes!("../../imsg-obex/tests/fixtures/get_folder_listing_000_rsp.bin");

fn fixture_xml() -> Result<&'static [u8], Box<dyn std::error::Error>> {
    // OBEX frame: response_code(1) + packet_length(2) + EndOfBody_header_id(1) + EndOfBody_header_len(2) = 6 bytes before XML body
    let hdr: [u8; 2] = FIXTURE_RSP.get(1..3).ok_or("fixture too short")?.try_into()?;
    let total = usize::from(u16::from_be_bytes(hdr));
    FIXTURE_RSP.get(6..total).ok_or_else(|| "fixture truncated".into())
}

#[test]
fn parse_fixture_returns_four_ios_folders() -> Result<(), Box<dyn std::error::Error>> {
    let listing = FolderListing::parse(fixture_xml()?)?;
    assert_eq!(listing.folders().len(), 4);
    Ok(())
}

#[test]
fn fixture_contains_expected_folder_names() -> Result<(), Box<dyn std::error::Error>> {
    let listing = FolderListing::parse(fixture_xml()?)?;
    let names: Vec<&str> = listing.folders().iter().map(FolderEntry::name).collect();
    assert!(names.contains(&"inbox"));
    assert!(names.contains(&"sent"));
    assert!(names.contains(&"outbox"));
    assert!(names.contains(&"deleted"));
    Ok(())
}

#[test]
fn parse_inline_single_folder() -> Result<(), Box<dyn std::error::Error>> {
    let xml = b"<?xml version=\"1.0\"?><folder-listing version=\"1.0\"><folder name=\"inbox\"/></folder-listing>";
    let listing = FolderListing::parse(xml)?;
    assert_eq!(listing.folders().len(), 1);
    assert_eq!(listing.folders().first().map(FolderEntry::name), Some("inbox"));
    Ok(())
}

#[test]
fn parse_empty_folder_listing() -> Result<(), Box<dyn std::error::Error>> {
    let xml = b"<?xml version=\"1.0\"?><folder-listing version=\"1.0\"></folder-listing>";
    let listing = FolderListing::parse(xml)?;
    assert!(listing.folders().is_empty());
    Ok(())
}

#[test]
fn parse_returns_folders_in_document_order() -> Result<(), Box<dyn std::error::Error>> {
    let xml = b"<folder-listing version=\"1.0\"><folder name=\"a\"/><folder name=\"b\"/><folder name=\"c\"/></folder-listing>";
    let listing = FolderListing::parse(xml)?;
    let names: Vec<&str> = listing.folders().iter().map(FolderEntry::name).collect();
    assert_eq!(names, ["a", "b", "c"]);
    Ok(())
}
