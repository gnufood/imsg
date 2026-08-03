//! SETPATH sequencing: folder navigation and the depth bookkeeping it depends on.

use futures::SinkExt;
use obex_core::client::ObexClient;
use tokio::io::{AsyncRead, AsyncWrite};

use super::MapClient;
use crate::{folders::Folder, MapError};

impl<T: AsyncRead + AsyncWrite + Unpin> MapClient<T> {
    /// `segment` is a single path component, not a slash-joined path — iOS requires one
    /// SETPATH per level. Does not validate that `segment` names an existing folder.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the request fails to encode, the transport closes, or the
    /// server returns a non-OK response.
    pub(crate) async fn setpath(&mut self, segment: &str) -> Result<(), MapError> {
        let req = self.obex.setpath_request(segment)?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        self.depth = self.depth.saturating_add(1);
        Ok(())
    }

    /// Decrements `depth` on success. Does not check whether depth is already zero.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if the request fails to encode, the transport closes, or the
    /// server returns a non-OK response.
    pub(crate) async fn setpath_up(&mut self) -> Result<(), MapError> {
        debug_assert!(self.depth > 0, "setpath_up called at root");
        let req = self.obex.setpath_backup_request()?;
        self.transport.send(req).await?;
        let rsp_bytes = Self::recv(&mut self.transport).await?;
        let rsp = ObexClient::parse_response(&rsp_bytes)?;
        if !rsp.opcode.is_ok() {
            return Err(MapError::ServerError(rsp.opcode.to_byte()));
        }
        self.depth = self.depth.saturating_sub(1);
        Ok(())
    }

    /// No-op when already at root (`depth == 0`). Does not navigate anywhere after resetting.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if any backup SETPATH fails to encode, the transport closes, or the
    /// server returns a non-OK response.
    pub(crate) async fn reset_to_root(&mut self) -> Result<(), MapError> {
        for _ in 0..self.depth {
            self.setpath_up().await?;
        }
        Ok(())
    }

    /// If already inside a subfolder (e.g. after a prior `set_folder` call), backs up to root
    /// first, then navigates `telecom` → `msg` → folder. iOS requires one SETPATH per level;
    /// a single slash-joined path is rejected.
    ///
    /// # Errors
    ///
    /// Returns [`MapError`] if any SETPATH fails to encode, the transport closes, or the
    /// server returns a non-OK response for any step.
    pub async fn set_folder(&mut self, folder: Folder) -> Result<(), MapError> {
        self.reset_to_root().await?;
        for segment in ["telecom", "msg", folder.as_str()] {
            self.setpath(segment).await?;
        }
        Ok(())
    }
}
