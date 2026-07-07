//! Leaf argument types for CLI subcommands — split out of `cli.rs` to stay under the
//! 250-line module ceiling as the subcommand tree grows.

use clap::{Subcommand, ValueEnum};
use map_core::folders::Folder;
use pbap_core::phonebook::PhonebookPath;

/// `broker` actions.
#[derive(Subcommand, Debug)]
pub(crate) enum BrokerCmd {
    /// Report whether the broker is running and whether its MAP session is connected.
    Status,
}

/// `daemon` actions.
#[derive(Subcommand, Debug)]
pub(crate) enum DaemonCmd {
    /// Start the persistent broker. Detaches into the background by default; idempotent if
    /// already running.
    Start {
        /// Stay attached instead of detaching — required under a process supervisor (e.g. a
        /// systemd unit). Stops on Ctrl-C, SIGTERM, or an IPC `Shutdown` request.
        #[arg(long)]
        foreground: bool,
    },
    /// Request a graceful stop. A no-op (not an error) if nothing is running.
    Stop,
    /// Report whether the daemon is running and whether its MAP session is connected.
    Status,
    /// Register the daemon with the native OS service manager (systemd/launchd/OpenRC/
    /// rc.d/sc.exe), so it starts on boot/login and restarts on failure. Optional — `imsg
    /// daemon start`/`stop` alone fully serve users who don't want OS-level supervision.
    Install {
        /// Register system-wide instead of for the current user only. Typically requires
        /// elevated privileges to install.
        #[arg(long)]
        system: bool,
    },
    /// Unregister the daemon service. A no-op if it was never installed.
    Uninstall {
        /// Match the `--system`/user scope the service was installed with.
        #[arg(long)]
        system: bool,
    },
}

/// `config` actions.
#[derive(Subcommand, Debug)]
pub(crate) enum ConfigCmd {
    /// Print the resolved configuration.
    Show,
    /// Persist the device MAC address to the user config file.
    SetDevice {
        /// Bluetooth MAC address (`XX:XX:XX:XX:XX:XX`).
        address: String,
    },
}

/// `spoke` actions.
#[derive(Subcommand, Debug)]
pub(crate) enum SpokeCmd {
    /// Persist the hub's iroh node key to the local config.
    Add {
        /// iroh hub node key (printed by `imsg hub`).
        key: String,
    },
}

/// MAP message folder selector. Maps to `map_core::Folder` at command time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum FolderArg {
    /// Received messages.
    Inbox,
    /// Sent messages.
    Sent,
    /// Pending outbound messages.
    Outbox,
    /// Deleted messages.
    Deleted,
}

/// Lowers the optional folder argument to a [`Folder`], defaulting to the inbox when omitted.
pub(crate) const fn folder_of(arg: Option<FolderArg>) -> Folder {
    match arg {
        Some(FolderArg::Inbox) | None => Folder::Inbox,
        Some(FolderArg::Sent) => Folder::Sent,
        Some(FolderArg::Outbox) => Folder::Outbox,
        Some(FolderArg::Deleted) => Folder::Deleted,
    }
}

/// Lowers the PBAP phonebook argument to a [`PhonebookPath`].
pub(crate) const fn path_of(arg: PathArg) -> PhonebookPath {
    match arg {
        PathArg::Pb => PhonebookPath::Pb,
        PathArg::Ich => PhonebookPath::Ich,
        PathArg::Och => PhonebookPath::Och,
        PathArg::Mch => PhonebookPath::Mch,
        PathArg::Cch => PhonebookPath::Cch,
        PathArg::Spd => PhonebookPath::Spd,
        PathArg::Fav => PhonebookPath::Fav,
    }
}

/// PBAP phonebook selector. Maps to `pbap_core::PhonebookPath` at command time.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum PathArg {
    /// Main phonebook.
    Pb,
    /// Incoming call history.
    Ich,
    /// Outgoing call history.
    Och,
    /// Missed call history.
    Mch,
    /// Combined call history.
    Cch,
    /// Speed-dial entries.
    Spd,
    /// Favourites.
    Fav,
}
