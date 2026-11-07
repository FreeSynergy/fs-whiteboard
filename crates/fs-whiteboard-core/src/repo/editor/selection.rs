//! Helpers on a selection of ids: no duplicates, with members, without nested ones.
//!
//! Pattern: Iterator helpers over the parent relation after cycles are broken — a group acts on its
//! members, each once, however many times it or they are named.

use std::collections::{BTreeMap, BTreeSet};

use fs_objects::ObjectId;

pub(super) type Parents = BTreeMap<ObjectId, Option<ObjectId>>;

/// `ids` without repeats, in the order first named.
pub(super) fn unique(ids: &[ObjectId]) -> Vec<ObjectId> {
    let mut seen = BTreeSet::new();
    ids.iter().copied().filter(|id| seen.insert(*id)).collect()
}

/// `ids` and everything below them.
pub(super) fn with_members(parents: &Parents, ids: &[ObjectId]) -> BTreeSet<ObjectId> {
    let mut children: BTreeMap<ObjectId, Vec<ObjectId>> = BTreeMap::new();
    for (id, p) in parents {
        if let Some(p) = p {
            children.entry(*p).or_default().push(*id);
        }
    }
    let mut out: BTreeSet<ObjectId> = BTreeSet::new();
    let mut todo: Vec<ObjectId> = ids.to_vec();
    while let Some(id) = todo.pop() {
        if out.insert(id) {
            todo.extend(children.get(&id).into_iter().flatten());
        }
    }
    out
}

/// True if `id` is `of` or lies below it.
pub(super) fn is_within(parents: &Parents, id: ObjectId, of: ObjectId) -> bool {
    let mut at = Some(id);
    while let Some(a) = at {
        if a == of {
            return true;
        }
        at = parents.get(&a).copied().flatten();
    }
    false
}

/// `ids` without those that lie below another one of them.
pub(super) fn outermost(parents: &Parents, ids: &[ObjectId]) -> Vec<ObjectId> {
    ids.iter()
        .copied()
        .filter(|id| {
            let up = parents.get(id).copied().flatten();
            !ids.iter()
                .any(|o| o != id && up.is_some_and(|u| is_within(parents, u, *o)))
        })
        .collect()
}
