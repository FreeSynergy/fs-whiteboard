//! Reading: the board, one element, a page of elements, bounds, the ends of a connector.
//!
//! Pattern: Iterator (cursor paging) over the spatial index — candidates from the R-tree, then the
//! reading rules, then front-to-back order. The cursor is the last `(order, id)` seen, so edits
//! between pages lose or repeat nothing.

use std::collections::BTreeSet;

use fs_store::ObjectId;

use super::handle::Editor;
use crate::error::{Error, Result};
use crate::geometry::union_all;
use crate::model::{Board, Cursor, Element, Paged, Rect};
use crate::validate;

fn key(e: &Element) -> (&[u8], ObjectId) {
    (e.order.as_bytes(), e.id)
}

fn parse(c: &Cursor) -> Option<(String, ObjectId)> {
    let (order, id) = c.0.rsplit_once(':')?;
    Some((order.to_string(), id.parse().ok()?))
}

impl Editor {
    pub(super) async fn get(&self) -> Result<Board> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        Ok(st.board(&self.repo.iri(self.board)))
    }

    pub(super) async fn get_element(&self, id: ObjectId) -> Result<Option<Element>> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        Ok(st.elements.get(&id).map(|e| view.shown(e)))
    }

    pub(super) async fn elements(
        &self,
        page: ObjectId,
        area: Option<Rect>,
        cursor: Option<&Cursor>,
    ) -> Result<Paged<Element>> {
        if let Some(a) = &area {
            validate::rect(a)?;
        }
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        if !st.doc.pages.contains_key(&page) {
            return Err(Error::Foreign(page));
        }
        let view = st.view();
        let mut ids: BTreeSet<ObjectId> = st.index.query(page, area.as_ref()).into_iter().collect();
        for id in st.groups.iter().chain(&st.connectors) {
            let e = &st.elements[id];
            if e.page == page && area.is_none_or(|a| view.bounds_of(e).intersects(&a)) {
                ids.insert(*id);
            }
        }
        let mut found: Vec<&Element> = ids
            .iter()
            .filter(|id| !view.hidden.contains(id))
            .map(|id| &st.elements[id])
            .collect();
        found.sort_by(|a, b| key(a).cmp(&key(b)));
        if let Some((order, id)) = cursor.and_then(parse) {
            found.retain(|e| key(e) > (order.as_bytes(), id));
        }
        let more = found.len() > self.repo.page_size;
        found.truncate(self.repo.page_size);
        let next = more
            .then(|| {
                found
                    .last()
                    .map(|e| Cursor(format!("{}:{}", e.order, e.id)))
            })
            .flatten();
        Ok(Paged {
            items: found.into_iter().map(|e| view.shown(e)).collect(),
            next,
        })
    }

    pub(super) async fn bounds(&self, page: ObjectId) -> Result<Option<Rect>> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        if !st.doc.pages.contains_key(&page) {
            return Err(Error::Foreign(page));
        }
        let view = st.view();
        let rects: Vec<Rect> = st
            .index
            .query(page, None)
            .into_iter()
            .filter(|id| !view.hidden.contains(id))
            .filter_map(|id| st.index.rect_of(id))
            .chain(
                st.connectors
                    .iter()
                    .filter(|id| !view.hidden.contains(id) && st.elements[id].page == page)
                    .map(|id| view.bounds_of(&st.elements[id])),
            )
            .collect();
        Ok(union_all(&rects))
    }
}
