//! Reading and writing the store objects of boards and elements.
//!
//! Pattern: Repository — the only place that talks to the store about this program's objects.
//! Elements are written with the logged `put`, so the activity log has every change.

use fs_store::{Envelope, ObjectId, Query, StoredObject};

use super::handle::WhiteboardRepo;
use super::state::BoardState;
use crate::consts::{BOARD_CONTEXT, BOARD_TYPE, ELEMENT_TYPE, TOMBSTONE_TYPE};
use crate::doc::{
    BoardDoc, ElementDoc, board_to_stored, element_to_stored, stored_to_board, stored_to_element,
};
use crate::error::Result;

impl WhiteboardRepo {
    /// The board with all its elements, if there is such a board.
    pub(super) async fn load(&self, id: ObjectId) -> Result<Option<BoardState>> {
        let Some(obj) = self.store.get(id).await? else {
            return Ok(None);
        };
        if obj.envelope.object_type != BOARD_TYPE
            || obj.envelope.context.as_deref() != Some(BOARD_CONTEXT)
        {
            return Ok(None);
        }
        let mut st = BoardState::new(id, stored_to_board(&obj)?);
        let q = Query::new()
            .of_type(ELEMENT_TYPE)
            .with_index("board", id.to_string());
        for o in self.store.query(&q).await? {
            // one bad object must not stop the others from loading: skip it and report it
            match stored_to_element(&o)
                .and_then(|d| crate::validate::element(&d.element).map(|()| d))
            {
                Ok(d) if d.element.id == o.envelope.id => st.put(d),
                Ok(_) => self.report(o.envelope.id, "the envelope id differs from the content id"),
                Err(e) => self.report(o.envelope.id, &e.to_string()),
            }
        }
        Ok(Some(st))
    }

    pub(super) async fn save_element(&self, doc: &ElementDoc) -> Result<()> {
        let o = element_to_stored(doc, &self.instance, None)?;
        self.store.put(o, &self.actor).await?;
        Ok(())
    }

    pub(super) async fn save_board(&self, id: ObjectId, doc: &BoardDoc) -> Result<()> {
        let o = board_to_stored(doc, id, &self.instance, &self.actor, None)?;
        self.store.put(o, &self.actor).await?;
        Ok(())
    }
}

/// True for the marker a deleted board leaves behind.
pub(super) fn is_marker(o: &StoredObject) -> bool {
    o.envelope.object_type == TOMBSTONE_TYPE
}

impl WhiteboardRepo {
    /// Leaves the marker of a deleted board (same id, so a late remote finds it).
    pub(super) async fn mark_deleted(&self, id: ObjectId) -> Result<()> {
        let mut env = Envelope::new(TOMBSTONE_TYPE, "deleted board");
        env.id = id;
        env.context = Some(format!("{BOARD_CONTEXT}-tombstone"));
        let o = StoredObject::new(env, "application/json", serde_json::json!({}));
        self.store.put(o, &self.actor).await?;
        Ok(())
    }

    /// Remembers that an object could not be read: once per object, at most 100 objects.
    pub(super) fn report(&self, id: ObjectId, why: &str) {
        if let Ok(mut p) = self.problems.lock() {
            if let Some(e) = p.iter_mut().find(|(i, _)| *i == id) {
                e.1 = why.to_string();
                return;
            }
            if p.len() >= 100 {
                p.remove(0);
            }
            p.push((id, why.to_string()));
        }
    }

    /// An object that reads again is no problem any more.
    pub(super) fn clear_problem(&self, id: ObjectId) {
        if let Ok(mut p) = self.problems.lock() {
            p.retain(|(i, _)| *i != id);
        }
    }

    /// Removes a board with every element of it, and leaves the marker. Caller holds the lock.
    pub(super) async fn purge(
        &self,
        guard: &mut super::handle::Boards,
        id: ObjectId,
    ) -> Result<()> {
        let q = Query::new()
            .of_type(ELEMENT_TYPE)
            .with_index("board", id.to_string());
        for o in self.store.query(&q).await? {
            self.store.delete(o.envelope.id, &self.actor).await?;
        }
        if self.store.get(id).await?.is_some_and(|o| !is_marker(&o)) {
            self.store.delete(id, &self.actor).await?;
        }
        self.mark_deleted(id).await?;
        guard.remove(&id);
        self.clear_problem(id);
        Ok(())
    }

    /// Objects that were skipped because they could not be read (id and reason), oldest first.
    pub fn problems(&self) -> Vec<(ObjectId, String)> {
        self.problems.lock().map(|p| p.clone()).unwrap_or_default()
    }
}
