//! SVG export: well-formed, escaped, rotated.

mod common;

use common::{el, fixture, xml};
use fs_whiteboard_core::{Element, Kind, Rect, ShapeKind, StrokePoint};
const SVG: &str = "image/svg+xml";

#[tokio::test]
async fn svg_is_well_formed_escaped_rotated_and_draws_strokes() {
    assert!(xml::check("<a><b></a>").is_err() && xml::check("<a x=\"<\"/>").is_err());
    assert!(xml::check("<a>&bogus;</a>").is_err() && xml::check("<a x=1/>").is_err());
    let f = fixture().await;
    let nasty = "<script>alert(\"x\") & 'y'</script>\u{0}\u{8}end";
    let mut t = el(&f, Kind::Text { text: nasty.into() }, 0.0);
    t.angle = std::f64::consts::FRAC_PI_2;
    f.ed.add(t).await.unwrap();
    let image_iri = "https://example.org/f?a=\"1\"&b=<2>";
    f.ed.add(el(
        &f,
        Kind::Image {
            iri: image_iri.into(),
        },
        60.0,
    ))
    .await
    .unwrap();
    let mut shape = el(
        &f,
        Kind::Shape {
            shape: ShapeKind::Ellipse,
        },
        120.0,
    );
    shape.label = "a < b & \"c\"".into();
    shape.style.stroke = "#00f\" onload=\"x".into();
    f.ed.add(shape).await.unwrap();
    let pts = vec![
        StrokePoint {
            x: 0.0,
            y: 0.0,
            pressure: None,
        },
        StrokePoint {
            x: 0.5,
            y: 1.0,
            pressure: None,
        },
        StrokePoint {
            x: 1.0,
            y: 0.0,
            pressure: Some(1.0),
        },
    ];
    let mut stroke = Element::new(
        f.page,
        Kind::Stroke { points: pts },
        Rect::new(10.0, 100.0, 80.0, 40.0),
    );
    stroke.style.stroke = "#123456".into();
    f.ed.add(stroke).await.unwrap();

    let svg = String::from_utf8(f.ed.export(Some(f.page), &[], SVG).await.unwrap().bytes).unwrap();
    let n = xml::check(&svg).unwrap_or_else(|e| panic!("{e}\n{svg}"));
    assert!(n > 8);
    assert!(!svg.contains("<script") && !svg.contains('\u{0}') && !svg.contains('\u{8}'));
    assert!(
        svg.contains("&lt;script&gt;alert(&quot;x&quot;) &amp; &apos;y&apos;&lt;/script&gt;end"),
        "{svg}"
    );
    assert!(
        svg.contains("href=\"https://example.org/f?a=&quot;1&quot;&amp;b=&lt;2&gt;\""),
        "{svg}"
    );
    assert!(svg.contains("a &lt; b &amp; &quot;c&quot;"));
    assert!(
        svg.contains("stroke=\"#00f&quot; onload=&quot;x\""),
        "attribute stays inside its quotes"
    );
    assert!(
        svg.contains("transform=\"rotate(90 20 15)\""),
        "a quarter turn around the middle of the frame"
    );
    assert!(
        svg.contains("d=\"M10 100 L50 140 L90 100\""),
        "points are fractions of the frame: {svg}"
    );
    assert!(svg.starts_with("<?xml"));
}

#[tokio::test]
async fn svg_leaves_out_deleted_and_hidden_elements() {
    let f = fixture().await;
    let keep =
        f.ed.add(el(
            &f,
            Kind::Text {
                text: "stay".into(),
            },
            0.0,
        ))
        .await
        .unwrap()
        .id;
    let gone =
        f.ed.add(el(
            &f,
            Kind::Text {
                text: "vanish".into(),
            },
            10.0,
        ))
        .await
        .unwrap()
        .id;
    let g = f.ed.group(&[gone]).await.unwrap();
    f.ed.delete(&[g]).await.unwrap();
    let svg = String::from_utf8(f.ed.export(Some(f.page), &[], SVG).await.unwrap().bytes).unwrap();
    assert!(svg.contains("stay") && !svg.contains("vanish"));
    assert!(svg.contains(&keep.to_string()) && !svg.contains(&gone.to_string()));
}
