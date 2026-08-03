use super::*;

/// A removal and a no-op must not render identically — reporting "uninstalled" for a level
/// that had nothing registered would be a false success.
#[test]
fn uninstall_message_distinguishes_removal_from_no_op() {
    let removed =
        uninstall_message(service::UninstallOutcome::Uninstalled, service::ServiceLevel::User);
    let absent =
        uninstall_message(service::UninstallOutcome::NotInstalled, service::ServiceLevel::User);

    assert_ne!(removed, absent);
    assert!(removed.contains("uninstalled"), "got {removed}");
    assert!(!absent.contains("uninstalled"), "no-op must not claim a removal: {absent}");
}

/// The level is echoed back so a user who ran the per-user variant by mistake can see it.
#[test]
fn uninstall_message_names_the_level() {
    let system =
        uninstall_message(service::UninstallOutcome::Uninstalled, service::ServiceLevel::System);
    assert!(system.contains("System"), "got {system}");
}
