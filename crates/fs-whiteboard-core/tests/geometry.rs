//! Pure geometry and reading rules.

mod common;

use common::rect_element;
use fs_objects::ObjectId;
use fs_whiteboard_core::geometry::bounds;
use fs_whiteboard_core::reading::{View, depths, effective_pages, effective_parents};
use fs_whiteboard_core::{Element, Kind, Page, Rect};
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::str::FromStr;

fn id(n: u8) -> ObjectId {
    ObjectId::from_str(&format!("01900000-0000-7000-8000-0000000000{n:02x}")).unwrap()
}

proptest! {
    #![proptest_config(common::cfg(200))]

    #[test]
    fn rotated_bounds_hold_all_four_corners_and_touch_them(
        x in -500i32..500, y in -500i32..500, w in 0u32..300, h in 0u32..300, a in -400i32..400,
    ) {
        let (x, y, w, h, a) = (f64::from(x), f64::from(y), f64::from(w), f64::from(h), f64::from(a) / 100.0);
        let f = Rect::new(x, y, w, h);
        let b = bounds(&f, a);
        let (c, (sin, cos)) = ((x + w / 2.0, y + h / 2.0), a.sin_cos());
        let corners: Vec<(f64, f64)> = [(x, y), (x + w, y), (x, y + h), (x + w, y + h)]
            .iter()
            .map(|(px, py)| (c.0 + (px - c.0) * cos - (py - c.1) * sin, c.1 + (px - c.0) * sin + (py - c.1) * cos))
            .collect();
        let min = |f: fn(&(f64, f64)) -> f64| corners.iter().map(f).fold(f64::INFINITY, f64::min);
        let max = |f: fn(&(f64, f64)) -> f64| corners.iter().map(f).fold(f64::NEG_INFINITY, f64::max);
        for (got, want) in [(b.x, min(|p| p.0)), (b.y, min(|p| p.1)), (b.right(), max(|p| p.0)), (b.bottom(), max(|p| p.1))] {
            prop_assert!((got - want).abs() < 1e-6, "{got} vs {want}");
        }
    }

    #[test]
    fn broken_cycles_end_and_keep_the_lowest_id_free(parents in prop::collection::vec(prop::option::of(0u8..8), 8)) {
        let els: BTreeMap<ObjectId, Element> = parents.iter().enumerate().map(|(i, p)| {
            let mut e = rect_element(id(100), 0.0, 0.0);
            e.id = id(i as u8);
            e.parent = p.map(id);
            (e.id, e)
        }).collect();
        let eff = effective_parents(&els);
        prop_assert_eq!(depths(&eff).len(), 8, "every walk up ends");
        for (k, v) in &eff {
            if *v != els[k].parent {
                // cut: it lay on a cycle of the raw relation, and was its lowest id
                let mut cycle = vec![*k];
                let mut at = els[k].parent;
                while let Some(n) = at.filter(|n| n != k) {
                    cycle.push(n);
                    at = els[&n].parent;
                    prop_assert!(cycle.len() <= 9, "it was on a cycle");
                }
                prop_assert_eq!(*k, *cycle.iter().min().unwrap());
                prop_assert!(v.is_none());
            }
        }
        // the same relation built in another order gives the same answer
        let reversed: BTreeMap<_, _> = els.iter().rev().map(|(k, v)| (*k, v.clone())).collect();
        prop_assert_eq!(effective_parents(&reversed), eff);
    }
}

#[test]
fn the_view_hides_through_deleted_ancestors_and_pages() {
    let pages = [
        Page {
            id: id(200),
            name: "a".into(),
            order: "1".into(),
            deleted: false,
        },
        Page {
            id: id(201),
            name: "b".into(),
            order: "2".into(),
            deleted: true,
        },
    ];
    let mut els = BTreeMap::new();
    let mut put = |n: u8, page: u8, parent: Option<u8>, deleted: bool, kind: Kind| {
        let mut e = Element::new(id(page), kind, Rect::new(0.0, 0.0, 1.0, 1.0));
        (e.id, e.parent, e.deleted) = (id(n), parent.map(id), deleted);
        els.insert(e.id, e);
    };
    put(
        1,
        200,
        None,
        true,
        Kind::Area {
            text: String::new(),
        },
    );
    put(
        2,
        200,
        Some(1),
        false,
        Kind::Area {
            text: String::new(),
        },
    );
    put(
        3,
        200,
        Some(2),
        false,
        Kind::Area {
            text: String::new(),
        },
    );
    put(
        4,
        200,
        None,
        false,
        Kind::Area {
            text: String::new(),
        },
    );
    put(
        5,
        201,
        None,
        false,
        Kind::Area {
            text: String::new(),
        },
    );
    let v = View::build(&els, &pages);
    let hidden: Vec<_> = v.hidden.iter().copied().collect();
    assert_eq!(hidden, vec![id(1), id(2), id(3), id(5)]);
    assert_eq!(
        effective_pages(&pages)
            .iter()
            .filter(|p| !p.deleted)
            .count(),
        1
    );
    let both = [
        pages[0].clone(),
        Page {
            deleted: true,
            ..pages[0].clone()
        },
        Page {
            id: id(202),
            deleted: true,
            ..pages[1].clone()
        },
    ];
    let live: Vec<_> = effective_pages(&[
        Page {
            deleted: true,
            ..both[0].clone()
        },
        both[2].clone(),
    ])
    .into_iter()
    .filter(|p| !p.deleted)
    .map(|p| p.id)
    .collect();
    assert_eq!(live, vec![id(202)], "all removed: the highest id stays");
}
