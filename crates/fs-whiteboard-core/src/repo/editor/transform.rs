//! Moving, resizing, rotating and aligning.
//!
//! Pattern: Composite — a group has no position of its own: moving it moves every member below it,
//! each once, however many times it or they are named. A locked element anywhere in that set
//! refuses the whole edit.

use fs_objects::ObjectId;
use serde_json::{Value, to_value};

use super::handle::Editor;
use super::selection::{outermost, unique, with_members};
use crate::error::{Error, Result};
use crate::geometry::align;
use crate::model::{Align, Anchor, Element, Kind, Rect};
use crate::repo::state::BoardState;
use crate::repo::tx::Tx;
use crate::unit::Unit;
use crate::validate;

fn shifted(a: &Anchor, dx: f64, dy: f64) -> Anchor {
    let mut a = *a;
    a.point.x += dx;
    a.point.y += dy;
    a
}

/// The unit changes that move an element by `(dx, dy)`.
pub(super) fn moved_units(e: &Element, dx: f64, dy: f64) -> Vec<(Unit, Value)> {
    let mut out = vec![(
        Unit::Frame,
        to_value(e.frame.moved(dx, dy)).unwrap_or_default(),
    )];
    if let Kind::Connector(c) = &e.kind {
        out.push((
            Unit::Source,
            to_value(shifted(&c.source, dx, dy)).unwrap_or_default(),
        ));
        out.push((
            Unit::Target,
            to_value(shifted(&c.target, dx, dy)).unwrap_or_default(),
        ));
    }
    out
}

/// Moves `ids` and everything below them; refuses a locked or unknown element.
pub(super) fn move_all(
    tx: &mut Tx,
    st: &BoardState,
    parents: &super::selection::Parents,
    ids: &[ObjectId],
    dx: f64,
    dy: f64,
) -> Result<()> {
    for id in with_members(parents, ids) {
        let named = ids.contains(&id);
        let e = if named {
            tx.editable(st, id)?
        } else {
            tx.el(st, id)?
        };
        if e.deleted {
            continue;
        }
        if e.locked {
            return Err(Error::Locked(id));
        }
        if !matches!(e.kind, Kind::Group) {
            validate::rect(&e.frame.moved(dx, dy))?; // the sum may leave Float64
            if let Kind::Connector(c) = &e.kind {
                for a in [&c.source, &c.target] {
                    validate::point(&shifted(a, dx, dy).point)?;
                }
            }
        }
        let units = (!matches!(e.kind, Kind::Group)).then(|| moved_units(e, dx, dy));
        if let Some(units) = units {
            tx.set(st, id, units)?;
        }
    }
    Ok(())
}

impl Editor {
    pub(super) async fn move_by(&self, ids: &[ObjectId], dx: f64, dy: f64) -> Result<()> {
        validate::finite("dx", dx)?;
        validate::finite("dy", dy)?;
        let ids = unique(ids);
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        let mut tx = Tx::new(&self.repo.actor);
        move_all(&mut tx, st, &view.parents, &ids, dx, dy)?;
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }

    pub(super) async fn set_frame(&self, id: ObjectId, frame: Rect) -> Result<()> {
        validate::rect(&frame)?;
        self.single(id, Unit::Frame, to_value(frame)?).await
    }

    pub(super) async fn rotate(&self, id: ObjectId, angle: f64) -> Result<()> {
        validate::finite("angle", angle)?;
        self.single(id, Unit::Angle, to_value(angle)?).await
    }

    async fn single(&self, id: ObjectId, unit: Unit, value: Value) -> Result<()> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        if matches!(tx.editable(st, id)?.kind, Kind::Group) {
            return Err(Error::invalid(
                "a group has no position of its own; move its members",
            ));
        }
        tx.set(st, id, vec![(unit, value)])?;
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }

    pub(super) async fn align(&self, ids: &[ObjectId], how: Align) -> Result<()> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let view = st.view();
        let mut tx = Tx::new(&self.repo.actor);
        let ids = outermost(&view.parents, &unique(ids));
        let mut items = Vec::new();
        for id in &ids {
            let e = tx.editable(st, *id)?;
            items.push((*id, view.bounds_of(e)));
        }
        for (id, dx, dy) in align(&items, how) {
            move_all(&mut tx, st, &view.parents, &[id], dx, dy)?;
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }
}
