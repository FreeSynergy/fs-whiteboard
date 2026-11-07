//! Merge properties over arbitrary sets of concurrent changes to every unit.

mod common;

use std::str::FromStr;

use fs_objects::ObjectId;
use fs_whiteboard_core::merge::{merge_board, merge_element as try_merge};
use fs_whiteboard_core::model::{Anchor, Connector};
use fs_whiteboard_core::{BoardDoc, Element, ElementDoc, Kind, Page, Rect, Unit};
use proptest::prelude::*;
use serde_json::{Value, json};

fn pool(n: u8) -> ObjectId {
    ObjectId::from_str(&format!("01900000-0000-7000-8000-00000000000{}", n % 4)).unwrap()
}

/// A value for a unit, from a small number so that equal values and equal clocks do collide.
fn value(unit: Unit, v: u8) -> Value {
    let n = f64::from(v);
    match unit {
        Unit::Page => json!(pool(v)),
        Unit::Parent => json!((v % 2 == 0).then(|| pool(v))),
        Unit::Order
        | Unit::Text
        | Unit::Label
        | Unit::StyleStroke
        | Unit::StyleFill
        | Unit::StyleFontFamily => json!(format!("v{}", v % 5)),
        Unit::Frame => json!({"x": n, "y": n / 2.0, "width": n, "height": 1.0}),
        Unit::Angle | Unit::StyleLineWidth => json!(n / 4.0),
        Unit::Locked | Unit::Deleted | Unit::StyleDashed => json!(v % 2 == 0),
        Unit::Source | Unit::Target => json!(Anchor {
            element: (v % 2 == 0).then(|| pool(v)),
            point: fs_whiteboard_core::Point { x: n, y: 0.0 }
        }),
        Unit::StyleOpacity | Unit::StyleFontSize => json!(n / 255.0),
    }
}

fn base() -> ElementDoc {
    let kind = Kind::Connector(Connector::default());
    let mut e = Element::new(pool(0), kind, Rect::new(0.0, 0.0, 5.0, 5.0));
    e.id = pool(1);
    e.board = pool(2);
    e.order = "V".into();
    ElementDoc::fresh(e, "base")
}

/// `(unit, value, actor, replica)`: one change by one actor on one replica.
fn changes() -> impl Strategy<Value = Vec<(usize, u8, u8, usize)>> {
    prop::collection::vec((0..Unit::ALL.len(), any::<u8>(), 0u8..3, 0usize..3), 0..24)
}

fn replicas(ch: &[(usize, u8, u8, usize)]) -> [ElementDoc; 3] {
    let mut docs = [base(), base(), base()];
    for &(u, v, a, r) in ch {
        let unit = Unit::ALL[u];
        docs[r]
            .set(vec![(unit, value(unit, v))], &format!("actor-{a}"))
            .unwrap();
    }
    docs
}

fn merge_element(a: &ElementDoc, b: &ElementDoc) -> ElementDoc {
    try_merge(a, b).unwrap()
}

fn bytes(d: &ElementDoc) -> String {
    serde_json::to_string(d).unwrap()
}

proptest! {
    #![proptest_config(common::cfg(200))]

    #[test]
    fn elements_commute_associate_and_are_idempotent(ch in changes()) {
        let [a, b, c] = replicas(&ch);
        prop_assert_eq!(merge_element(&a, &b), merge_element(&b, &a));
        prop_assert_eq!(
            merge_element(&merge_element(&a, &b), &c),
            merge_element(&a, &merge_element(&b, &c))
        );
        prop_assert_eq!(merge_element(&a, &a), a.clone());
        let ab = merge_element(&a, &b);
        prop_assert_eq!(merge_element(&ab, &b), ab.clone());
        prop_assert_eq!(merge_element(&ab, &a), ab);
    }

    #[test]
    fn replicas_end_byte_identical_in_any_order(ch in changes()) {
        let docs = replicas(&ch);
        let orders: [[usize; 3]; 6] = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
        let results: Vec<String> = orders
            .iter()
            .map(|o| bytes(&o.iter().skip(1).fold(docs[o[0]].clone(), |acc, &i| merge_element(&acc, &docs[i]))))
            .collect();
        prop_assert!(results.windows(2).all(|w| w[0] == w[1]));
    }

    #[test]
    fn no_change_is_lost_unless_something_larger_replaced_it(ch in changes()) {
        let docs = replicas(&ch);
        let merged = merge_element(&merge_element(&docs[0], &docs[1]), &docs[2]);
        for unit in Unit::ALL {
            let best = docs.iter().map(|d| d.clock(unit)).max().unwrap();
            prop_assert_eq!(merged.clock(unit), best);
        }
    }
}

fn board_base() -> BoardDoc {
    let page = |n: u8| Page {
        id: pool(n),
        name: format!("p{n}"),
        order: format!("{n}"),
        deleted: false,
    };
    let mut d = BoardDoc::fresh("board", page(0), "base");
    d.add_page(page(1), "base");
    d
}

/// `(what, page, value, actor, replica)`
fn board_changes() -> impl Strategy<Value = Vec<(u8, u8, u8, u8, usize)>> {
    prop::collection::vec((0u8..5, 0u8..4, any::<u8>(), 0u8..3, 0usize..3), 0..20)
}

proptest! {
    #![proptest_config(common::cfg(200))]

    #[test]
    fn boards_commute_associate_and_are_idempotent(ch in board_changes()) {
        let mut docs = [board_base(), board_base(), board_base()];
        for (what, page, v, a, r) in ch {
            let actor = format!("actor-{a}");
            let d = &mut docs[r];
            match what {
                0 => d.set_name(&format!("n{}", v % 4), &actor),
                1 => d.add_page(Page { id: pool(page), name: format!("x{v}"), order: format!("{}", v % 7), deleted: false }, &actor),
                2 => d.edit_page(pool(page), "name", &actor, |p| p.name = format!("r{}", v % 3)),
                3 => d.edit_page(pool(page), "order", &actor, |p| p.order = format!("{}", v % 7)),
                _ => d.edit_page(pool(page), "deleted", &actor, |p| p.deleted = v % 2 == 0),
            }
        }
        let [a, b, c] = docs;
        prop_assert_eq!(merge_board(&a, &b), merge_board(&b, &a));
        prop_assert_eq!(merge_board(&merge_board(&a, &b), &c), merge_board(&a, &merge_board(&b, &c)));
        prop_assert_eq!(merge_board(&a, &a), a.clone());
        let ab = merge_board(&a, &b);
        prop_assert_eq!(serde_json::to_string(&merge_board(&ab, &b)).unwrap(), serde_json::to_string(&ab).unwrap());
    }
}

#[test]
fn every_unit_is_covered_by_the_generator() {
    let d = replicas(
        &Unit::ALL
            .iter()
            .enumerate()
            .map(|(i, _)| (i, 1, 0, 0))
            .collect::<Vec<_>>(),
    );
    for unit in Unit::ALL {
        assert!(d[0].clock(unit).counter() >= 2, "{unit:?} was changed");
    }
}
