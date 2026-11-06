//! [`View`]: everything the reading rules decide about one board, computed once per state.
//!
//! Pattern: Memento-like snapshot of derived data — rebuilt after a change, never edited. Hidden:
//! deleted itself, in a deleted group or frame (also one that got its parent concurrently), or on
//! a removed page. A group's frame is the union of its members, at reading.

use std::collections::{BTreeMap, BTreeSet};

use fs_store::ObjectId;

use super::connector::End;
use super::pages::effective_pages;
use super::parents::{depths, effective_parents};
use crate::geometry::bounds;
use crate::model::{Anchor, Element, Kind, Page, Point, Rect};

/// The derived data of one board.
#[derive(Clone, Debug, Default)]
pub struct View {
    /// Parent of each element, cycles broken.
    pub parents: BTreeMap<ObjectId, Option<ObjectId>>,
    /// Elements that are not shown.
    pub hidden: BTreeSet<ObjectId>,
    /// Frame of each group: the union of its members.
    pub group_frames: BTreeMap<ObjectId, Rect>,
    /// Pages that count as present.
    pub live_pages: BTreeSet<ObjectId>,
    /// Where the two ends of each connector are.
    pub connector_ends: BTreeMap<ObjectId, (Point, Point)>,
}

impl View {
    /// Applies the reading rules to the loaded elements and pages.
    pub fn build(elements: &BTreeMap<ObjectId, Element>, pages: &[Page]) -> View {
        let parents = effective_parents(elements);
        let live_pages: BTreeSet<ObjectId> = effective_pages(pages)
            .into_iter()
            .filter(|p| !p.deleted)
            .map(|p| p.id)
            .collect();
        let depth = depths(&parents);
        let mut order: Vec<ObjectId> = elements.keys().copied().collect();
        order.sort_by_key(|id| (depth[id], *id));
        let mut hidden = BTreeSet::new();
        for id in &order {
            let e = &elements[id];
            let up = parents[id].is_some_and(|p| hidden.contains(&p));
            if e.deleted || up || !live_pages.contains(&e.page) {
                hidden.insert(*id);
            }
        }
        let group_frames = group_frames(elements, &parents, &order);
        // a group without a live member has no frame, and is not shown
        hidden.extend(
            elements
                .values()
                .filter(|e| matches!(e.kind, Kind::Group) && !group_frames.contains_key(&e.id))
                .map(|e| e.id),
        );
        let mut view = View {
            parents,
            hidden,
            group_frames,
            live_pages,
            connector_ends: BTreeMap::new(),
        };
        for e in elements.values() {
            if let Kind::Connector(c) = &e.kind {
                let ends = (
                    view.end(elements, &c.source).point(),
                    view.end(elements, &c.target).point(),
                );
                view.connector_ends.insert(e.id, ends);
            }
        }
        view
    }

    /// The element as the reader sees it: a group has the union of its members as frame.
    pub fn shown(&self, e: &Element) -> Element {
        let mut e = e.clone();
        if let Some(f) = self.group_frames.get(&e.id) {
            e.frame = *f;
        } else if let Some((a, b)) = self.connector_ends.get(&e.id) {
            e.frame = Rect::new(
                a.x.min(b.x),
                a.y.min(b.y),
                (a.x - b.x).abs(),
                (a.y - b.y).abs(),
            );
            e.angle = 0.0;
        }
        e
    }

    /// Bounds after rotation, as the index and `bounds` use them.
    pub fn bounds_of(&self, e: &Element) -> Rect {
        let shown = self.shown(e);
        bounds(&shown.frame, shown.angle)
    }

    /// Where one end of a connector is: at its element's frame (the last one, if the element is
    /// deleted), or free.
    pub fn end(&self, elements: &BTreeMap<ObjectId, Element>, a: &Anchor) -> End {
        match a.element.and_then(|id| elements.get(&id)) {
            Some(e) => End::Attached {
                element: e.id,
                frame: self.shown(e).frame,
                deleted: e.deleted,
            },
            None => End::Free(a.point),
        }
    }
}

fn group_frames(
    elements: &BTreeMap<ObjectId, Element>,
    parents: &BTreeMap<ObjectId, Option<ObjectId>>,
    by_depth: &[ObjectId],
) -> BTreeMap<ObjectId, Rect> {
    let mut acc: BTreeMap<ObjectId, Rect> = BTreeMap::new();
    for id in by_depth.iter().rev() {
        let e = &elements[id];
        let own = match &e.kind {
            Kind::Group => acc.get(id).copied(),
            // a connector lies between its ends, not where its stored frame says
            Kind::Connector(c) => {
                let at = |a: &Anchor| match a.element.and_then(|x| elements.get(&x)) {
                    Some(t) if matches!(t.kind, Kind::Group) => {
                        acc.get(&t.id).map_or(a.point, Rect::center)
                    }
                    Some(t) => t.frame.center(),
                    None => a.point,
                };
                let (p, q) = (at(&c.source), at(&c.target));
                Some(Rect::new(
                    p.x.min(q.x),
                    p.y.min(q.y),
                    (p.x - q.x).abs(),
                    (p.y - q.y).abs(),
                ))
            }
            _ => Some(bounds(&e.frame, e.angle)),
        };
        let parent = parents[id].filter(|p| matches!(elements[p].kind, Kind::Group));
        if let (Some(p), Some(r), false) = (parent, own, e.deleted) {
            let u = acc.get(&p).map_or(r, |a| a.union(&r));
            acc.insert(p, u);
        }
    }
    acc
}
