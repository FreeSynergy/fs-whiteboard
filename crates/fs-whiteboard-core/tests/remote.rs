//! Regression: what `apply_remote` refuses, and what a late remote cannot undo.

mod common;

use common::{add_rect, element_object, pair, raw, replicate};
use fs_objects::{ObjectId, StoredObject};
use fs_whiteboard_core::merge::merge_element;
use fs_whiteboard_core::{Boards, ElementDoc, Error, StyleField};
use serde_json::{Value, json};

fn refused(r: Result<(), Error>) {
    match r {
        Err(Error::Invalid(_) | Error::TooLarge(_)) => {}
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn an_envelope_id_that_differs_from_the_content_id_is_refused() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    p.a.ed
        .set_style(&[id], &[StyleField::Fill("#aaaaaa".into())])
        .await
        .unwrap();
    let mut o = element_object(&p.a, id).await;
    // all clocks zero: it would lose every unit if merged, and must not even be tried
    for c in o.content["fs:clocks"].as_object_mut().unwrap().values_mut() {
        *c = json!([0, ""]);
    }
    o.content["fill"] = json!("");
    o.envelope.id = ObjectId::new();
    let before = raw(&p.b.store, id).await;
    refused(p.b.repo.apply_remote(o).await);
    assert_eq!(before, raw(&p.b.store, id).await);
}

#[tokio::test]
async fn clocks_too_large_and_other_identity_are_refused() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    let base = element_object(&p.a, id).await;
    let mut o = base.clone();
    o.content["fs:clocks"]["frame"] = json!([u64::MAX, "x"]);
    refused(p.b.repo.apply_remote(o).await);
    let mut o = base.clone();
    o.content["fs:clocks"]["frame"] = json!([u64::MAX / 2 + 1, "x"]);
    refused(p.b.repo.apply_remote(o).await);
    let mut o = base.clone();
    o.content["fs:clocks"]["frame"] = json!([u64::MAX / 2, "x"]);
    p.b.repo.apply_remote(o).await.unwrap();
    p.b.ed.move_by(&[id], 1.0, 1.0).await.unwrap(); // the next local counter does not wrap
    let mut o = base.clone();
    o.content["kind"] = json!({"type": "text", "text": "x"});
    refused(p.b.repo.apply_remote(o).await);
    let mut o = base.clone();
    o.content["board"] = json!(ObjectId::new());
    refused(p.b.repo.apply_remote(o).await);
    // and the pure merge refuses it as well
    let (a, b): (ElementDoc, ElementDoc) = (
        serde_json::from_value(base.content.clone()).unwrap(),
        serde_json::from_value(o_with_kind(&base)).unwrap(),
    );
    assert!(merge_element(&a, &b).is_err());
}

fn o_with_kind(o: &StoredObject) -> Value {
    let mut v = o.content.clone();
    v["kind"] = json!({"type": "shape", "shape": "ellipse"});
    v
}

#[tokio::test]
async fn bad_order_keys_are_refused_in_elements_and_pages() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    let base = element_object(&p.a, id).await;
    for bad in [
        "é".to_string(),
        "~".into(),
        String::new(),
        "a0".into(),
        "x".repeat(5 * 1024 * 1024),
        "V".repeat(1025),
    ] {
        let mut o = base.clone();
        o.content["order"] = json!(bad);
        refused(p.b.repo.apply_remote(o).await);
    }
    let board = p.a.store.get(p.a.board).await.unwrap().unwrap();
    let mut o = board.clone();
    o.content["pages"][p.a.page.to_string()]["order"] = json!("é");
    refused(p.b.repo.apply_remote(o).await);
    // the editor still works after all that
    p.b.ed.reorder(&[id], None).await.unwrap();
    add_rect(&*p.b.ed, p.b.page, 1.0, 1.0).await;
}

#[tokio::test]
async fn remote_boards_are_validated() {
    let p = pair().await;
    let board = p.a.store.get(p.a.board).await.unwrap().unwrap();
    let mut o = board.clone();
    o.content["pages"] = json!({});
    refused(p.b.repo.apply_remote(o).await);
    let mut o = board.clone();
    o.content["name"] = json!("n".repeat(3 * 1024 * 1024));
    refused(p.b.repo.apply_remote(o).await);
    let mut o = board.clone();
    o.content["name"] = json!("  ");
    refused(p.b.repo.apply_remote(o).await);
}

#[tokio::test]
async fn a_late_remote_cannot_bring_a_deleted_board_back() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    let old_board = p.a.store.get(p.a.board).await.unwrap().unwrap();
    let old_element = element_object(&p.a, id).await;
    p.b.repo.delete(p.b.board).await.unwrap();
    p.b.repo.apply_remote(old_board).await.unwrap();
    p.b.repo.apply_remote(old_element).await.unwrap();
    replicate(&p.a.store, &p.b.repo, p.a.board).await;
    assert!(p.b.repo.open(p.b.board).await.unwrap().is_none());
    assert_eq!(
        p.b.repo.list(None).await.unwrap().items,
        [] as [fs_whiteboard_core::Board; 0]
    );
    let left = p.b.store.query(&fs_objects::Query::new()).await.unwrap();
    assert_eq!(left.len(), 1, "only the marker");
}
