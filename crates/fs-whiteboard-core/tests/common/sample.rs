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
pub async fn busy(fx: &Fixture) -> Vec<ObjectId> {
    let ed = &*fx.ed;
    let diamond = ed
        .add(el(
            fx,
            Kind::Shape {
                shape: ShapeKind::Diamond,
            },
            0.0,
        ))
        .await
        .unwrap()
        .id;
    let note = ed
        .add(el(
            fx,
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
    let stroke = ed
        .add(el(fx, Kind::Stroke { points: pts }, 100.0))
        .await
        .unwrap()
        .id;
    let link = Connector {
        source: Anchor {
            element: Some(diamond),
            point: Point::default(),
        },
        target: Anchor {
            element: Some(note),
            point: Point::default(),
        },
        label: "uses".into(),
        arrow_end: true,
    };
    let link_el = ed.add(el(fx, Kind::Connector(link), 0.0)).await.unwrap().id;
    let text = ed
        .add(el(
            fx,
            Kind::Text {
                text: "hello".into(),
            },
            150.0,
        ))
        .await
        .unwrap()
        .id;
    ed.add(el(
        fx,
        Kind::Image {
            iri: "https://example.org/files/9".into(),
        },
        200.0,
    ))
    .await
    .unwrap();
    ed.add(el(
        fx,
        Kind::Area {
            text: "Frame".into(),
        },
        250.0,
    ))
    .await
    .unwrap();
    let grp = ed.group(&[stroke, text]).await.unwrap();
    ed.set_style(
        &[diamond],
        &[
            StyleField::Fill("#ff0000".into()),
            StyleField::Dashed(true),
            StyleField::Opacity(0.5),
        ],
    )
    .await
    .unwrap();
    ed.rotate(diamond, 0.5).await.unwrap();
    extras(fx).await;
    vec![diamond, note, stroke, link_el, text, grp]
}

/// A second page and a deleted element, so the formats see both.
async fn extras(fx: &Fixture) {
    let ed = &*fx.ed;
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
            fx,
            Kind::Text {
                text: "deleted".into(),
            },
            300.0,
        ))
        .await
        .unwrap()
        .id;
    ed.delete(&[gone]).await.unwrap();
}

/// An export read back as JSON.
pub fn exported(bytes: &[u8]) -> serde_json::Value {
    serde_json::from_slice(bytes).unwrap()
}
