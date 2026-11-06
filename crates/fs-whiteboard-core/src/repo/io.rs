//! Reading and writing the store objects of boards and elements.
//!
//! Pattern: Repository — the only place that talks to the store about this program's objects.
//! Elements are written with the logged `put`, so the activity log has every change.

use fs_store::{ObjectId, Query};

use super::handle::WhiteboardRepo;
use super::state::BoardState;
use crate::consts::{BOARD_CONTEXT, BOARD_TYPE, ELEMENT_TYPE};
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
            st.put(stored_to_element(&o)?);
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
