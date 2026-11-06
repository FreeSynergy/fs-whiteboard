//! `export`: the own format or SVG, of a page, a selection or the whole board.
//!
//! Pattern: Strategy per media type — this file only gathers what is shown (the reading rules
//! applied) into a `Scene`; the format writes it.

use std::collections::BTreeSet;

use fs_interfaces::Unsupported;
use fs_store::{Blob, ObjectId};

use super::handle::Editor;
use super::selection::with_members;
use crate::error::{Error, Result};
use crate::format::{self, Scene};
use crate::model::Element;
use crate::reading::effective_pages;

impl Editor {
    pub(super) async fn export(
        &self,
        page: Option<ObjectId>,
        ids: &[ObjectId],
        media_type: &str,
    ) -> Result<Blob> {
        let Some(fmt) = format::find(media_type) else {
            return Err(
                Unsupported::new("board.export", format!("no export as {media_type}")).into(),
            );
        };
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        if page.is_none() && (!ids.is_empty() || !fmt.whole_board()) {
            return Err(Error::invalid("this export needs a page"));
        }
        if let Some(p) = page.filter(|p| !st.doc.pages.contains_key(p)) {
            return Err(Error::Foreign(p));
        }
        let view = st.view();
        let picked: BTreeSet<ObjectId> = if ids.is_empty() {
            st.elements.keys().copied().collect()
        } else {
            for id in ids {
                if !st.elements.contains_key(id) {
                    return Err(Error::Foreign(*id));
                }
            }
            with_members(&view.parents, ids)
        };
        let mut elements: Vec<Element> = picked
            .iter()
            .map(|id| &st.elements[id])
            .filter(|e| !view.hidden.contains(&e.id) && page.is_none_or(|p| e.page == p))
            .map(|e| view.shown(e))
            .collect();
        elements.sort_by(|a, b| (a.order.as_bytes(), a.id).cmp(&(b.order.as_bytes(), b.id)));
        let pages = effective_pages(&st.pages())
            .into_iter()
            .filter(|p| !p.deleted && page.is_none_or(|x| p.id == x))
            .collect();
        let scene = Scene {
            name: st.doc.name.clone(),
            pages,
            elements,
        };
        Ok(Blob {
            media_type: fmt.media_type().to_string(),
            bytes: fmt.export(&scene)?,
        })
    }
}
