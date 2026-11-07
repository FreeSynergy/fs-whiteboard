//! Regression: `apply_remote` touches only this program's objects; the front key ignores hidden
//! elements.

mod common;

use common::{all, element_object, fixture, rect_element};
use fs_objects::{Envelope, ObjectId, StoredObject};
use fs_whiteboard_core::{BOARD_CONTEXT, Boards, ELEMENT_TYPE, Error};
use serde_json::json;

fn foreign(object_type: &str, context: Option<&str>) -> StoredObject {
    let mut env = Envelope::new(object_type, "someone else's");
    env.context = context.map(String::from);
    StoredObject::new(env, "application/json", json!({"mine": true}))
}

fn refused(r: Result<(), Error>) {
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}

#[tokio::test]
async fn a_forged_marker_deletes_only_a_board_of_this_program() {
    let f = fixture().await;
    let other = foreign("Note", Some("notes"));
    let id = other.envelope.id;
    f.store.put(other, "x").await.unwrap();
    let mut marker = foreign("WhiteboardTombstone", Some("whiteboard-tombstone"));
    marker.envelope.id = id;
    refused(f.repo.apply_remote(marker).await);
    assert!(
        f.store.get(id).await.unwrap().unwrap().content["mine"]
            .as_bool()
            .unwrap(),
        "untouched"
    );
    // a marker with the wrong context, and one aimed at an element
    let mut marker = foreign("WhiteboardTombstone", Some("elsewhere"));
    marker.envelope.id = f.board;
    refused(f.repo.apply_remote(marker).await);
    let e = f.rect(0.0, 0.0).await;
    let mut marker = foreign("WhiteboardTombstone", Some("whiteboard-tombstone"));
    marker.envelope.id = e;
    refused(f.repo.apply_remote(marker).await);
    assert!(f.store.get(e).await.unwrap().unwrap().envelope.object_type == ELEMENT_TYPE);
    assert!(f.repo.open(f.board).await.unwrap().is_some());
}

#[tokio::test]
async fn remote_objects_do_not_overwrite_another_programs_object() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let good = element_object(&f, id).await;
    // the id is taken by another program's object
    let other = foreign("Note", Some("notes"));
    let taken = other.envelope.id;
    f.store.put(other, "x").await.unwrap();
    let mut el = good.clone();
    el.envelope.id = taken;
    el.content["id"] = json!(taken);
    refused(f.repo.apply_remote(el).await);
    let mut board = f.store.get(f.board).await.unwrap().unwrap();
    board.envelope.id = taken;
    refused(f.repo.apply_remote(board).await);
    assert!(
        f.store.get(taken).await.unwrap().unwrap().content["mine"]
            .as_bool()
            .unwrap()
    );
    // incoming objects that only look like ours
    let mut board = f.store.get(f.board).await.unwrap().unwrap();
    board.envelope.id = ObjectId::new();
    board.envelope.context = Some("notes".into());
    refused(f.repo.apply_remote(board).await);
    // an element over a board, a board over an element
    let mut el = good.clone();
    el.envelope.id = f.board;
    el.content["id"] = json!(f.board);
    refused(f.repo.apply_remote(el).await);
    let mut board = f.store.get(f.board).await.unwrap().unwrap();
    board.envelope.id = id;
    refused(f.repo.apply_remote(board).await);
    assert_eq!(
        f.store
            .get(f.board)
            .await
            .unwrap()
            .unwrap()
            .envelope
            .context
            .as_deref(),
        Some(BOARD_CONTEXT)
    );
}

#[tokio::test]
async fn a_hidden_holder_of_a_long_key_does_not_block_add() {
    let f = fixture().await;
    let own = "application/vnd.freesynergy.whiteboard+json";
    let mut file = common::exported(&f.ed.export(None, &[], own).await.unwrap().bytes);
    let mut v = serde_json::to_value(rect_element(ObjectId::new(), 0.0, 0.0)).unwrap();
    v["order"] = json!("z".repeat(1024));
    v["deleted"] = json!(true);
    v["page"] = file["pages"][0]["id"].clone();
    file["elements"] = json!([v]);
    let b = f
        .repo
        .import(own, file.to_string().as_bytes())
        .await
        .unwrap();
    let ed = f.repo.open(b.id).await.unwrap().unwrap();
    let page = b.pages[0].id;
    assert!(all(&*ed, page).await.is_empty());
    for i in 0..3 {
        ed.add(rect_element(page, f64::from(i), 0.0)).await.unwrap();
    }
    assert_eq!(all(&*ed, page).await.len(), 3, "the board is usable");
    let ids: Vec<_> = all(&*ed, page).await.iter().map(|e| e.id).collect();
    ed.reorder(&[ids[0]], None).await.unwrap();
    ed.copy(&ids[..1], 1.0, 1.0).await.unwrap();
}
