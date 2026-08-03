//! Live (non-opted-in) MAP op dispatch — split out of `super` to keep that module under the
//! size ceiling.

use ipc::BrokerResponse;
use map_core::client::MapClient;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;

use super::super::{dispatch, Actor, OpOutcome};
use crate::runtime::types::DeviceOp;

/// The `Live*`/`Backfill` [`DeviceOp`] variants. Narrowing conversion target for
/// [`Actor::handle_live_op`] — turns a routing mistake into a logged no-op instead of a panic.
pub(super) enum LiveOp {
    MarkRead {
        handle: String,
        reply: oneshot::Sender<BrokerResponse>,
    },
    Send {
        number: String,
        message: String,
        reply: oneshot::Sender<BrokerResponse>,
    },
    Backfill {
        reply: oneshot::Sender<BrokerResponse>,
    },
    List {
        folder: Option<String>,
        unread: bool,
        from: Option<String>,
        since: Option<String>,
        limit: Option<u16>,
        offset: u16,
        reply: oneshot::Sender<BrokerResponse>,
    },
    Get {
        handle: String,
        reply: oneshot::Sender<BrokerResponse>,
    },
    Threads {
        reply: oneshot::Sender<BrokerResponse>,
    },
    Folders {
        reply: oneshot::Sender<BrokerResponse>,
    },
}

impl TryFrom<DeviceOp> for LiveOp {
    type Error = DeviceOp;

    fn try_from(op: DeviceOp) -> Result<Self, DeviceOp> {
        Ok(match op {
            DeviceOp::LiveMarkRead { handle, reply } => Self::MarkRead { handle, reply },
            DeviceOp::LiveSend { number, message, reply } => Self::Send { number, message, reply },
            DeviceOp::Backfill { reply } => Self::Backfill { reply },
            DeviceOp::LiveList { folder, unread, from, since, limit, offset, reply } => {
                Self::List { folder, unread, from, since, limit, offset, reply }
            }
            DeviceOp::LiveGet { handle, reply } => Self::Get { handle, reply },
            DeviceOp::LiveThreads { reply } => Self::Threads { reply },
            DeviceOp::LiveFolders { reply } => Self::Folders { reply },
            other => return Err(other),
        })
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Actor<T> {
    /// The `Live*`/`Backfill` [`DeviceOp`]s — split out of
    /// [`handle_op`][super::Actor::handle_op] to keep it under the size ceiling. A non-matching
    /// `op` (only reachable if `handle_op`'s match stops covering every other variant) is
    /// logged and dropped rather than panicking, same as
    /// [`handle_pbap_op`][super::Actor::handle_pbap_op].
    // `&self` would compile (no field is mutated), but `Actor<T>` holds `Box<dyn FnMut() + Send>`
    // connector fields that aren't `Sync` — a shared `&Actor<T>` held across an `.await` inside
    // the actor's spawned task would then need `Actor<T>: Sync`, which it isn't. `&mut self`
    // sidesteps that: exclusive references don't require the referent to be `Sync`.
    #[allow(clippy::needless_pass_by_ref_mut)]
    pub(super) async fn handle_live_op(
        &mut self,
        client: &mut MapClient<T>,
        op: DeviceOp,
    ) -> OpOutcome {
        let Ok(op) = LiveOp::try_from(op) else {
            tracing::error!("handle_live_op called with an unrouted DeviceOp — dropping");
            return OpOutcome::Continue;
        };
        match op {
            LiveOp::MarkRead { handle, reply } => {
                Self::finish_map(dispatch::do_live_mark_read(client, handle).await, reply)
            }
            LiveOp::Send { number, message, reply } => {
                Self::finish_map(dispatch::do_live_send(client, number, message).await, reply)
            }
            LiveOp::Backfill { reply } => {
                Self::finish_map(dispatch::do_backfill(client, &self.store).await, reply)
            }
            LiveOp::List { folder, unread, from, since, limit, offset, reply } => Self::finish_map(
                dispatch::do_live_list(client, folder, unread, from, since, limit, offset).await,
                reply,
            ),
            LiveOp::Get { handle, reply } => {
                Self::finish_map(dispatch::do_live_get(client, handle).await, reply)
            }
            LiveOp::Threads { reply } => {
                Self::finish_map(dispatch::do_live_threads(client).await, reply)
            }
            LiveOp::Folders { reply } => {
                Self::finish_map(dispatch::do_live_folders(client).await, reply)
            }
        }
    }
}
