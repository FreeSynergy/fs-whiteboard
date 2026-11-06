//! [`BoardState`]: one board held in memory.
//!
//! Pattern: Identity map with derived caches — the loaded elements with their clocks, the spatial
//! index, and the reading rules' `View`, rebuilt lazily after a change. The store is written first;
//! the state follows, so it never holds what the store does not.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use fs_store::ObjectId;
use serde_json::json;

use crate::clock::Clock;
use crate::doc::{BoardDoc, ElementDoc};
use crate::geometry::bounds;
use crate::model::{Board, Element, Kind, Page};
use crate::reading::{View, effective_pages};
use crate::spatial::SpatialIndex;
use crate::unit::Unit;

type Clocks = BTreeMap<Unit, Clock>;

pub(super) struct BoardState {
    pub(super) id: ObjectId,
    pub(super) doc: BoardDoc,
    pub(super) elements: BTreeMap<ObjectId, Element>,
    pub(super) clocks: BTreeMap<ObjectId, Clocks>,
    pub(super) groups: BTreeSet<ObjectId>,
    pub(super) index: SpatialIndex,
    view: Option<Arc<View>>,
}

impl BoardState {
    pub(super) fn new(id: ObjectId, doc: BoardDoc) -> Self {
        Self {
            id,
            doc,
            elements: BTreeMap::new(),
            clocks: BTreeMap::new(),
            groups: BTreeSet::new(),
            index: SpatialIndex::new(),
            view: None,
        }
    }

    /// Takes an element document in (new or changed).
    pub(super) fn put(&mut self, d: ElementDoc) {
        let e = d.element;
        if matches!(e.kind, Kind::Group) {
            self.groups.insert(e.id);
        } else {
            self.index.upsert(e.id, e.page, bounds(&e.frame, e.angle));
        }
        self.clocks.insert(e.id, d.clocks);
        self.elements.insert(e.id, e);
        self.view = None;
    }

    pub(super) fn set_doc(&mut self, doc: BoardDoc) {
        self.doc = doc;
        self.view = None;
    }

    pub(super) fn element_doc(&self, id: ObjectId) -> Option<ElementDoc> {
        Some(ElementDoc {
            element: self.elements.get(&id)?.clone(),
            clocks: self.clocks.get(&id)?.clone(),
        })
    }

    pub(super) fn clock(&self, id: ObjectId, unit: Unit) -> Clock {
        self.clocks
            .get(&id)
            .and_then(|c| c.get(&unit))
            .cloned()
            .unwrap_or_default()
    }

    pub(super) fn pages(&self) -> Vec<Page> {
        self.doc.pages.values().cloned().collect()
    }

    /// The reading rules applied to the current state (computed once per change).
    pub(super) fn view(&mut self) -> Arc<View> {
        if self.view.is_none() {
            self.view = Some(Arc::new(View::build(&self.elements, &self.pages())));
        }
        self.view.clone().unwrap_or_default()
    }

    /// The board as the reader sees it.
    pub(super) fn board(&self, iri: &str) -> Board {
        let as2 = json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Collection", "id": iri, "name": self.doc.name,
        });
        Board {
            id: self.id,
            iri: iri.to_string(),
            name: self.doc.name.clone(),
            as2: as2.to_string(),
            pages: effective_pages(&self.pages()),
        }
    }
}
