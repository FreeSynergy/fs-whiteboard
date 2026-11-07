//! Sample boards for the format tests.

use fs_objects::ObjectId;
use fs_whiteboard_core::{
    Anchor, Connector, Element, Kind, Point, Rect, ShapeKind, StrokePoint, StyleField,
};

use super::Fixture;

pub fn el(f: &Fixture, kind: Kind, x: f64) -> Element {
    Element::new(f.page, kind, Rect::new(x, 5.0, 40.0, 20.0))
}

/// A board with every kind, a group, a second page and a deleted element.
/// A board with every kind, a group, a second page and a deleted element.
pub async fn busy(f: &Fixture) -> Vec<ObjectId> {
    let ed = &*f.ed;
    let a = ed
        .add(el(
            f,
            Kind::Shape {
                shape: ShapeKind::Diamond,
            },
            0.0,
        ))
        .await
        .unwrap()
        .id;
    let b = ed
        .add(el(
            f,
            Kind::Note {
                text: "note\nline two".into(),
            },
            50.0,
        ))
        .await
        .unwrap()
        .id;
    let pts = vec![
        StrokePoint {
            x: 0.0,
            y: 0.0,
            pressure: Some(0.5),
        },
        StrokePoint {
            x: 1.0,
            y: 0.25,
            pressure: None,
        },
    ];
    let s = ed
        .add(el(f, Kind::Stroke { points: pts }, 100.0))
        .await
        .unwrap()
        .id;
    let link = Connector {
        source: Anchor {
            element: Some(a),
            point: Point::default(),
        },
        target: Anchor {
            element: Some(b),
            point: Point::default(),
        },
        label: "uses".into(),
        arrow_end: true,
    };
    let c = ed.add(el(f, Kind::Connector(link), 0.0)).await.unwrap().id;
    let t = ed
        .add(el(
            f,
            Kind::Text {
                text: "hello".into(),
            },
            150.0,
        ))
        .await
        .unwrap()
        .id;
    ed.add(el(
        f,
        Kind::Image {
            iri: "https://example.org/files/9".into(),
        },
        200.0,
    ))
    .await
    .unwrap();
    ed.add(el(
        f,
        Kind::Area {
            text: "Frame".into(),
        },
        250.0,
    ))
    .await
    .unwrap();
    let g = ed.group(&[s, t]).await.unwrap();
    ed.set_style(
        &[a],
        &[
            StyleField::Fill("#ff0000".into()),
            StyleField::Dashed(true),
            StyleField::Opacity(0.5),
        ],
    )
    .await
    .unwrap();
    ed.rotate(a, 0.5).await.unwrap();
    let (_, second) = ed.add_page("Second").await.unwrap();
    ed.add(Element::new(
        second,
        Kind::Text { text: "p2".into() },
        Rect::new(0.0, 0.0, 5.0, 5.0),
    ))
    .await
    .unwrap();
    let gone = ed
        .add(el(
            f,
            Kind::Text {
                text: "deleted".into(),
            },
            300.0,
        ))
        .await
        .unwrap()
        .id;
    ed.delete(&[gone]).await.unwrap();
    vec![a, b, s, c, t, g]
}

/// An export read back as JSON.
pub fn exported(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).unwrap()
}
