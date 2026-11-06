//! Grouping: `group`, `ungroup`, `set_parent`.
//!
//! Pattern: Composite — a group is an element whose `parent` it is for its members. A cycle is
//! refused here against the parents as they will be after this edit; concurrent changes that form
//! one anyway are broken at reading.

use std::collections::BTreeSet;

use fs_store::ObjectId;
use serde_json::{Value, to_value};

use super::add::front_key;
use super::handle::Editor;
use super::selection::{Parents, is_within, outermost, unique, with_members};
use crate::error::{Error, Result};
use crate::geometry::union_all;
use crate::model::{Element, Kind, Rect};
use crate::repo::tx::Tx;
use crate::unit::Unit;

fn parent_value(p: Option<ObjectId>) -> Value {
    to_value(p).unwrap_or_default()
}

impl Editor {
    pub(super) async fn group(&self, ids: &[ObjectId]) -> Result<ObjectId> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        let mut tx = Tx::new(&self.repo.actor);
        let ids = outermost(&view.parents, &unique(ids));
        let first = *ids
            .first()
            .ok_or_else(|| Error::invalid("nothing to group"))?;
        let page = tx.editable(st, first)?.page;
        let mut rects: Vec<Rect> = Vec::new();
        for id in &ids {
            let e = tx.editable(st, *id)?;
            if e.page != page {
                return Err(Error::invalid("the elements are on different pages"));
            }
            rects.push(view.bounds_of(e));
        }
        let parent =
            view.parents[&first].filter(|p| ids.iter().all(|i| view.parents[i] == Some(*p)));
        let frame = union_all(&rects).unwrap_or_default();
        let mut group = Element::new(page, Kind::Group, frame);
        let id = ObjectId::new();
        (group.id, group.board, group.parent, group.order) =
            (id, self.board, parent, front_key(st));
        if let Some(p) = parent.filter(|p| ids.iter().any(|i| is_within(&view.parents, *p, *i))) {
            return Err(Error::Cycle {
                child: id,
                parent: p,
            });
        }
        tx.create(group)?;
        for m in &ids {
            tx.set(st, *m, vec![(Unit::Parent, parent_value(Some(id)))])?;
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(id)
    }

    pub(super) async fn ungroup(&self, id: ObjectId) -> Result<Vec<ObjectId>> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        let mut tx = Tx::new(&self.repo.actor);
        if !matches!(tx.editable(st, id)?.kind, Kind::Group) {
            return Err(Error::invalid("only a group can be ungrouped"));
        }
        let up = view.parents[&id];
        let members: Vec<ObjectId> = view
            .parents
            .iter()
            .filter(|(_, p)| **p == Some(id))
            .map(|(m, _)| *m)
            .collect();
        for m in &members {
            let e = tx.el(st, *m)?;
            if e.locked && !e.deleted {
                return Err(Error::Locked(*m));
            }
            tx.set(st, *m, vec![(Unit::Parent, parent_value(up))])?;
        }
        tx.set(st, id, vec![(Unit::Deleted, Value::Bool(true))])?;
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(members
            .into_iter()
            .filter(|m| !st.elements[m].deleted)
            .collect())
    }

    pub(super) async fn set_parent(
        &self,
        ids: &[ObjectId],
        page: Option<ObjectId>,
        parent: Option<ObjectId>,
    ) -> Result<()> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        let mut tx = Tx::new(&self.repo.actor);
        let ids = unique(ids);
        let target = match parent {
            Some(p) => {
                let e = tx.el(st, p)?;
                if !e.kind.is_container() || e.deleted {
                    return Err(Error::invalid("the parent must be a group or frame"));
                }
                if page.is_some_and(|pg| pg != e.page) {
                    return Err(Error::invalid("the parent is on another page"));
                }
                Some(e.page)
            }
            None => page,
        };
        if let Some(pg) = target.filter(|pg| !view.live_pages.contains(pg)) {
            return Err(Error::NotFound {
                what: "page",
                id: pg,
            });
        }
        let mut parents: Parents = view.parents.clone();
        for id in &ids {
            tx.editable(st, *id)?;
            if let Some(p) = parent.filter(|p| is_within(&parents, *p, *id)) {
                return Err(Error::Cycle {
                    child: *id,
                    parent: p,
                });
            }
            parents.insert(*id, parent);
            tx.set(st, *id, vec![(Unit::Parent, parent_value(parent))])?;
        }
        if let Some(pg) = target {
            let moving: BTreeSet<ObjectId> = with_members(&view.parents, &ids);
            for m in moving {
                let e = tx.el(st, m)?;
                if e.page != pg && e.locked && !e.deleted {
                    return Err(Error::Locked(m));
                }
                if tx.el(st, m)?.page != pg {
                    tx.set(st, m, vec![(Unit::Page, to_value(pg)?)])?;
                }
            }
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }
}
