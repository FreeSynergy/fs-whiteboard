//! `reorder`: places elements directly behind another, keeping their order among themselves.
//!
//! Pattern: Command over order keys — new keys come from `order::spread` between the neighbours, so
//! nothing else is renumbered. Equal neighbour keys (two concurrent adds) cannot be split by a key;
//! then the element to go behind gets a new key after the moved ones, and that is recorded too.

use fs_store::ObjectId;
use serde_json::to_value;

use super::handle::Editor;
use super::selection::unique;
use crate::error::{Error, Result};
use crate::model::Element;
use crate::order;
use crate::repo::tx::Tx;
use crate::unit::Unit;

fn sorted(mut v: Vec<&Element>) -> Vec<&Element> {
    v.sort_by(|a, b| (a.order.as_bytes(), a.id).cmp(&(b.order.as_bytes(), b.id)));
    v
}

impl Editor {
    pub(super) async fn reorder(&self, ids: &[ObjectId], before: Option<ObjectId>) -> Result<()> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        let ids = unique(ids);
        let mut pages = Vec::new();
        for id in &ids {
            pages.push(tx.editable(st, *id)?.page);
        }
        pages.dedup();
        let Some(&page) = pages.first() else {
            return Ok(());
        };
        let anchor = match before {
            Some(b) if ids.contains(&b) => return Err(Error::invalid("cannot go behind itself")),
            Some(b) => Some(tx.editable(st, b)?.page)
                .filter(|p| *p == page)
                .map(|_| b),
            None => None,
        };
        if (before.is_some() && anchor.is_none()) || pages.iter().any(|p| *p != page) {
            return Err(Error::invalid(
                "the elements and the one to go behind must share a page",
            ));
        }
        let moved = sorted(ids.iter().map(|id| &st.elements[id]).collect());
        let others = sorted(
            st.elements
                .values()
                .filter(|e| e.page == page && !ids.contains(&e.id))
                .collect(),
        );
        let pos = anchor
            .and_then(|b| others.iter().position(|e| e.id == b))
            .unwrap_or(others.len());
        let lo = pos.checked_sub(1).map(|i| others[i].order.clone());
        let hi = others.get(pos).map(|e| e.order.clone());
        let n = moved.len();
        let mut changes: Vec<(ObjectId, String)> = Vec::new();
        if lo.is_some() && lo >= hi && hi.is_some() {
            let upper = others[pos..]
                .iter()
                .find(|e| Some(&e.order) > hi.as_ref())
                .map(|e| e.order.clone());
            let keys = order::spread(hi.as_deref(), upper.as_deref(), n + 1);
            changes.extend(moved.iter().map(|e| e.id).zip(keys.iter().cloned()));
            changes.extend(anchor.zip(keys.last().cloned()));
        } else {
            let keys = order::spread(lo.as_deref(), hi.as_deref(), n);
            changes.extend(moved.iter().map(|e| e.id).zip(keys));
        }
        for (id, key) in changes {
            tx.set(st, id, vec![(Unit::Order, to_value(key)?)])?;
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }
}
