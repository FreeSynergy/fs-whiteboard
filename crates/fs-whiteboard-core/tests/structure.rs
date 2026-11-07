//! Regression: empty groups, long parent chains, page moves, order key limits.

mod common;

use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::Instant;

use common::{all, fixture, pair, rect_element};
use fs_objects::ObjectId;
use fs_whiteboard_core::order::{between, is_valid};
use fs_whiteboard_core::reading::effective_parents;
use fs_whiteboard_core::{Boards, Element, Error};
use serde_json::json;

#[tokio::test]
async fn a_group_without_a_live_member_is_not_shown() {
    let f = fixture().await;
    let (a, b) = (f.rect(0.0, 0.0).await, f.rect(20.0, 0.0).await);
    let g = f.ed.group(&[a, b]).await.unwrap();
    assert_eq!(all(&*f.ed, f.page).await.len(), 3);
    f.ed.delete(&[a]).await.unwrap();
    assert_eq!(all(&*f.ed, f.page).await.len(), 2, "one member is left");
    f.ed.delete(&[b]).await.unwrap();
    assert!(all(&*f.ed, f.page).await.is_empty(), "no member, no group");
    assert!(f.ed.bounds(f.page).await.unwrap().is_none());
    assert!(
        f.ed.get_element(g).await.unwrap().is_some(),
        "still there as a marker"
    );
    f.ed.undo().await.unwrap();
    assert_eq!(all(&*f.ed, f.page).await.len(), 2, "back with its member");
}

#[test]
fn a_chain_of_twenty_thousand_parents_is_read_in_linear_time() {
    let n: u128 = 20_000;
    let id = |i: u128| {
        ObjectId::from_str(&format!(
            "{:032x}",
            (0x0190_0000_0000_7000_8000_0000_0000_0000u128) | i
        ))
        .unwrap()
    };
    let page = id(n + 1);
    let els: BTreeMap<ObjectId, Element> = (0..n)
        .map(|i| {
            let mut e = rect_element(page, 0.0, 0.0);
            e.id = id(i);
            e.parent = Some(id((i + 1) % n)); // one long ring: a cycle through all of them
            (e.id, e)
        })
        .collect();
    let t = Instant::now();
    let eff = effective_parents(&els);
    assert!(t.elapsed().as_secs_f64() < 2.0, "took {:?}", t.elapsed());
    assert_eq!(
        eff.values().filter(|p| p.is_none()).count(),
        1,
        "cut once, at the lowest id"
    );
    assert!(eff[&id(0)].is_none());
}

#[tokio::test]
async fn move_page_before_itself_changes_nothing_and_equal_keys_are_split() {
    let p = pair().await;
    let first = p.a.page;
    let (_, x) = p.a.ed.add_page("X").await.unwrap();
    let (_, y) = p.b.ed.add_page("Y").await.unwrap(); // the same key as X, made apart
    p.sync().await;
    let ids = |b: fs_whiteboard_core::Board| b.pages.iter().map(|q| q.id).collect::<Vec<_>>();
    let before = ids(p.a.ed.get().await.unwrap());
    let same = p.a.ed.move_page(first, Some(first)).await.unwrap();
    assert_eq!(ids(same), before, "before itself is no move");
    let hi = if x > y { x } else { y };
    let lo = if x > y { y } else { x };
    assert_eq!(before, vec![first, lo, hi], "equal keys sort by id");
    let moved = p.a.ed.move_page(first, Some(hi)).await.unwrap();
    assert_eq!(
        ids(moved),
        vec![lo, first, hi],
        "directly before the page it was asked to go before"
    );
}

#[tokio::test]
async fn a_key_over_one_kib_is_refused_clearly() {
    let f = fixture().await;
    let (lo, hi) = (
        format!("V{}1", "1".repeat(1022)),
        format!("V{}2", "1".repeat(1022)),
    );
    assert_eq!((lo.len(), hi.len()), (1024, 1024));
    assert!(is_valid(&lo) && is_valid(&hi));
    let own = "application/vnd.freesynergy.whiteboard+json";
    let mut file = common::exported(&f.ed.export(None, &[], own).await.unwrap().bytes);
    let page = file["pages"][0].clone();
    let mk = |order: &str, x: f64| {
        let e = rect_element(ObjectId::new(), x, 0.0);
        let mut v = serde_json::to_value(&e).unwrap();
        v["order"] = json!(order);
        v["page"] = page["id"].clone();
        v
    };
    file["elements"] = json!([mk(&lo, 0.0), mk(&hi, 1.0), mk("W", 2.0)]);
    let b = f
        .repo
        .import(own, file.to_string().as_bytes())
        .await
        .unwrap();
    let ed = f.repo.open(b.id).await.unwrap().unwrap();
    let list = all(&*ed, b.pages[0].id).await;
    assert_eq!(list.len(), 3);
    let r = ed.reorder(&[list[2].id], Some(list[1].id)).await;
    assert!(matches!(r, Err(Error::TooLarge(_))), "{r:?}");
    assert_eq!(
        all(&*ed, b.pages[0].id)
            .await
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>(),
        list.iter().map(|e| e.id).collect::<Vec<_>>()
    );
    let _ = between(None, None);
}

#[tokio::test]
async fn ten_thousand_appends_give_keys_of_at_most_four_characters() {
    // measured: the longest of 10 000 appended keys has 4 characters (W1.. 2, X01.. 3, Y001.. 4)
    let mut key: Option<String> = None;
    let mut longest = 0;
    for _ in 0..10_000 {
        let k = between(key.as_deref(), None);
        assert!(key.as_ref().is_none_or(|p| p.as_str() < k.as_str()));
        longest = longest.max(k.len());
        key = Some(k);
    }
    assert_eq!(longest, 4, "keys grow with the logarithm of the appends");
}
