//! Resolves the real invoking user for a `--system` service install, so it runs as them
//! instead of root — root has no access to the installing user's config or keyring.

use std::env;
use std::path::PathBuf;

use crate::{Error, ServiceLevel};

/// Picks `SUDO_USER` over `$USER`/`whoami` (which report `root` under `sudo`), but only when
/// `sudo_user_uid` (the claimed name's real passwd UID, looked up by the caller) matches
/// `sudo_uid` (`$SUDO_UID`, set by the same `sudo` invocation that set `SUDO_USER`) — an
/// unmatched or unparseable pair means `SUDO_USER` didn't come from that trusted `sudo` call, so
/// it's ignored in favor of `$USER`. Refuses a bare `root` result either way — a genuine root
/// login (no `sudo`) would otherwise silently install a `--system` service that runs as root,
/// defeating the point of resolving a real user at all. Pure so the guard logic is testable
/// without a real environment or passwd database.
fn resolve_invoking_username(
    sudo_user: Option<&str>,
    sudo_user_uid: Option<u32>,
    sudo_uid: Option<&str>,
    user: Option<&str>,
) -> Option<String> {
    let corroborated = sudo_uid.and_then(|s| s.parse::<u32>().ok()) == sudo_user_uid;
    let name =
        sudo_user.filter(|s| !s.is_empty()).filter(|_| corroborated).or(user).unwrap_or_default();
    (!name.is_empty() && name != "root").then(|| name.to_owned())
}

/// Resolves the real invoking user's passwd entry for a `--system` install, so the service can
/// run as them (config, keyring, and Bluetooth pairing all live under their account, not root's).
pub(crate) fn invoking_user() -> Result<nix::unistd::User, Error> {
    let sudo_user = env::var("SUDO_USER").ok();
    let sudo_user_uid = sudo_user
        .as_deref()
        .and_then(|name| nix::unistd::User::from_name(name).ok().flatten())
        .map(|u| u.uid.as_raw());
    let name = resolve_invoking_username(
        sudo_user.as_deref(),
        sudo_user_uid,
        env::var("SUDO_UID").ok().as_deref(),
        env::var("USER").ok().as_deref(),
    )
    .ok_or(Error::NoInvokingUser)?;
    nix::unistd::User::from_name(&name).map_err(Error::UserLookup)?.ok_or(Error::UnknownUser(name))
}

/// `username`, home directory, and `HOME`/`XDG_CONFIG_HOME` environment for `ServiceInstallCtx`.
pub(crate) type ServiceIdentity = (Option<String>, Option<PathBuf>, Option<Vec<(String, String)>>);

/// The real invoking user's identity and home for [`ServiceLevel::System`] (which otherwise
/// runs as root with no access to that user's config or keyring), or all-`None` for
/// [`ServiceLevel::User`], which already runs as the right account without any of this.
pub(crate) fn system_identity(level: ServiceLevel) -> Result<ServiceIdentity, Error> {
    if level != ServiceLevel::System {
        return Ok((None, None, None));
    }
    let user = invoking_user()?;
    let config_home = user.dir.join(".config").to_string_lossy().into_owned();
    let env = vec![
        ("HOME".to_owned(), user.dir.to_string_lossy().into_owned()),
        ("XDG_CONFIG_HOME".to_owned(), config_home),
    ];
    Ok((Some(user.name), Some(user.dir), Some(env)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SUDO_USER` wins over `$USER` when `SUDO_UID` corroborates it — under `sudo`, `$USER`
    /// reports `root`.
    #[test]
    fn resolve_invoking_username_prefers_corroborated_sudo_user() {
        assert_eq!(
            resolve_invoking_username(Some("alice"), Some(1000), Some("1000"), Some("root")),
            Some("alice".to_owned())
        );
    }

    /// `SUDO_USER` is set but `SUDO_UID` doesn't match its real passwd UID — not a trustworthy
    /// pair, so it's ignored in favor of `$USER` rather than blindly followed.
    #[test]
    fn resolve_invoking_username_rejects_uncorroborated_sudo_user() {
        assert_eq!(
            resolve_invoking_username(Some("alice"), Some(1000), Some("999"), Some("bob")),
            Some("bob".to_owned())
        );
    }

    /// `SUDO_UID` missing or unparseable alongside a `SUDO_USER` claim: also not corroborated.
    #[test]
    fn resolve_invoking_username_rejects_sudo_user_without_sudo_uid() {
        assert_eq!(
            resolve_invoking_username(Some("alice"), Some(1000), None, Some("bob")),
            Some("bob".to_owned())
        );
        assert_eq!(
            resolve_invoking_username(Some("alice"), Some(1000), Some("not-a-uid"), Some("bob")),
            Some("bob".to_owned())
        );
    }

    /// No `sudo` in play (direct login as a normal user): falls back to `$USER`.
    #[test]
    fn resolve_invoking_username_falls_back_to_user() {
        assert_eq!(
            resolve_invoking_username(None, None, None, Some("alice")),
            Some("alice".to_owned())
        );
    }

    /// A genuine root login (no `sudo`, `$USER` is `root`) has no real user to fall back to.
    #[test]
    fn resolve_invoking_username_refuses_bare_root() {
        assert_eq!(resolve_invoking_username(None, None, None, Some("root")), None);
        assert_eq!(resolve_invoking_username(None, None, None, None), None);
    }

    /// An empty `SUDO_USER` (unset-but-present edge case) doesn't win over a real `$USER`.
    #[test]
    fn resolve_invoking_username_skips_empty_sudo_user() {
        assert_eq!(
            resolve_invoking_username(Some(""), None, None, Some("alice")),
            Some("alice".to_owned())
        );
    }
}
