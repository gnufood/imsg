//! [`DeviceOp`] — split out of `super` to keep that module under the size ceiling.

use ipc::WatchEvent;
use tokio::sync::{broadcast, oneshot};

/// A message sent from a connection task to the device actor.
pub(in crate::runtime) enum DeviceOp {
    /// Drain outbox then run a full MAP folder sync.
    Sync {
        /// MAP folder path, or `None` to sync all standard folders.
        folder: Option<String>,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Push an outgoing SMS via the MAP outbox.
    Send {
        /// E.164 or local phone number.
        number: String,
        /// UTF-8 message body.
        message: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Delete a MAP message by handle and folder.
    Delete {
        /// MAP message handle (opaque string from the device).
        msg_handle: String,
        /// MAP folder name (e.g. `"inbox"`).
        folder: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Run incremental catch-up backfill without a full drain.
    Backfill { reply: oneshot::Sender<ipc::BrokerResponse> },
    /// Read a folder listing live and return message DTOs; no store write.
    LiveList {
        /// MAP folder name, or `None` for inbox.
        folder: Option<String>,
        /// Keep only unread messages.
        unread: bool,
        /// Keep only messages whose resolved address equals this value.
        from: Option<String>,
        /// Earliest message datetime as a MAP string; ignored if unparseable.
        since: Option<String>,
        /// Maximum rows after `offset`.
        limit: Option<u16>,
        /// Rows to skip from the newest-first window.
        offset: u16,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Fetch one message body live by handle and return its DTO; no store write.
    LiveGet {
        /// Opaque MAP message handle.
        handle: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Aggregate live Inbox+Sent listings into per-contact thread DTOs; no store write.
    LiveThreads { reply: oneshot::Sender<ipc::BrokerResponse> },
    /// List the device's MAP message folders under `telecom/msg`; no store write.
    LiveFolders { reply: oneshot::Sender<ipc::BrokerResponse> },
    /// Mark a message read on the device only (non-opted-in `get --read`); no store write.
    LiveMarkRead {
        /// Opaque MAP message handle.
        handle: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Push an outgoing SMS to the device only (non-opted-in `send`); no store write.
    LiveSend {
        /// Recipient phone number.
        number: String,
        /// UTF-8 message body.
        message: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Pull the main phonebook and upsert contact display names into the local contacts cache;
    /// runs against the actor's held PBAP session, independent of the MAP session.
    SyncContacts { reply: oneshot::Sender<ipc::BrokerResponse> },
    /// List a phonebook live and return entry DTOs; no store write. Runs against the actor's
    /// held PBAP session.
    ListContacts {
        /// Lowercase PBAP phonebook path name, or `None` for the main phonebook.
        path: Option<String>,
        /// Maximum rows after `offset`.
        limit: Option<u16>,
        /// Rows to skip, in device-reported order.
        offset: u16,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Fetch one contact vCard live by handle; no store write. Runs against the actor's held
    /// PBAP session.
    GetContact {
        /// Lowercase PBAP phonebook path name, or `None` for the main phonebook.
        path: Option<String>,
        /// Opaque PBAP vCard handle.
        handle: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Reverse-look up a contact by phone number live, then pull its vCard; no store write.
    /// Runs against the actor's held PBAP session.
    LookupContact {
        /// Lowercase PBAP phonebook path name, or `None` for the main phonebook.
        path: Option<String>,
        /// Phone number to search for.
        number: String,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Pull every contact vCard in a phonebook live; no store write. Runs against the actor's
    /// held PBAP session.
    PullAllContacts {
        /// Lowercase PBAP phonebook path name, or `None` for the main phonebook.
        path: Option<String>,
        /// Maximum rows after `offset`.
        limit: Option<u16>,
        /// Rows to skip, in device-reported order.
        offset: u16,
        reply: oneshot::Sender<ipc::BrokerResponse>,
    },
    /// Subscribe to inbound MAP event notifications.
    ///
    /// Starts the MNS listener on the first subscriber. Returns a
    /// [`broadcast::Receiver`] for [`WatchEvent`] frames.
    Subscribe { reply: oneshot::Sender<broadcast::Receiver<WatchEvent>> },
    /// Decrement the subscriber count; stops MNS when it reaches zero.
    Unsubscribe,
}
