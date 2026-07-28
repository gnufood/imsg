//! Unit tests for `folders` rendering and the footer composition [`super::run`] applies.

use ipc::FolderDto;

use super::render;
use crate::commands::live_footer;

fn rows(names: &[&str]) -> Vec<FolderDto> {
    names.iter().map(|n| FolderDto { name: (*n).to_owned() }).collect()
}

#[test]
fn renders_one_folder_per_line_in_device_order() {
    let out = render(&rows(&["inbox", "sent", "outbox", "deleted"]));
    assert_eq!(out, "inbox\nsent\noutbox\ndeleted\n");
}

#[test]
fn renders_placeholder_for_an_empty_listing() {
    assert_eq!(render(&rows(&[])), "(no folders)");
}

/// Both `run` paths compose exactly this. `folders` is always a live device read, so the
/// footer distinguishes a device that reports no folders from a stale or store-backed read.
#[test]
fn run_output_carries_the_live_footer() {
    assert_eq!(live_footer(render(&rows(&["inbox"]))), "inbox\n(live from device)");
    assert_eq!(live_footer(render(&rows(&[]))), "(no folders)\n(live from device)");
}
