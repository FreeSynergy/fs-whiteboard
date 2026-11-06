//! Order keys: strictly between, bounded growth, ties by id.

mod common;

use common::{all, pair, rect_element};
use fs_whiteboard_core::order::{between, is_valid, spread};
use proptest::prelude::*;

#[test]
fn edges_are_strictly_inside() {
    let first = between(None, None);
    assert!(is_valid(&first));
    for b in ["1", "01", "V", "z", "zz", "0001", "10001"] {
        let k = between(None, Some(b));
        assert!(is_valid(&k) && k.as_str() < b, "{k} < {b}");
        let a = between(Some(b), None);
        assert!(is_valid(&a) && a.as_str() > b, "{a} > {b}");
    }
    // no room is ever missing: right next to each other still has a key between
    let k = between(Some("V"), Some("W"));
    assert!("V" < k.as_str() && k.as_str() < "W");
    let k = between(Some("z"), Some("z1"));
    assert!("z" < k.as_str() && k.as_str() < "z1");
}

/// Inserts `n` keys, each at the place `pick` names, and returns the longest key.
fn run(n: usize, mut pick: impl FnMut(&[String]) -> (Option<usize>, Option<usize>)) -> usize {
    let mut keys: Vec<String> = Vec::new();
    for _ in 0..n {
        let (lo, hi) = pick(&keys);
        let k = between(lo.map(|i| keys[i].as_str()), hi.map(|i| keys[i].as_str()));
        assert!(is_valid(&k));
        let at = lo.map_or(0, |i| i + 1);
        if let Some(l) = lo {
            assert!(keys[l] < k);
        }
        if let Some(h) = hi {
            assert!(k < keys[h]);
        }
        keys.insert(at, k);
    }
    assert!(keys.windows(2).all(|w| w[0] < w[1]));
    keys.iter().map(String::len).max().unwrap_or(0)
}

#[test]
fn ten_thousand_inserts_at_one_place() {
    // Longest key measured: appends 165, prepends 323, always into the same shrinking gap 2001
    // (5 inserts a character, 6 bits each at best, which no scheme with keys that never change beats).
    let end = run(10_000, |k| (k.len().checked_sub(1), None));
    let front = run(10_000, |k| (None, (!k.is_empty()).then_some(0)));
    let nested_after = run(10_000, |k| {
        if k.is_empty() {
            (None, None)
        } else {
            (Some(0), if k.len() > 1 { Some(1) } else { None })
        }
    });
    eprintln!("longest key: append {end}, prepend {front}, nested {nested_after}");
    assert!(end <= 200 && nested_after <= 2100);
}

proptest! {
    #![proptest_config(common::cfg(48))]

    #[test]
    fn random_places_stay_short_and_ordered(places in prop::collection::vec(0usize..1000, 1..400)) {
        let mut keys: Vec<String> = Vec::new();
        for p in places {
            let at = p % (keys.len() + 1);
            let lo = at.checked_sub(1).map(|i| keys[i].as_str());
            let k = between(lo, keys.get(at).map(String::as_str));
            keys.insert(at, k);
        }
        prop_assert!(keys.windows(2).all(|w| w[0] < w[1]));
        prop_assert!(keys.iter().all(|k| is_valid(k) && k.len() <= 24));
    }

    #[test]
    fn between_any_two_valid_keys(a in "[0-9A-Za-z]{0,6}[1-9A-Za-z]", b in "[0-9A-Za-z]{0,6}[1-9A-Za-z]") {
        prop_assume!(a < b);
        let k = between(Some(&a), Some(&b));
        prop_assert!(is_valid(&k) && a < k && k < b, "{a} < {k} < {b}");
    }

    #[test]
    fn spread_is_increasing_and_inside(n in 0usize..200, a in "[1-9A-Za-z]{1,3}") {
        let b = between(Some(&a), None);
        let keys = spread(Some(&a), Some(&b), n);
        prop_assert_eq!(keys.len(), n);
        let all: Vec<&String> = std::iter::once(&a).chain(&keys).chain(std::iter::once(&b)).collect();
        prop_assert!(all.windows(2).all(|w| w[0] < w[1]));
    }
}

#[tokio::test]
async fn equal_keys_sort_by_id_on_every_replica() {
    let p = pair().await;
    // both add at the front while apart: both compute the same key
    let ea = p.a.ed.add(rect_element(p.a.page, 0.0, 0.0)).await.unwrap();
    let eb = p.b.ed.add(rect_element(p.b.page, 0.0, 0.0)).await.unwrap();
    assert_eq!(ea.order, eb.order);
    p.sync().await;
    let (la, lb) = (all(&*p.a.ed, p.a.page).await, all(&*p.b.ed, p.b.page).await);
    let (ia, ib): (Vec<_>, Vec<_>) = (
        la.iter().map(|e| e.id).collect(),
        lb.iter().map(|e| e.id).collect(),
    );
    assert_eq!(ia, ib);
    assert_eq!(ia.len(), 2);
    assert!(ia[0] < ia[1], "equal keys: lower id behind");
}
