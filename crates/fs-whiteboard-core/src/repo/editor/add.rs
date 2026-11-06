//! `add`: a new element at the front of its page.
//!
//! Pattern: Factory — the id, the board, the order and `deleted` are decided here, whatever the
//! caller put in; everything else is checked first, so nothing unusable reaches the store.

use fs_store::ObjectId;

use super::handle::Editor;
use crate::error::{Error, Result};
use crate::model::{Element, Kind};
use crate::order;
use crate::reading::View;
use crate::repo::state::BoardState;
use crate::repo::tx::Tx;
use crate::validate;

/// The largest order key among the elements that are shown (a hidden one cannot block the front),
/// or `None` if there is none.
pub(super) fn highest_key(st: &BoardState, view: &View) -> Option<String> {
    st.elements
        .values()
        .filter(|e| !view.hidden.contains(&e.id))
        .map(|e| (e.order.as_bytes(), e.id))
        .max()
        .and_then(|(_, id)| st.elements.get(&id).map(|e| e.order.clone()))
}

/// A key in front of every element of the board.
pub(super) fn front_key(st: &BoardState, view: &View) -> String {
    order::between(highest_key(st, view).as_deref(), None)
}

impl Editor {
    pub(super) async fn add(&self, element: Element) -> Result<Element> {
        validate::element(&element)?;
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        if !view.live_pages.contains(&element.page) {
            return Err(Error::NotFound {
                what: "page",
                id: element.page,
            });
        }
        if let Some(p) = element.parent {
            let parent = st.elements.get(&p).ok_or(Error::Foreign(p))?;
            if !parent.kind.is_container() || parent.deleted || parent.page != element.page {
                return Err(Error::invalid(
                    "the parent must be a group or frame on the same page",
                ));
            }
        }
        if let Kind::Connector(c) = &element.kind {
            for id in [c.source.element, c.target.element].into_iter().flatten() {
                if !st.elements.contains_key(&id) {
                    return Err(Error::Foreign(id));
                }
            }
        }
        let id = ObjectId::new();
        let new = Element {
            id,
            board: self.board,
            order: front_key(st, &view),
            deleted: false,
            ..element
        };
        let mut tx = Tx::new(&self.repo.actor);
        tx.create(new)?;
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        let view = st.view();
        st.elements
            .get(&id)
            .map(|e| view.shown(e))
            .ok_or(Error::Foreign(id))
    }
}
