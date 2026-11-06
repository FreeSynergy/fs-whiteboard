//! `apply_remote`: writes the merge of a remote object with the local one.
//!
//! Pattern: Adapter (for a later sync adapter) — `merge` decides, this validates what comes in like a
//! local entry, writes the result and brings the state and the spatial index along. The local object
//! is read under the board lock. A deleted board leaves a marker, so a late remote cannot bring it
//! back. `fs-hub-client::sync_once` overwrites objects, so it must not feed this store.

use fs_store::StoredObject;

use super::handle::WhiteboardRepo;
use super::io::is_marker;
use crate::consts::{BOARD_CONTEXT, BOARD_TYPE, ELEMENT_TYPE, TOMBSTONE_TYPE};
use crate::doc::{board_to_stored, stored_to_board, stored_to_element};
use crate::error::{Error, Result};
use crate::merge::{merge_board, merge_element};
use crate::validate;

impl WhiteboardRepo {
    /// Takes a remote version of a board or an element in. The result is the merge of both: per
    /// property, the larger clock wins. Applying the same object again, or in another order, changes
    /// nothing. Refused: an envelope id that differs from the content id, other identity, a clock
    /// counter above `u64::MAX / 2`, a board without page, bad numbers, limits or keys.
    pub async fn apply_remote(&self, remote: StoredObject) -> Result<()> {
        let id = remote.envelope.id;
        let mut guard = self.boards.lock().await;
        let mut local = self.store.get(id).await?;
        // this program's objects only: the incoming one, the one it would replace or delete
        let kind =
            Kind::of(&remote).ok_or_else(|| Error::invalid("not an object of this program"))?;
        if local.as_ref().is_some_and(is_marker) {
            return Ok(());
        }
        if let Some(l) = &local {
            let same = Kind::of(l) == Some(kind);
            let deletes_board = kind == Kind::Marker && Kind::of(l) == Some(Kind::Board);
            if !(same || deletes_board) {
                return Err(Error::invalid("the id belongs to another object"));
            }
        }
        if remote.envelope.object_type == TOMBSTONE_TYPE {
            // the board was deleted elsewhere: it goes here too
            return self.purge(&mut guard, id).await;
        }
        // a broken local object is replaced by a valid remote one
        if local.as_ref().is_some_and(|l| !Self::readable(l)) {
            local = None;
        }
        match remote.envelope.object_type.as_str() {
            ELEMENT_TYPE => {
                let rdoc = stored_to_element(&remote)?;
                if rdoc.element.id != id {
                    return Err(Error::invalid(
                        "the envelope id differs from the content id",
                    ));
                }
                validate::element_doc(&rdoc)?;
                let board = rdoc.element.board;
                if self.store.get(board).await?.as_ref().is_some_and(is_marker) {
                    return Ok(());
                }
                let doc = match &local {
                    Some(l) => merge_element(&stored_to_element(l)?, &rdoc)?,
                    None => rdoc,
                };
                if local
                    .as_ref()
                    .is_none_or(|l| l.content != serde_json::to_value(&doc).unwrap_or_default())
                {
                    self.save_element(&doc).await?;
                }
                if let Some(st) = guard.get_mut(&board) {
                    st.put(doc);
                }
            }
            BOARD_TYPE => {
                let rdoc = stored_to_board(&remote)?;
                validate::board_doc(&rdoc)?;
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
        self.clear_problem(id);
        Ok(())
    }

    /// True if a stored object of this program reads and passes the checks.
    fn readable(o: &StoredObject) -> bool {
        match o.envelope.object_type.as_str() {
            ELEMENT_TYPE => stored_to_element(o).is_ok_and(|d| {
                d.element.id == o.envelope.id && validate::element(&d.element).is_ok()
            }),
            _ => stored_to_board(o).is_ok(),
        }
    }
}

/// Which of this program's kinds of object a stored object is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Element,
    Board,
    Marker,
}

impl Kind {
    fn of(o: &StoredObject) -> Option<Kind> {
        let ctx = o.envelope.context.as_deref();
        match o.envelope.object_type.as_str() {
            ELEMENT_TYPE => Some(Kind::Element),
            BOARD_TYPE if ctx == Some(BOARD_CONTEXT) => Some(Kind::Board),
            TOMBSTONE_TYPE if ctx == Some(&format!("{BOARD_CONTEXT}-tombstone")[..]) => {
                Some(Kind::Marker)
            }
            _ => None,
        }
    }
}
