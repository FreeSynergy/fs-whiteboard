//! Refusals at every entry: numbers, sizes, limits.

mod common;

use common::{fixture, frame_of, rect_element};
use fs_whiteboard_core::validate::{MAX_STROKE_POINTS, MAX_TEXT_BYTES};
use fs_whiteboard_core::{
    Boards, ConnectorEnd, Element, Error, Kind, Point, Rect, StrokePoint, StyleField,
};

fn invalid<T: std::fmt::Debug>(r: Result<T, Error>) {
    match r {
        Err(Error::Invalid(_)) => {}
        other => panic!("{other:?}"),
    }
}

fn stroke(n: usize) -> Kind {
    Kind::Stroke {
        points: vec![
            StrokePoint {
                x: 0.5,
                y: 0.5,
                pressure: None
            };
            n
        ],
    }
}

#[tokio::test]
async fn numbers_must_be_finite_and_sizes_not_negative() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut e = rect_element(f.page, 0.0, 0.0);
        e.frame.x = bad;
        invalid(f.ed.add(e).await);
        let mut e = rect_element(f.page, 0.0, 0.0);
        e.angle = bad;
        invalid(f.ed.add(e).await);
        let mut e = rect_element(f.page, 0.0, 0.0);
        e.style.line_width = bad;
        invalid(f.ed.add(e).await);
        invalid(f.ed.move_by(&[id], bad, 0.0).await);
        invalid(f.ed.move_by(&[id], 0.0, bad).await);
        invalid(f.ed.rotate(id, bad).await);
        invalid(f.ed.set_frame(id, Rect::new(0.0, bad, 1.0, 1.0)).await);
        invalid(f.ed.set_frame(id, Rect::new(0.0, 0.0, bad, 1.0)).await);
        invalid(f.ed.copy(&[id], bad, 0.0).await);
        invalid(f.ed.set_style(&[id], &[StyleField::LineWidth(bad)]).await);
        invalid(
            f.ed.set_end(id, ConnectorEnd::Source, None, Point { x: bad, y: 0.0 })
                .await,
        );
        invalid(
            f.ed.elements(f.page, Some(Rect::new(bad, 0.0, 1.0, 1.0)), None)
                .await,
        );
    }
    invalid(f.ed.set_frame(id, Rect::new(0.0, 0.0, -1.0, 1.0)).await);
    invalid(f.ed.set_frame(id, Rect::new(0.0, 0.0, 1.0, -0.5)).await);
    let mut e = rect_element(f.page, 0.0, 0.0);
    e.frame.height = -1.0;
    invalid(f.ed.add(e).await);
    invalid(
        f.ed.elements(f.page, Some(Rect::new(0.0, 0.0, -1.0, 1.0)), None)
            .await,
    );
    assert_eq!(
        frame_of(&*f.ed, id).await,
        Rect::new(0.0, 0.0, 10.0, 10.0),
        "nothing was changed"
    );
}

#[tokio::test]
async fn opacity_is_between_zero_and_one() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    for bad in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        invalid(f.ed.set_style(&[id], &[StyleField::Opacity(bad)]).await);
        let mut e = rect_element(f.page, 0.0, 0.0);
        e.style.opacity = bad;
        invalid(f.ed.add(e).await);
    }
    invalid(
        f.ed.set_style(&[id], &[StyleField::FontSize(f32::NAN)])
            .await,
    );
    for ok in [0.0, 0.5, 1.0] {
        f.ed.set_style(&[id], &[StyleField::Opacity(ok)])
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn limits_on_strokes_text_and_imports() {
    let f = fixture().await;
    let frame = Rect::new(0.0, 0.0, 10.0, 10.0);
    let too_long =
        f.ed.add(Element::new(f.page, stroke(MAX_STROKE_POINTS + 1), frame))
            .await;
    assert!(matches!(too_long, Err(Error::TooLarge(_))), "{too_long:?}");
    let e =
        f.ed.add(Element::new(f.page, stroke(MAX_STROKE_POINTS), frame))
            .await
            .unwrap();
    assert!(matches!(e.kind, Kind::Stroke { points } if points.len() == MAX_STROKE_POINTS));
    let mut bad = StrokePoint {
        x: f64::NAN,
        y: 0.0,
        pressure: None,
    };
    invalid(
        f.ed.add(Element::new(
            f.page,
            Kind::Stroke { points: vec![bad] },
            frame,
        ))
        .await,
    );
    bad.x = 0.0;
    bad.pressure = Some(2.0);
    invalid(
        f.ed.add(Element::new(
            f.page,
            Kind::Stroke { points: vec![bad] },
            frame,
        ))
        .await,
    );

    let big = "x".repeat(MAX_TEXT_BYTES + 1);
    let r =
        f.ed.add(Element::new(
            f.page,
            Kind::Text { text: big.clone() },
            frame,
        ))
        .await;
    assert!(matches!(r, Err(Error::TooLarge(_))));
    let t =
        f.ed.add(Element::new(
            f.page,
            Kind::Text {
                text: "x".repeat(MAX_TEXT_BYTES),
            },
            frame,
        ))
        .await
        .unwrap();
    assert!(matches!(
        f.ed.set_text(t.id, &big).await,
        Err(Error::TooLarge(_))
    ));

    let data = vec![b' '; 32 * 1024 * 1024 + 1];
    let r = f
        .repo
        .import("application/vnd.freesynergy.whiteboard+json", &data)
        .await;
    assert!(matches!(r, Err(Error::TooLarge(_))), "{r:?}");
}
