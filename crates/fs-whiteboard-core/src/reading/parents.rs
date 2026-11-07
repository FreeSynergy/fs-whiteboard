//! The parent relation after a cycle is broken.
//!
//! Pattern: Strategy (conflict rule) — two concurrent changes of `parent` can form a cycle. At reading,
//! the element with the lowest id on each cycle counts as having no parent, on every replica.

use std::collections::{BTreeMap, BTreeSet};

use fs_objects::ObjectId;

use crate::model::Element;

/// Parent of every element, with a missing parent (not on the board) read as none and each cycle cut
/// at its lowest id. The result has no cycle, so walks up from any element end.
pub fn effective_parents(
    elements: &BTreeMap<ObjectId, Element>,
) -> BTreeMap<ObjectId, Option<ObjectId>> {
    let mut parents: BTreeMap<ObjectId, Option<ObjectId>> = elements
        .iter()
        .map(|(id, e)| (*id, e.parent.filter(|p| elements.contains_key(p))))
        .collect();
    let mut done: BTreeSet<ObjectId> = BTreeSet::new();
    for &start in elements.keys() {
        let mut path: Vec<ObjectId> = Vec::new();
        let mut on_path: BTreeMap<ObjectId, usize> = BTreeMap::new();
        let mut at = Some(start);
        while let Some(id) = at.filter(|id| !done.contains(id)) {
            if let Some(&pos) = on_path.get(&id) {
                if let Some(lowest) = path[pos..].iter().min() {
                    parents.insert(*lowest, None);
                }
                break;
            }
            on_path.insert(id, path.len());
            path.push(id);
            at = parents[&id];
        }
        done.extend(path);
    }
    parents
}

/// Distance to the root of every element (0 = no parent). `parents` must have no cycle.
pub fn depths(parents: &BTreeMap<ObjectId, Option<ObjectId>>) -> BTreeMap<ObjectId, usize> {
    let mut depth: BTreeMap<ObjectId, usize> = BTreeMap::new();
    for &start in parents.keys() {
        let mut path = Vec::new();
        let mut at = Some(start);
        while let Some(id) = at.filter(|id| !depth.contains_key(id)) {
            path.push(id);
            at = parents.get(&id).copied().flatten();
        }
        let base = at.map_or(0, |id| depth[&id] + 1);
        for (d, id) in (base..).zip(path.into_iter().rev()) {
            depth.insert(id, d);
        }
    }
    depth
}
