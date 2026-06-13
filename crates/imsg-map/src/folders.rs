//! Folder listing and SETPATH navigation for MAP folder hierarchy.

use tokio::io::{AsyncRead, AsyncWrite};

use crate::client::MapClient;
use crate::{FolderListing, MapError};

/// iOS telecom/msg folder hierarchy. Validated on SETPATH.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Folder {
    /// SETPATH segment `"inbox"`.
    Inbox,
    /// SETPATH segment `"sent"`.
    Sent,
    /// Queued outbound messages awaiting delivery.
    Outbox,
    /// Messages moved to the trash.
    Deleted,
}

impl Folder {
    /// SETPATH segment name for this folder as required by the MAP specification.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Sent => "sent",
            Self::Outbox => "outbox",
            Self::Deleted => "deleted",
        }
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> MapClient<T> {
    /// Backs up to root if already inside a subfolder, then navigates `telecom` → `msg` and
    /// lists that level. Use this instead of a bare [`MapClient::get_folder_listing`], which
    /// lists only the current OBEX directory — on iOS that is the root unless the
    /// `telecom/msg` SETPATHs are issued first.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if a backup or forward SETPATH fails, the server returns a
    /// non-OK response, or the listing XML is malformed.
    pub async fn list_message_folders(&mut self) -> Result<FolderListing, MapError> {
        self.reset_to_root().await?;
        self.setpath("telecom").await?;
        self.setpath("msg").await?;
        self.get_folder_listing().await
    }
}
