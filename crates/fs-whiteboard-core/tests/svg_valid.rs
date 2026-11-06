//! Regression: every kind of element gives SVG that a real XML parser accepts (no duplicate
//! attributes), also dashed areas, strokes and rotated elements.

mod common;

use common::{busy, fixture, xml};
use fs_whiteboard_core::{Element, Kind, Rect, StrokePoint, StyleField};

#[test]
fn the_check_rejects_duplicate_attributes_like_a_real_parser() {
    assert!(xml::check("<a x=\"1\" x=\"2\"/>").is_err());
    assert!(roxmltree::Document::parse("<a x=\"1\" x=\"2\"/>").is_err());
}

#[tokio::test]
async fn stroke_and_dashed_area_are_well_formed() {
    let f = fixture().await;
    let pts = vec![
        StrokePoint {
            x: 0.0,
            y: 0.0,
            pressure: None,
        },
        StrokePoint {
            x: 1.0,
            y: 1.0,
            pressure: None,
        },
    ];
    let s =
        f.ed.add(Element::new(
            f.page,
            Kind::Stroke { points: pts },
            Rect::new(0.0, 0.0, 9.0, 9.0),
        ))
        .await
        .unwrap()
        .id;
    let a =
        f.ed.add(Element::new(
            f.page,
            Kind::Area { text: "A".into() },
            Rect::new(0.0, 20.0, 9.0, 9.0),
        ))
        .await
        .unwrap()
        .id;
    f.ed.set_style(
        &[s, a],
        &[
            StyleField::Dashed(true),
            StyleField::Stroke("#000000".into()),
        ],
    )
    .await
    .unwrap();
    let svg = String::from_utf8(
        f.ed.export(Some(f.page), &[], "image/svg+xml")
            .await
            .unwrap()
            .bytes,
    )
    .unwrap();
    roxmltree::Document::parse(&svg).unwrap_or_else(|e| panic!("{e}\n{svg}"));
    xml::check(&svg).unwrap();
    assert_eq!(svg.matches("stroke-dasharray").count(), 2, "{svg}");
}

#[tokio::test]
async fn a_busy_board_is_well_formed_too() {
    let f = fixture().await;
    busy(&f).await;
    let svg = String::from_utf8(
        f.ed.export(Some(f.page), &[], "image/svg+xml")
            .await
            .unwrap()
            .bytes,
    )
    .unwrap();
    roxmltree::Document::parse(&svg).unwrap();
    xml::check(&svg).unwrap();
}
