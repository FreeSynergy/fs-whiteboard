//! `apply_remote`: writes the merge of a remote object with the local one.
//!
//! Pattern: Adapter (for a later sync adapter) — `merge` decides, this writes the result and brings
//! the state and the spatial index along. `fs-hub-client::sync_once` overwrites objects, so it must not
//! feed this store; the hub delivers only other programs' activities anyway.

use fs_store::StoredObject;

use super::handle::WhiteboardRepo;
use crate::consts::{BOARD_TYPE, ELEMENT_TYPE};
use crate::doc::{board_to_stored, stored_to_board, stored_to_element};
use crate::error::{Error, Result};
use crate::merge::{merge_board, merge_element};
use crate::validate;

impl WhiteboardRepo {
    /// Takes a remote version of a board or an element in. The result is the merge of both: per
    /// property, the larger clock wins. Applying the same object again, or in another order, changes
    /// nothing.
    pub async fn apply_remote(&self, remote: StoredObject) -> Result<()> {
        let id = remote.envelope.id;
        let local = self.store.get(id).await?;
        let mut guard = self.boards.lock().await;
        match remote.envelope.object_type.as_str() {
            ELEMENT_TYPE => {
                let rdoc = stored_to_element(&remote)?;
                validate::element(&rdoc.element)?;
                let doc = match &local {
                    Some(l) => merge_element(&stored_to_element(l)?, &rdoc),
                    None => rdoc,
                };
                if local
                    .as_ref()
                    .is_none_or(|l| l.content != serde_json::to_value(&doc).unwrap_or_default())
                {
                    self.save_element(&doc).await?;
                }
                if let Some(st) = guard.get_mut(&doc.element.board) {
                    st.put(doc);
                }
            }
            BOARD_TYPE => {
                let rdoc = stored_to_board(&remote)?;
                let doc = match &local {
                    Some(l) => merge_board(&stored_to_board(l)?, &rdoc),
                    None => rdoc,
                };
                if local
                    .as_ref()
                    .is_none_or(|l| l.content != serde_json::to_value(&doc).unwrap_or_default())
                {
                    let o = board_to_stored(&doc, id, &self.instance, &self.actor, local.as_ref())?;
                    self.store.put(o, &self.actor).await?;
                }
                if let Some(st) = guard.get_mut(&id) {
                    st.set_doc(doc);
                }
            }
            other => {
                return Err(Error::invalid(format!(
                    "{other} is not an object of this program"
                )));
            }
        }
        Ok(())
    }
}
