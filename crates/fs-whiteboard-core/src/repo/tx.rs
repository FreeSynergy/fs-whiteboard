//! [`Tx`]: the working copy of one edit.
//!
//! Pattern: Unit of Work — an edit changes copies of the documents and notes each unit change; only
//! when every check has passed are the copies written to the store and taken into the state. A
//! refused edit therefore changes nothing.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use fs_store::ObjectId;
use serde_json::Value;

use super::handle::WhiteboardRepo;
use super::history::{Edit, UnitChange};
use super::state::BoardState;
use crate::clock::Clock;
use crate::doc::ElementDoc;
use crate::error::{Error, Result};
use crate::model::Element;
use crate::unit::Unit;

pub(super) struct Tx {
    actor: String,
    docs: BTreeMap<ObjectId, ElementDoc>,
    written: Vec<ObjectId>,
    pub(super) changes: Edit,
}

impl Tx {
    pub(super) fn new(actor: &str) -> Self {
        Self {
            actor: actor.to_string(),
            docs: BTreeMap::new(),
            written: vec![],
            changes: vec![],
        }
    }

    /// The element as this edit sees it; `Foreign` if it is not on the board.
    pub(super) fn el(&mut self, st: &BoardState, id: ObjectId) -> Result<&Element> {
        if let Entry::Vacant(v) = self.docs.entry(id) {
            v.insert(st.element_doc(id).ok_or(Error::Foreign(id))?);
        }
        Ok(&self.docs[&id].element)
    }

    /// Like [`Tx::el`], but refuses a deleted or locked element.
    pub(super) fn editable(&mut self, st: &BoardState, id: ObjectId) -> Result<&Element> {
        let e = self.el(st, id)?;
        if e.deleted {
            return Err(Error::NotFound {
                what: "element",
                id,
            });
        }
        if e.locked {
            return Err(Error::Locked(id));
        }
        Ok(e)
    }

    /// Changes some units of one element under one new clock; units that already hold the value
    /// are left alone.
    pub(super) fn set(
        &mut self,
        st: &BoardState,
        id: ObjectId,
        changes: Vec<(Unit, Value)>,
    ) -> Result<()> {
        self.el(st, id)?;
        let doc = self.docs.get_mut(&id).ok_or(Error::Foreign(id))?;
        let todo: Vec<(Unit, Value, Value)> = changes
            .into_iter()
            .filter_map(|(u, v)| {
                let old = u.read(&doc.element)?;
                (old != v).then_some((u, old, v))
            })
            .collect();
        if todo.is_empty() {
            return Ok(());
        }
        let before: Vec<Clock> = todo.iter().map(|(u, _, _)| doc.clock(*u)).collect();
        let clock = doc.set(
            todo.iter().map(|(u, _, v)| (*u, v.clone())).collect(),
            &self.actor,
        )?;
        for ((unit, old, new), before) in todo.into_iter().zip(before) {
            let clock = clock.clone();
            self.changes.push(UnitChange {
                element: id,
                unit,
                old,
                new,
                clock,
                before,
            });
        }
        if !self.written.contains(&id) {
            self.written.push(id);
        }
        Ok(())
    }

    /// A new element: it exists from the first clock on; undo of the add deletes it.
    pub(super) fn create(&mut self, element: Element) {
        let id = element.id;
        let doc = ElementDoc::fresh(element, &self.actor);
        let clock = doc.clock(Unit::Deleted);
        self.changes.push(UnitChange {
            element: id,
            unit: Unit::Deleted,
            old: Value::Bool(true),
            new: Value::Bool(false),
            clock,
            before: Clock::default(),
        });
        self.docs.insert(id, doc);
        self.written.push(id);
    }
}

impl WhiteboardRepo {
    /// Writes the working copies to the store, then takes them into the state.
    pub(super) async fn commit(&self, st: &mut BoardState, tx: Tx) -> Result<Edit> {
        for id in &tx.written {
            let doc = &tx.docs[id];
            self.save_element(doc).await?;
            st.put(doc.clone());
        }
        Ok(tx.changes)
    }
}
