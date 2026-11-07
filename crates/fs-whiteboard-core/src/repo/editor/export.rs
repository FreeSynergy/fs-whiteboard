//! `export`: the own format or SVG, of a page, a selection or the whole board.
//!
//! Pattern: Strategy per media type — this file only gathers what is shown (the reading rules
//! applied) into a `Scene`; the format writes it.

use std::collections::BTreeSet;

use fs_interfaces::Unsupported;
use fs_objects::{Blob, ObjectId};

use super::handle::Editor;
use super::selection::with_members;
use crate::error::{Error, Result};
use crate::format::{self, Scene};
use crate::model::{Anchor, Element, Kind};
use crate::reading::effective_pages;
use crate::validate;

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
        // a reference to something that is not exported becomes a free end at its position, and a
        // parent outside the set is none, so every export can be imported again
        let inside: BTreeSet<ObjectId> = elements.iter().map(|e| e.id).collect();
        for e in &mut elements {
            e.parent = e.parent.filter(|p| inside.contains(p));
            if let (Kind::Connector(c), Some((a, b))) =
                (&mut e.kind, view.connector_ends.get(&e.id))
            {
                for (anchor, at) in [(&mut c.source, a), (&mut c.target, b)] {
                    if anchor.element.is_some_and(|x| !inside.contains(&x)) {
                        *anchor = Anchor {
                            element: None,
                            point: *at,
                        };
                    }
                }
            }
        }
        // a connector is written with its stored frame (its place is in its ends); what cannot be
        // imported again by the same checks is not written
        for e in &mut elements {
            if let Some(stored) = st
                .elements
                .get(&e.id)
                .filter(|_| matches!(e.kind, Kind::Connector(_)))
            {
                (e.frame, e.angle) = (stored.frame, stored.angle);
            }
            validate::remote_element(e)?;
        }
        let ends = view
            .connector_ends
            .iter()
            .filter(|(id, _)| inside.contains(id))
            .map(|(k, v)| (*k, *v))
            .collect();
        let pages = effective_pages(&st.pages())
            .into_iter()
            .filter(|p| !p.deleted && page.is_none_or(|x| p.id == x))
            .collect();
        let scene = Scene {
            name: st.doc.name.clone(),
            pages,
            elements,
            ends,
        };
        Ok(Blob {
            media_type: fmt.media_type().to_string(),
            bytes: fmt.export(&scene)?,
        })
    }
}
