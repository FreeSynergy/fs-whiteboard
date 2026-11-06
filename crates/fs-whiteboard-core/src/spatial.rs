//! The spatial index: which elements of a page lie in an area. In memory, no I/O.
//!
//! Pattern: Facade over `rstar` — one R-tree per page over the bounding boxes after rotation, plus
//! a map id -> entry so an update removes the old box exactly. Hiding is not known here: the caller
//! filters the candidates with the reading rules.

use std::collections::HashMap;

use fs_store::ObjectId;
use rstar::{AABB, RTree, RTreeObject};

use crate::model::Rect;

#[derive(Clone, Debug, PartialEq)]
struct Entry {
    id: ObjectId,
    page: ObjectId,
    rect: Rect,
}

impl RTreeObject for Entry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        aabb(&self.rect)
    }
}

fn aabb(r: &Rect) -> AABB<[f64; 2]> {
    AABB::from_corners([r.x, r.y], [r.right(), r.bottom()])
}

/// R-trees per page.
#[derive(Default)]
pub struct SpatialIndex {
    trees: HashMap<ObjectId, RTree<Entry>>,
    entries: HashMap<ObjectId, Entry>,
}

impl SpatialIndex {
    /// An empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Puts an element with its bounds after rotation on `page`, replacing an earlier entry.
    pub fn upsert(&mut self, id: ObjectId, page: ObjectId, rect: Rect) {
        self.remove(id);
        let entry = Entry { id, page, rect };
        self.trees.entry(page).or_default().insert(entry.clone());
        self.entries.insert(id, entry);
    }

    /// Takes an element out.
    pub fn remove(&mut self, id: ObjectId) {
        if let Some(old) = self.entries.remove(&id) {
            if let Some(tree) = self.trees.get_mut(&old.page) {
                tree.remove(&old);
            }
        }
    }

    /// Ids on `page` whose bounds touch `area`; all of the page if `area` is `None`.
    pub fn query(&self, page: ObjectId, area: Option<&Rect>) -> Vec<ObjectId> {
        let Some(tree) = self.trees.get(&page) else {
            return Vec::new();
        };
        match area {
            Some(a) => tree
                .locate_in_envelope_intersecting(aabb(a))
                .map(|e| e.id)
                .collect(),
            None => tree.iter().map(|e| e.id).collect(),
        }
    }

    /// The bounds stored for an element.
    pub fn rect_of(&self, id: ObjectId) -> Option<Rect> {
        self.entries.get(&id).map(|e| e.rect)
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if there are none.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
