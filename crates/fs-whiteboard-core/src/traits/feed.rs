//! The change feed of one board.
//!
//! Pattern: Observer — a [`Subscription`] over the store's change feed, filtered by board. Dropping
//! it ends the subscription.

use async_trait::async_trait;
use fs_store::{Activity, ActivityKind, Envelope, ObjectId, StoredObject};
use tokio::sync::broadcast::{Receiver, error::RecvError};

use crate::error::Result;
use crate::unsupported::not_possible;

/// One change on a board, as in `common.capnp`.
#[derive(Clone, Debug, PartialEq)]
pub enum Change {
    /// An object was created.
    Created(Envelope),
    /// An object was changed.
    Updated(Envelope),
    /// An object was removed.
    Deleted(ObjectId),
}

/// The changes of one board. `next` answers `None` when the feed ended (the receiver fell too far
/// behind, or the store closed); changes may have been missed, so subscribe again and read the
/// current state, in that order.
pub struct Subscription {
    pub(crate) rx: Receiver<Activity>,
    pub(crate) board: ObjectId,
}

fn on_board(o: &StoredObject, board: ObjectId) -> bool {
    o.envelope.id == board
        || o.index
            .get("board")
            .is_some_and(|v| v.contains(&board.to_string()))
}

impl Subscription {
    /// The next change on this board; waits for it.
    pub async fn next(&mut self) -> Option<Change> {
        loop {
            let act = match self.rx.recv().await {
                Ok(a) => a,
                Err(RecvError::Lagged(_) | RecvError::Closed) => return None,
            };
            let Ok(set) = act.change_set() else { continue };
            let image = set.after.as_ref().or(set.before.as_ref());
            if !image.is_some_and(|o| on_board(o, self.board)) {
                continue;
            }
            return match (act.kind, set.after) {
                (ActivityKind::Create, Some(o)) => Some(Change::Created(o.envelope)),
                (_, Some(o)) => Some(Change::Updated(o.envelope)),
                (_, None) => Some(Change::Deleted(act.object)),
            };
        }
    }
}

/// Subscribing to a board. Interface status: draft (ADR 0045).
#[async_trait]
pub trait WhiteboardFeed: Send + Sync {
    /// `board.subscribe`: the changes of this board only.
    async fn subscribe(&self, _board: ObjectId) -> Result<Subscription> {
        not_possible("board.subscribe")
    }
}
