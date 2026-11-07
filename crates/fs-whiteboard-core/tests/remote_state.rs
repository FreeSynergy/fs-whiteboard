//! Regression: stray clocks, and apply_remote next to local edits.

mod common;

use common::{element_object, fixture};
use fs_objects::ObjectId;
use fs_whiteboard_core::merge::{merge_board, merge_element};
use fs_whiteboard_core::{BoardDoc, ElementDoc, Unit};
use serde_json::json;

#[test]
fn a_stray_clock_on_an_absent_unit_does_not_break_commutativity() {
    let mk = |stray: bool| -> ElementDoc {
        let f = serde_json::from_value::<ElementDoc>(json!({
            "id": "01900000-0000-7000-8000-000000000001", "board": "01900000-0000-7000-8000-000000000002",
            "page": "01900000-0000-7000-8000-000000000003", "parent": null, "order": "V",
            "frame": {"x":0.0,"y":0.0,"width":1.0,"height":1.0}, "angle": 0.0,
            "style": {"stroke":"","fill":"","line_width":1.0,"dashed":false,"opacity":1.0,"font_family":"","font_size":16.0},
            "locked": false, "deleted": false, "label": "", "kind": {"type":"shape","shape":"rectangle"},
            "fs:clocks": {"frame": [1, "a"]}
        }));
        let mut d = f.unwrap();
        if stray {
            d.clocks
                .insert(Unit::Source, fs_whiteboard_core::Clock(9, "z".into()));
        }
        d
    };
    let (a, b) = (mk(true), mk(false));
    assert_eq!(
        merge_element(&a, &b).unwrap(),
        merge_element(&b, &a).unwrap()
    );
    assert!(
        !merge_element(&a, &b)
            .unwrap()
            .clocks
            .contains_key(&Unit::Source)
    );
    // the same for boards
    let page = fs_whiteboard_core::Page {
        id: ObjectId::new(),
        name: "p".into(),
        order: "V".into(),
        deleted: false,
    };
    let x = BoardDoc::fresh("b", page, "a");
    let mut y = x.clone();
    y.clocks.insert(
        format!("{}/name", ObjectId::new()),
        fs_whiteboard_core::Clock(9, "z".into()),
    );
    assert_eq!(merge_board(&x, &y), merge_board(&y, &x));
}

#[tokio::test]
async fn applying_while_editing_keeps_state_and_store_equal() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let remote = element_object(&f, id).await;
    let fut_edits = async {
        for i in 0..20 {
            f.ed.move_by(&[id], f64::from(i), 0.0).await.unwrap();
        }
    };
    let fut_remote = async {
        for _ in 0..20 {
            f.repo.apply_remote(remote.clone()).await.unwrap();
        }
    };
    tokio::join!(fut_edits, fut_remote);
    let stored: ElementDoc = serde_json::from_value(element_object(&f, id).await.content).unwrap();
    assert_eq!(
        stored.element.frame,
        common::frame_of(&*f.ed, id).await,
        "the state follows the store"
    );
    assert_eq!(stored.element.frame.x, 190.0);
}
