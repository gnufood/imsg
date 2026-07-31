//! `completions` subcommand: print a shell completion script, or install it directly.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::CommandFactory as _;
use clap_complete::Shell;

use crate::cli::Cli;

/// Prints the completion script for `shell`, or — with `install` set — writes it to the
/// shell's completion directory after a confirmation prompt; declining leaves disk untouched.
pub(crate) fn run(shell: Shell, install: bool) -> Result<String> {
    let mut buf = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "imsg", &mut buf);
    let script = String::from_utf8(buf).context("completion script was not valid UTF-8")?;

    if install {
        install_script(shell, &script)
    } else {
        Ok(script)
    }
}

// Can silently clobber a hand-edited completion file, hence the confirm prompt.
fn install_script(shell: Shell, script: &str) -> Result<String> {
    let path = install_path(shell)?;
    let prompt = format!("Install {shell} completions to {}? (overwrites)", path.display());
    let confirmed = dialoguer::Confirm::new()
        .with_prompt(prompt)
        .default(true)
        .interact()
        .context("reading install confirmation")?;
    if !confirmed {
        return Ok("Aborted — no completions installed.".to_owned());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, script).with_context(|| format!("writing {}", path.display()))?;

    let note = match shell {
        Shell::Bash => " Requires the `bash-completion` package.",
        Shell::Zsh => " Add `fpath+=(~/.zfunc)` before `compinit` in ~/.zshrc if not already.",
        _ => "",
    };
    Ok(format!("Installed to {}.{note} Restart your shell to pick it up.", path.display()))
}

// PowerShell/Elvish have no drop-in completion directory convention worth guessing at — plain
// stdout printing still works for both, just not `--install`.
fn install_path(shell: Shell) -> Result<PathBuf> {
    match shell {
        Shell::Bash => Ok(dirs::data_dir()
            .context("resolving XDG data directory")?
            .join("bash-completion/completions/imsg")),
        Shell::Zsh => {
            Ok(dirs::home_dir().context("resolving home directory")?.join(".zfunc/_imsg"))
        }
        Shell::Fish => Ok(dirs::config_dir()
            .context("resolving XDG config directory")?
            .join("fish/completions/imsg.fish")),
        _ => anyhow::bail!(
            "`--install` isn't supported for {shell}; redirect its stdout output instead"
        ),
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    #[test]
    fn run_without_install_prints_script() {
        let result = run(Shell::Zsh, false);
        assert!(result.is_ok(), "expected Ok, got {result:?}");
        let Ok(script) = result else { unreachable!() };
        assert!(script.contains("_imsg"), "expected zsh completion script, got: {script}");
    }

    #[test]
    fn install_unsupported_for_powershell_and_elvish() {
        assert!(install_path(Shell::PowerShell).is_err());
        assert!(install_path(Shell::Elvish).is_err());
    }

    #[test]
    #[serial]
    fn install_path_matches_shell_convention() {
        let tmp =
            std::env::temp_dir().join(format!("imsg_completions_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).ok();
        let saved_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", &tmp);
        std::env::remove_var("XDG_DATA_HOME");
        std::env::remove_var("XDG_CONFIG_HOME");

        let bash = install_path(Shell::Bash);
        let zsh = install_path(Shell::Zsh);
        let fish = install_path(Shell::Fish);

        match saved_home {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
        std::fs::remove_dir_all(&tmp).ok();

        assert!(bash.is_ok(), "expected Ok, got {bash:?}");
        let Ok(bash) = bash else { unreachable!() };
        assert!(bash.ends_with("bash-completion/completions/imsg"));

        assert!(zsh.is_ok(), "expected Ok, got {zsh:?}");
        let Ok(zsh) = zsh else { unreachable!() };
        assert_eq!(zsh, tmp.join(".zfunc/_imsg"));

        assert!(fish.is_ok(), "expected Ok, got {fish:?}");
        let Ok(fish) = fish else { unreachable!() };
        assert!(fish.ends_with("fish/completions/imsg.fish"));
    }
}
