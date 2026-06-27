//! Command-line surface: global options and the subcommand tree.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::Verbosity;
use map_core::folders::Folder;
use pbap_core::phonebook::PhonebookPath;

/// Top-level invocation: global transport/config options plus the chosen subcommand.
#[derive(Parser, Debug)]
#[command(name = "imsg", version, about = "iMessage client over Bluetooth MAP/PBAP")]
pub(crate) struct Cli {
    /// Route MAP and PBAP connections through the iroh hub configured via `imsg spoke add`.
    /// `imsg watch --hub` streams live MAP notification events from the hub.
    #[arg(long, global = true)]
    pub(crate) hub: bool,

    /// Override the configured device MAC address (RFCOMM only).
    #[arg(long, global = true, value_name = "ADDR")]
    pub(crate) device: Option<String>,

    /// Explicit config file path, overriding the layered default search.
    #[arg(long, global = true, value_name = "PATH")]
    pub(crate) config: Option<PathBuf>,

    /// Verbosity: `-v`/`-vv` raise the log level, `-q` lowers it.
    #[command(flatten)]
    pub(crate) verbosity: Verbosity,

    /// Operation to perform.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// One MAP/PBAP operation. Flag sets mirror the underlying crate APIs.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Send an SMS to a phone number.
    Send {
        /// Recipient phone number.
        number: String,
        /// Message body.
        message: String,
    },
    /// List messages in a folder.
    List {
        /// Folder to list; defaults to inbox.
        folder: Option<FolderArg>,
        /// Only unread messages.
        #[arg(long)]
        unread: bool,
        /// Show the raw MAP handle for each message (for use with `get`/`delete`).
        #[arg(long, short = 'l')]
        long: bool,
        /// Filter by originating address.
        #[arg(long, value_name = "NUMBER")]
        from: Option<String>,
        /// Filter to messages at or after this MAP timestamp (`YYYYMMDDTHHMMSS`).
        #[arg(long, value_name = "DATE")]
        since: Option<String>,
        /// Maximum number of entries to return.
        #[arg(long, value_name = "N")]
        limit: Option<u16>,
        /// Skip the first N entries (pagination).
        #[arg(long, value_name = "N")]
        offset: Option<u16>,
    },
    /// Fetch one message body by handle.
    Get {
        /// Message handle.
        handle: String,
        /// Folder the handle lives in; defaults to inbox.
        #[arg(long)]
        folder: Option<FolderArg>,
        /// Mark the message read after fetching.
        #[arg(long)]
        mark_read: bool,
    },
    /// Delete (or undelete) a message by handle.
    Delete {
        /// Message handle.
        handle: String,
        /// Folder the handle lives in; defaults to inbox.
        #[arg(long)]
        folder: Option<FolderArg>,
        /// Restore a previously deleted message instead of deleting it.
        #[arg(long)]
        undelete: bool,
    },
    /// Pull contacts from a phonebook.
    Contacts {
        /// List handles and names only, without full vCards.
        #[arg(long, conflicts_with_all = ["get", "lookup"])]
        list: bool,
        /// Fetch a single contact by handle.
        #[arg(long, value_name = "HANDLE", conflicts_with = "lookup")]
        get: Option<String>,
        /// Reverse-lookup a contact by phone number.
        #[arg(long, value_name = "NUMBER")]
        lookup: Option<String>,
        /// Phonebook to read.
        #[arg(long, value_enum, default_value = "pb")]
        path: PathArg,
        /// Show phone numbers as stored; skip E.164 normalisation. No effect on `--list`.
        #[arg(long)]
        raw: bool,
        /// Maximum contacts per page; omit to show all.
        #[arg(long, value_name = "N")]
        limit: Option<u16>,
        /// Page number (1-indexed). Ignored when `--limit` is not set.
        #[arg(long, value_name = "N")]
        page: Option<u16>,
    },
    /// Group inbox and sent messages into conversation threads.
    Threads,
    /// Stream live message-notification events and persist them to the local store.
    Watch,
    /// Backfill the local store with all messages from the device since the last sync.
    Sync {
        /// Restrict the backfill to a single folder; omit to sync all folders.
        #[arg(long, value_enum)]
        folder: Option<FolderArg>,
    },
    /// Stop using the local store for reads; synced data is preserved by default.
    Unsync {
        /// Delete the database file and all synced data in addition to disabling sync.
        #[arg(long)]
        purge: bool,
    },
    /// List the MAP message folders on the device.
    Folders,
    /// Start the iroh hub on this machine and print the node key for spokes.
    Hub,
    /// Manage spoke configuration for connecting to a remote hub.
    Spoke {
        /// Spoke action.
        #[command(subcommand)]
        cmd: SpokeCmd,
    },
    /// Inspect or modify local configuration.
    Config {
        /// Configuration action.
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Query the session broker.
    Broker {
        /// Broker action.
        #[command(subcommand)]
        cmd: BrokerCmd,
    },
    /// Internal: session broker process, auto-started by the CLI — not for direct invocation.
    #[command(name = "__broker_serve", hide = true)]
    BrokerServe,
}

/// `broker` actions.
#[derive(Subcommand, Debug)]
pub(crate) enum BrokerCmd {
    /// Report whether the broker is running and whether its MAP session is connected.
    Status,
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
