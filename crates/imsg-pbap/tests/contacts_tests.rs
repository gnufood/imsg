//! Integration tests for PBAP contact operations.

use imsg_pbap::normalize_number;

#[test]
fn normalize_us_10digit() {
    assert_eq!(normalize_number("5551234567"), "+15551234567");
}

#[test]
fn normalize_us_formatted() {
    assert_eq!(normalize_number("(555) 123-4567"), "+15551234567");
}

#[test]
fn normalize_us_11digit_with_1() {
    assert_eq!(normalize_number("15551234567"), "+15551234567");
}

#[test]
fn normalize_already_e164() {
    assert_eq!(normalize_number("+15551234567"), "+15551234567");
}

#[test]
fn normalize_non_us_passthrough() {
    assert_eq!(normalize_number("+447911123456"), "+447911123456");
}

#[test]
fn normalize_short_passthrough() {
    assert_eq!(normalize_number("555"), "555");
}
