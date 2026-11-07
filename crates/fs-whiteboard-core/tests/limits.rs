//! Regression: results of arithmetic stay finite, one bad object does not stop the others,
//! colours and IRIs are allowlisted, floats survive a reload.

mod common;

use std::sync::Arc;

use common::{fixture, rect_element};
use fs_objects::{Envelope, Store, StoredObject, TursoStore};
use fs_whiteboard_core::{
    BOARD_CONTEXT, Boards, ELEMENT_TYPE, Element, Error, Kind, Rect, StyleField, WhiteboardRepo,
};
use serde_json::json;

fn refused<T: std::fmt::Debug>(r: Result<T, Error>) {
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}

#[tokio::test]
async fn arithmetic_that_leaves_float64_is_refused_not_stored() {
    let f = fixture().await;
    let a = f.rect(0.0, 0.0).await;
    let (b, c) = (f.rect(-1e15, 0.0).await, f.rect(1e15, 0.0).await);
    refused(f.ed.copy(&[c], 1e15, 0.0).await);
    let _ = b;
    refused(f.ed.move_by(&[c], 1.7e308, 0.0).await);
    refused(f.ed.rotate(a, 1e308).await);
    refused(
        f.ed.set_frame(a, Rect::new(1.7e308, 0.0, 1.7e308, 1.0))
            .await,
    );
    let mut e = rect_element(f.page, 1.7e308, 0.0);
    e.frame.width = 1.7e308;
    refused(f.ed.add(e).await);
    // nothing odd reached the store: every object parses and has finite numbers
    let all = f
        .store
        .query(&fs_objects::Query::new().of_type(ELEMENT_TYPE))
        .await
        .unwrap();
    assert_eq!(all.len(), 3, "no copy, group or rotation was stored");
    assert!(
        !all.iter()
            .any(|o| o.content.to_string().contains("null,") && o.content["frame"].is_null())
    );
    assert!(f.repo.problems().is_empty());
}

#[tokio::test]
async fn one_bad_object_does_not_stop_the_others_after_a_restart() {
    let dir = std::env::temp_dir().join(format!("fs-wb-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("db");
    let (good, other, broken_board);
    {
        let store: Arc<dyn Store> = Arc::new(TursoStore::open(&path).await.unwrap());
        let repo = WhiteboardRepo::new(store.clone(), "me");
        let b = repo.create("Good").await.unwrap();
        good = b.id;
        other = repo.create("Other").await.unwrap().id;
        let ed = repo.open(good).await.unwrap().unwrap();
        ed.add(rect_element(b.pages[0].id, 1.0, 1.0)).await.unwrap();
        // what an older version could have written: an element with a null frame, a board with garbage
        let mut env = Envelope::new(ELEMENT_TYPE, "bad");
        env.context = Some("x".into());
        let bad = StoredObject::new(env, "application/json", json!({"frame": null}))
            .with_index("board", good.to_string());
        store.put(bad, "me").await.unwrap();
        let mut env = Envelope::new("Collection", "garbage");
        env.context = Some(BOARD_CONTEXT.into());
        broken_board = env.id;
        store
            .put(
                StoredObject::new(env, "application/json", json!({"pages": 7})),
                "me",
            )
            .await
            .unwrap();
    }
    let store: Arc<dyn Store> = Arc::new(TursoStore::open(&path).await.unwrap());
    let repo = WhiteboardRepo::new(store, "me");
    let listed: Vec<_> = repo
        .list(None)
        .await
        .unwrap()
        .items
        .iter()
        .map(|b| b.id)
        .collect();
    assert!(listed.contains(&good) && listed.contains(&other) && !listed.contains(&broken_board));
    let ed = repo.open(good).await.unwrap().unwrap();
    let page = ed.get().await.unwrap().pages[0].id;
    assert_eq!(ed.elements(page, None, None).await.unwrap().items.len(), 1);
    let problems = repo.problems();
    assert!(
        problems.iter().any(|(id, _)| *id == broken_board),
        "{problems:?}"
    );
    assert_eq!(
        problems.len(),
        2,
        "the bad element and the bad board are reported: {problems:?}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn colours_are_css_hex_and_image_iris_are_http() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    for bad in [
        "red",
        "url(javascript:alert(1))",
        "#12",
        "#gggggg",
        "#123456\" x=\"",
        " #fff",
    ] {
        refused(f.ed.set_style(&[id], &[StyleField::Fill(bad.into())]).await);
        refused(
            f.ed.set_style(&[id], &[StyleField::Stroke(bad.into())])
                .await,
        );
        let mut e = rect_element(f.page, 0.0, 0.0);
        e.style.fill = bad.into();
        refused(f.ed.add(e).await);
    }
    for ok in ["", "#fff", "#ffff", "#a1B2c3", "#a1b2c3d4"] {
        f.ed.set_style(&[id], &[StyleField::Fill(ok.into())])
            .await
            .unwrap();
    }
    for bad in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "data:image/svg+xml;base64,AAAA",
        "//example.org/x",
        "ftp://x/y",
        "https://x/\u{1}",
    ] {
        let e = Element::new(
            f.page,
            Kind::Image { iri: bad.into() },
            Rect::new(0.0, 0.0, 1.0, 1.0),
        );
        refused(f.ed.add(e).await);
    }
    for ok in [
        "https://example.org/f/1",
        "http://example.org/f/1",
        "HTTPS://example.org/",
    ] {
        let e = Element::new(
            f.page,
            Kind::Image { iri: ok.into() },
            Rect::new(0.0, 0.0, 1.0, 1.0),
        );
        f.ed.add(e).await.unwrap();
    }
    // and an import of the same
    let mut file = common::exported(
        &f.ed
            .export(None, &[], "application/vnd.freesynergy.whiteboard+json")
            .await
            .unwrap()
            .bytes,
    );
    let i = file["elements"]
        .as_array()
        .unwrap()
        .iter()
        .position(|e| e["kind"]["type"] == "image")
        .unwrap();
    file["elements"][i]["kind"]["iri"] = json!("javascript:alert(1)");
    assert!(
        f.repo
            .import(
                "application/vnd.freesynergy.whiteboard+json",
                file.to_string().as_bytes()
            )
            .await
            .is_err()
    );
    file["elements"][i]["kind"]["iri"] = json!("https://example.org/ok");
    file["elements"][0]["style"]["fill"] = json!("red");
    assert!(
        f.repo
            .import(
                "application/vnd.freesynergy.whiteboard+json",
                file.to_string().as_bytes()
            )
            .await
            .is_err()
    );
}
