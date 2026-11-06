//! Regression: a connector's place comes from its ends, in the index, in `bounds` and in export.

mod common;

use common::{all, fixture, rect_element};
use fs_store::ObjectId;
use fs_whiteboard_core::{Anchor, Boards, Connector, Element, Kind, Point, Rect, Unit};

async fn linked(f: &common::Fixture) -> (ObjectId, ObjectId, ObjectId) {
    let (a, b) = (f.rect(0.0, 0.0).await, f.rect(500.0, 0.0).await);
    let k = Connector {
        source: Anchor {
            element: Some(a),
            point: Point::default(),
        },
        target: Anchor {
            element: Some(b),
            point: Point::default(),
        },
        label: String::new(),
        arrow_end: true,
    };
    // the stored frame is nowhere near the ends
    let c =
        f.ed.add(Element::new(
            f.page,
            Kind::Connector(k),
            Rect::new(9000.0, 9000.0, 1.0, 1.0),
        ))
        .await
        .unwrap()
        .id;
    (a, b, c)
}

#[tokio::test]
async fn the_index_and_bounds_follow_the_ends() {
    let f = fixture().await;
    let (a, b, c) = linked(&f).await;
    let ids = |area| common::ids_in(&*f.ed, f.page, area);
    assert_eq!(
        f.ed.bounds(f.page).await.unwrap().unwrap(),
        Rect::new(0.0, 0.0, 510.0, 10.0)
    );
    f.ed.move_by(&[b], 0.0, 300.0).await.unwrap();
    assert!(
        ids(Rect::new(250.0, 100.0, 2.0, 2.0)).await.contains(&c),
        "it follows the element that moved"
    );
    assert!(!ids(Rect::new(250.0, 0.0, 1.0, 1.0)).await.contains(&c));
    let shown = f.ed.get_element(c).await.unwrap().unwrap();
    assert_eq!((shown.frame.width, shown.frame.height), (500.0, 300.0));
    let _ = a;
}

#[tokio::test]
async fn export_draws_a_deleted_end_at_its_last_frame_and_can_be_imported_again() {
    let f = fixture().await;
    let (a, b, c) = linked(&f).await;
    f.ed.move_by(&[b], 0.0, 40.0).await.unwrap();
    f.ed.delete(&[b]).await.unwrap();
    let svg = String::from_utf8(
        f.ed.export(Some(f.page), &[], "image/svg+xml")
            .await
            .unwrap()
            .bytes,
    )
    .unwrap();
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let line = doc.descendants().find(|n| n.has_tag_name("line")).unwrap();
    let at = |n: &str| line.attribute(n).unwrap().parse::<f64>().unwrap();
    assert_eq!(
        (at("x1"), at("y1"), at("x2"), at("y2")),
        (5.0, 5.0, 505.0, 45.0),
        "{svg}"
    );

    let own = "application/vnd.freesynergy.whiteboard+json";
    let blob = f.ed.export(None, &[], own).await.unwrap();
    let copy = f.repo.import(own, &blob.bytes).await.unwrap();
    let ed2 = f.repo.open(copy.id).await.unwrap().unwrap();
    let list = all(&*ed2, copy.pages[0].id).await;
    let Kind::Connector(k) = &list
        .iter()
        .find(|e| matches!(e.kind, Kind::Connector(_)))
        .unwrap()
        .kind
    else {
        panic!()
    };
    assert_eq!(
        (k.target.element, k.target.point),
        (None, Point { x: 505.0, y: 45.0 }),
        "a free end at its last position"
    );
    assert!(k.source.element.is_some());

    // a selection that holds the connector but not what it points at, or a member without its group
    let part = f.ed.export(Some(f.page), &[c], own).await.unwrap();
    let copy = f.repo.import(own, &part.bytes).await.unwrap();
    let ed3 = f.repo.open(copy.id).await.unwrap().unwrap();
    let list = all(&*ed3, copy.pages[0].id).await;
    assert_eq!(list.len(), 1);
    let Kind::Connector(k) = &list[0].kind else {
        panic!()
    };
    assert_eq!((k.source.element, k.target.element), (None, None));
    let g = f.ed.group(&[a]).await.unwrap();
    let inner = f.ed.export(Some(f.page), &[a], own).await.unwrap();
    f.repo
        .import(own, &inner.bytes)
        .await
        .expect("a member exported without its group");
    let _ = (g, rect_element, Unit::Frame);
}
