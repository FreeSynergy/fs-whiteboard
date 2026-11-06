//! Regression: coordinates and sizes stay within +-1e15 at every entry; nothing near the limits
//! freezes, vanishes on reload or breaks the export.

mod common;

use common::{all, element_object, fixture, rect_element};
use fs_whiteboard_core::{Boards, Rect, WhiteboardRepo};
use proptest::prelude::*;
use serde_json::json;

const OWN: &str = "application/vnd.freesynergy.whiteboard+json";

fn big() -> impl Strategy<Value = f64> {
    prop_oneof![
        -1e17..1e17f64,
        -1e308..1e308f64,
        -1e15..1e15f64,
        0.0..10.0f64
    ]
}

proptest! {
    #![proptest_config(common::cfg(24))]

    #[test]
    fn nothing_near_the_limits_freezes_vanishes_or_breaks_export(
        frames in prop::collection::vec((big(), big(), big(), big(), -7.0..7.0f64), 1..5),
        shift in big(),
    ) {
        tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
            let f = fixture().await;
            let mut ids = Vec::new();
            for (x, y, w, h, a) in frames {
                let mut e = rect_element(f.page, x, y);
                e.frame = Rect::new(x, y, w.abs(), h.abs());
                e.angle = a;
                if let Ok(e) = f.ed.add(e).await {
                    ids.push(e.id);
                }
            }
            let _ = f.ed.move_by(&ids, shift, shift).await;
            let _ = f.ed.rotate(ids.first().copied().unwrap_or(f.page), shift).await;
            if !ids.is_empty() {
                let _ = f.ed.group(&ids).await;
            }
            // after a restart everything reads, and the export can be imported again
            let again = WhiteboardRepo::new(f.store.clone(), "me");
            let ed = again.open(f.board).await.unwrap().unwrap();
            assert!(again.problems().is_empty(), "{:?}", again.problems());
            let kept = all(&*ed, f.page).await.len();
            assert!(kept >= ids.len().min(1) || ids.is_empty());
            // one check for both ways: what is written can be read again, or nothing is written
            match ed.export(None, &[], OWN).await {
                Ok(blob) => {
                    let b = again.import(OWN, &blob.bytes).await.unwrap();
                    let ed2 = again.open(b.id).await.unwrap().unwrap();
                    assert_eq!(all(&*ed2, b.pages[0].id).await.len(), kept);
                }
                Err(e) => assert!(matches!(e, fs_whiteboard_core::Error::Invalid(_)), "{e:?}"),
            }
        });
    }
}

#[tokio::test]
async fn coordinates_beyond_1e15_are_refused_everywhere() {
    let f = fixture().await;
    let a = f.rect(0.0, 0.0).await;
    let over = 1.0e15 + 1.0e5;
    assert!(
        f.ed.set_frame(a, Rect::new(over, 0.0, 1.0, 1.0))
            .await
            .is_err()
    );
    assert!(
        f.ed.set_frame(a, Rect::new(0.0, 0.0, over, 1.0))
            .await
            .is_err()
    );
    assert!(f.ed.move_by(&[a], over, 0.0).await.is_err());
    f.ed.set_frame(a, Rect::new(1.0e15, -1.0e15, 1.0e15, 1.0))
        .await
        .unwrap();
    let mut file = common::exported(&f.ed.export(None, &[], OWN).await.unwrap().bytes);
    file["elements"][0]["frame"]["x"] = json!(over);
    assert!(
        f.repo
            .import(OWN, file.to_string().as_bytes())
            .await
            .is_err()
    );
    let mut o = element_object(&f, a).await;
    o.content["frame"]["x"] = json!(over);
    assert!(f.repo.apply_remote(o).await.is_err());
}

#[tokio::test]
async fn strokes_stay_within_the_limits_and_far_connectors_export() {
    use fs_whiteboard_core::{Anchor, Connector, Element, Kind, StrokePoint};
    let f = fixture().await;
    let pt = |x: f64| StrokePoint {
        x,
        y: 0.0,
        pressure: None,
    };
    for bad in [1e15 + 1e5, -2e15, 1e300, f64::MAX] {
        let k = Kind::Stroke {
            points: vec![pt(0.0), pt(bad)],
        };
        assert!(
            f.ed.add(Element::new(f.page, k, Rect::new(0.0, 0.0, 1.0, 1.0)))
                .await
                .is_err()
        );
    }
    let k = Kind::Stroke {
        points: vec![pt(1e15), pt(-1e15)],
    };
    let wide = Rect::new(-1e15, 0.0, 1e15, 1.0);
    f.ed.add(Element::new(f.page, k, wide)).await.unwrap();
    let (a, b) = (f.rect(-1e15, 0.0).await, f.rect(1e15, 0.0).await);
    let ends = Connector {
        source: Anchor {
            element: Some(a),
            ..Anchor::default()
        },
        target: Anchor {
            element: Some(b),
            ..Anchor::default()
        },
        ..Connector::default()
    };
    f.ed.add(Element::new(
        f.page,
        Kind::Connector(ends),
        Rect::new(0.0, 0.0, 1.0, 1.0),
    ))
    .await
    .unwrap();
    let blob = f.ed.export(None, &[], OWN).await.unwrap();
    let copy = f.repo.import(OWN, &blob.bytes).await.unwrap();
    let ed = f.repo.open(copy.id).await.unwrap().unwrap();
    assert_eq!(all(&*ed, copy.pages[0].id).await.len(), 4);
    let svg =
        f.ed.export(Some(f.page), &[], "image/svg+xml")
            .await
            .unwrap();
    let text = String::from_utf8(svg.bytes).unwrap();
    assert!(!text.contains("inf") && !text.contains("NaN"), "{text}");
    roxmltree::Document::parse(&text).unwrap();
}
