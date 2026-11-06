//! `copy`: copies with their members, connectors between copies pointing at the copies.
//!
//! Pattern: Prototype — every element below the ones named is cloned with a new id; a reference
//! to a copied element is retargeted, a reference to anything else is left alone.

use std::collections::BTreeMap;

use fs_store::ObjectId;

use super::handle::Editor;
use super::selection::{unique, with_members};
use crate::error::{Error, Result};
use crate::model::{Anchor, Element, Kind};
use crate::order;
use crate::repo::tx::Tx;
use crate::validate;

fn retarget(a: &Anchor, map: &BTreeMap<ObjectId, ObjectId>, dx: f64, dy: f64) -> Anchor {
    match a.element {
        Some(old) => Anchor {
            element: Some(map.get(&old).copied().unwrap_or(old)),
            point: a.point,
        },
        None => Anchor {
            element: None,
            point: crate::model::Point {
                x: a.point.x + dx,
                y: a.point.y + dy,
            },
        },
    }
}

impl Editor {
    pub(super) async fn copy(&self, ids: &[ObjectId], dx: f64, dy: f64) -> Result<Vec<ObjectId>> {
        validate::finite("dx", dx)?;
        validate::finite("dy", dy)?;
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        let mut tx = Tx::new(&self.repo.actor);
        let named = unique(ids);
        for id in &named {
            if tx.el(st, *id)?.deleted {
                return Err(Error::NotFound {
                    what: "element",
                    id: *id,
                });
            }
        }
        let mut all: Vec<&Element> = with_members(&view.parents, &named)
            .into_iter()
            .map(|id| &st.elements[&id])
            .filter(|e| !e.deleted)
            .collect();
        all.sort_by(|a, b| (a.order.as_bytes(), a.id).cmp(&(b.order.as_bytes(), b.id)));
        let map: BTreeMap<ObjectId, ObjectId> =
            all.iter().map(|e| (e.id, ObjectId::new())).collect();
        let keys = order::spread(super::add::highest_key(st).as_deref(), None, all.len());
        for (e, key) in all.into_iter().zip(keys) {
            let mut c = e.clone();
            c.id = map[&e.id];
            c.parent = e.parent.map(|p| map.get(&p).copied().unwrap_or(p));
            c.order = key;
            c.locked = false;
            c.frame = e.frame.moved(dx, dy);
            if let Kind::Connector(k) = &mut c.kind {
                k.source = retarget(&k.source, &map, dx, dy);
                k.target = retarget(&k.target, &map, dx, dy);
            }
            tx.create(c);
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(named.iter().map(|id| map[id]).collect())
    }
}
