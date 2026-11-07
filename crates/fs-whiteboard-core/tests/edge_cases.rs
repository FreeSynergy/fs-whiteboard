//! Regression: connectors in groups, the feed after a delete, keys at the limit, remote markers,
//! problem reports, coordinate limits, missing clocks.

mod common;

use std::time::Duration;

use common::{add_rect, all, element_object, fixture, pair, rect_element};
use fs_objects::{Envelope, Query, StoredObject};
use fs_whiteboard_core::merge::merge_element;
use fs_whiteboard_core::{
    Anchor, BOARD_CONTEXT, Boards, Change, Connector, ELEMENT_TYPE, Element, ElementDoc, Error,
    Kind, Point, Rect, Unit, WhiteboardFeed, WhiteboardRepo,
};
use serde_json::json;

const OWN: &str = "application/vnd.freesynergy.whiteboard+json";

#[tokio::test]
async fn a_connector_in_a_group_counts_with_its_ends_not_its_stored_frame() {
    let f = fixture().await;
    let (a, b) = (f.rect(1000.0, 0.0).await, f.rect(1100.0, 0.0).await);
    let k = Connector {
        source: Anchor {
            element: Some(a),
            point: Point::default(),
        },
        target: Anchor {
            element: Some(b),
            point: Point::default(),
        },
        ..Connector::default()
    };
    let c =
        f.ed.add(Element::new(
            f.page,
            Kind::Connector(k),
            Rect::new(0.0, 0.0, 1.0, 1.0),
        ))
        .await
        .unwrap()
        .id;
    let g = f.ed.group(&[a, b, c]).await.unwrap();
    let frame = f.ed.get_element(g).await.unwrap().unwrap().frame;
    assert_eq!((frame.x, frame.right()), (1000.0, 1110.0), "{frame:?}");
}

#[tokio::test]
async fn deleting_a_board_emits_deletes_only() {
    let f = fixture().await;
    f.rect(0.0, 0.0).await;
    let mut sub = f.repo.subscribe(f.board).await.unwrap();
    f.repo.delete(f.board).await.unwrap();
    let mut seen = Vec::new();
    while let Ok(Some(c)) = tokio::time::timeout(Duration::from_millis(300), sub.next()).await {
        seen.push(c);
    }
    assert_eq!(seen.len(), 2, "{seen:?}");
    assert!(
        seen.iter().all(|c| matches!(c, Change::Deleted(_))),
        "{seen:?}"
    );
    assert!(matches!(seen.last(), Some(Change::Deleted(id)) if *id == f.board));
}

#[tokio::test]
async fn add_next_to_a_key_at_the_limit_is_refused_before_writing() {
    let f = fixture().await;
    let mut file = common::exported(&f.ed.export(None, &[], OWN).await.unwrap().bytes);
    let mut v = serde_json::to_value(rect_element(fs_objects::ObjectId::new(), 0.0, 0.0)).unwrap();
    v["order"] = json!("z".repeat(1024));
    v["page"] = file["pages"][0]["id"].clone();
    file["elements"] = json!([v]);
    let b = f
        .repo
        .import(OWN, file.to_string().as_bytes())
        .await
        .unwrap();
    let ed = f.repo.open(b.id).await.unwrap().unwrap();
    let page = b.pages[0].id;
    let r = ed.add(rect_element(page, 1.0, 1.0)).await;
    assert!(matches!(r, Err(Error::TooLarge(_))), "{r:?}");
    assert_eq!(all(&*ed, page).await.len(), 1);
    let r = ed.copy(&[all(&*ed, page).await[0].id], 1.0, 1.0).await;
    assert!(matches!(r, Err(Error::TooLarge(_))), "{r:?}");
}

#[tokio::test]
async fn a_remote_marker_deletes_the_board_here_too() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    let old = element_object(&p.a, id).await;
    p.b.repo.delete(p.b.board).await.unwrap();
    let marker = p.b.store.get(p.b.board).await.unwrap().unwrap();
    p.a.repo.apply_remote(marker).await.unwrap();
    assert!(p.a.repo.open(p.a.board).await.unwrap().is_none());
    assert!(p.a.repo.list(None).await.unwrap().items.is_empty());
    p.a.repo.apply_remote(old).await.unwrap(); // a late remote is dropped on purpose
    assert_eq!(
        p.a.store.query(&Query::new()).await.unwrap().len(),
        1,
        "only the marker"
    );
}

fn garbage_board() -> StoredObject {
    let mut env = Envelope::new("Collection", "garbage");
    env.context = Some(BOARD_CONTEXT.into());
    StoredObject::new(env, "application/json", json!({"pages": 7}))
}

#[tokio::test]
async fn problems_are_listed_once_per_object_and_capped() {
    let f = fixture().await;
    let bad = garbage_board();
    let bad_id = bad.envelope.id;
    f.store.put(bad, "x").await.unwrap();
    for _ in 0..5 {
        f.repo.list(None).await.unwrap();
    }
    assert_eq!(
        f.repo
            .problems()
            .iter()
            .filter(|(i, _)| *i == bad_id)
            .count(),
        1
    );
    for _ in 0..130 {
        f.store.put(garbage_board(), "x").await.unwrap();
    }
    f.repo.list(None).await.unwrap();
    assert!(f.repo.problems().len() <= 100);
}

#[tokio::test]
async fn a_valid_remote_object_replaces_a_broken_local_one() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let good = element_object(&f, id).await;
    let mut bad = good.clone();
    bad.content["frame"] = json!(null);
    f.store.put(bad, "x").await.unwrap();
    let repo = WhiteboardRepo::new(f.store.clone(), "me");
    let ed = repo.open(f.board).await.unwrap().unwrap();
    assert!(all(&*ed, f.page).await.is_empty());
    assert_eq!(repo.problems().len(), 1);
    repo.apply_remote(good).await.unwrap();
    assert_eq!(all(&*ed, f.page).await.len(), 1);
    assert!(repo.problems().is_empty());
    let _ = ELEMENT_TYPE;
}

#[tokio::test]
async fn a_missing_clock_is_the_lowest_and_written_out() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let obj = element_object(&f, id).await;
    let full: ElementDoc = serde_json::from_value(obj.content.clone()).unwrap();
    let mut forged = full.clone();
    forged.clocks.remove(&Unit::Frame);
    let (ab, ba) = (
        merge_element(&forged, &full).unwrap(),
        merge_element(&full, &forged).unwrap(),
    );
    assert_eq!(
        serde_json::to_string(&ab).unwrap(),
        serde_json::to_string(&ba).unwrap()
    );
    assert_eq!(
        ab.clocks.len(),
        full.clocks.len(),
        "the missing clock is written out"
    );
    let mut o = obj.clone();
    o.content["fs:clocks"]
        .as_object_mut()
        .unwrap()
        .remove("frame");
    f.repo.apply_remote(o).await.unwrap();
    assert!(element_object(&f, id).await.content["fs:clocks"]["frame"].is_array());
}
