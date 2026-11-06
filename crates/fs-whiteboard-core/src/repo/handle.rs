//! [`WhiteboardRepo`]: the store-backed implementation of the whiteboard traits.
//!
//! Pattern: Repository + Facade — one small type hides the object store, the merge rules and the
//! in-memory state of the boards behind `Boards`, `BoardEditor` and `WhiteboardFeed`. Clones share
//! one state per board, which all editors of a board use. Use one repository per store: a second one
//! would not see this one's cache.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use fs_store::{ObjectId, Store};
use tokio::sync::{Mutex, MutexGuard};

use super::state::BoardState;
use crate::error::{Error, Result};

pub(super) type Boards = HashMap<ObjectId, BoardState>;

/// Whiteboards over an object store. Cheap to clone; clones share the state of the boards.
#[derive(Clone)]
pub struct WhiteboardRepo {
    pub(super) store: Arc<dyn Store>,
    pub(super) actor: String,
    pub(super) instance: String,
    pub(super) page_size: usize,
    pub(super) boards: Arc<Mutex<Boards>>,
}

impl WhiteboardRepo {
    /// Whiteboards over a whole [`Store`]. `actor` is the IRI of the acting person / program: it
    /// goes into the log and into the clocks of every change.
    pub fn new(store: Arc<dyn Store>, actor: impl Into<String>) -> Self {
        Self {
            store,
            actor: actor.into(),
            instance: "local.invalid".into(),
            page_size: 100,
            boards: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Instance domain for external IRIs `https://<instance>/whiteboard/<id>` (ADR 0027).
    #[must_use]
    pub fn with_instance(mut self, instance: impl Into<String>) -> Self {
        self.instance = instance.into();
        self
    }

    /// Items per page of `list` and `elements` (default 100, at least 1).
    #[must_use]
    pub fn with_page_size(mut self, n: usize) -> Self {
        self.page_size = n.max(1);
        self
    }

    /// The same repository acting as another person: shared store and shared board state, but the
    /// clocks and the log carry `actor`. One editor per person, all on one state.
    #[must_use]
    pub fn as_actor(&self, actor: impl Into<String>) -> Self {
        Self {
            actor: actor.into(),
            ..self.clone()
        }
    }

    pub(super) fn iri(&self, id: ObjectId) -> String {
        crate::doc::iri(&self.instance, id)
    }

    /// The state of a board, loaded from the store the first time.
    pub(super) async fn state<'a>(
        &self,
        guard: &'a mut MutexGuard<'_, Boards>,
        id: ObjectId,
    ) -> Result<&'a mut BoardState> {
        match guard.entry(id) {
            Entry::Occupied(o) => Ok(o.into_mut()),
            Entry::Vacant(v) => {
                let st = self
                    .load(id)
                    .await?
                    .ok_or(Error::NotFound { what: "board", id })?;
                Ok(v.insert(st))
            }
        }
    }
}
