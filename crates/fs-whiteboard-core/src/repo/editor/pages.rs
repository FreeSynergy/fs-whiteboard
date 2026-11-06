//! Pages: `add_page`, `remove_page`, `move_page`.
//!
//! Pattern: Command on the board document — a page is never dropped; removing sets `deleted`, which
//! merges. Page changes are not part of undo (the contract lists undo for elements).

use fs_store::ObjectId;

use super::handle::Editor;
use crate::error::{Error, Result};
use crate::model::{Board, Page};
use crate::order;
use crate::reading::effective_pages;
use crate::validate;

impl Editor {
    pub(super) async fn add_page(&self, name: &str) -> Result<(Board, ObjectId)> {
        validate::text(name)?;
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let last = effective_pages(&st.pages()).last().map(|p| p.order.clone());
        let page = Page {
            id: ObjectId::new(),
            name: name.to_string(),
            order: order::between(last.as_deref(), None),
            deleted: false,
        };
        let id = page.id;
        let mut doc = st.doc.clone();
        doc.add_page(page, &self.repo.actor);
        self.repo.save_board(self.board, &doc).await?;
        st.set_doc(doc);
        Ok((st.board(&self.repo.iri(self.board)), id))
    }

    pub(super) async fn remove_page(&self, page: ObjectId) -> Result<Board> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let pages = effective_pages(&st.pages());
        let live: Vec<&Page> = pages.iter().filter(|p| !p.deleted).collect();
        if !st.doc.pages.contains_key(&page) {
            return Err(Error::Foreign(page));
        }
        if live.iter().all(|p| p.id == page) {
            return Err(Error::LastPage);
        }
        let mut doc = st.doc.clone();
        doc.edit_page(page, "deleted", &self.repo.actor, |p| p.deleted = true);
        self.repo.save_board(self.board, &doc).await?;
        st.set_doc(doc);
        Ok(st.board(&self.repo.iri(self.board)))
    }

    pub(super) async fn move_page(
        &self,
        page: ObjectId,
        before: Option<ObjectId>,
    ) -> Result<Board> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        for id in std::iter::once(page).chain(before) {
            if !st.doc.pages.contains_key(&id) {
                return Err(Error::Foreign(id));
            }
        }
        let others: Vec<Page> = effective_pages(&st.pages())
            .into_iter()
            .filter(|p| p.id != page)
            .collect();
        let pos = before
            .and_then(|b| others.iter().position(|p| p.id == b))
            .unwrap_or(others.len());
        let lo = pos.checked_sub(1).map(|i| others[i].order.clone());
        let key = order::between(lo.as_deref(), others.get(pos).map(|p| p.order.as_str()));
        let mut doc = st.doc.clone();
        doc.edit_page(page, "order", &self.repo.actor, |p| p.order = key);
        self.repo.save_board(self.board, &doc).await?;
        st.set_doc(doc);
        Ok(st.board(&self.repo.iri(self.board)))
    }
}
