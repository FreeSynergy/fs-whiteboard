//! Refusals: media types, last page, text, locks, references.

mod common;

use common::{fixture, frame_of, rect_element};
use fs_store::ObjectId;
use fs_whiteboard_core::{
    Align, Boards, ConnectorEnd, Element, Error, Kind, Point, Rect, StrokePoint, StyleField,
};

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

fn invalid<T: std::fmt::Debug>(r: Result<T, Error>) {
    assert!(matches!(r, Err(Error::Invalid(_))), "{r:?}");
}

#[tokio::test]
async fn unknown_media_types_are_unsupported() {
    let f = fixture().await;
    for mt in [
        "image/png",
        "application/pdf",
        "text/plain",
        "application/x-drawio",
        "",
    ] {
        assert!(
            matches!(f.repo.import(mt, b"{}").await, Err(Error::Unsupported(_))),
            "import {mt}"
        );
        let r = f.ed.export(Some(f.page), &[], mt).await;
        assert!(matches!(r, Err(Error::Unsupported(_))), "export {mt}");
    }
    assert!(
        matches!(
            f.repo.import("image/svg+xml", b"<svg/>").await,
            Err(Error::Unsupported(_))
        ),
        "SVG is export only"
    );
}

#[tokio::test]
async fn the_last_page_stays_and_text_needs_a_kind_with_text() {
    let f = fixture().await;
    assert!(matches!(
        f.ed.remove_page(f.page).await,
        Err(Error::LastPage)
    ));
    let (_, second) = f.ed.add_page("Second").await.unwrap();
    f.ed.remove_page(second).await.unwrap();
    assert!(
        matches!(f.ed.remove_page(f.page).await, Err(Error::LastPage)),
        "a removed page does not count"
    );

    let frame = Rect::new(0.0, 0.0, 10.0, 10.0);
    let s =
        f.ed.add(Element::new(f.page, stroke(2), frame))
            .await
            .unwrap()
            .id;
    let i =
        f.ed.add(Element::new(
            f.page,
            Kind::Image {
                iri: "https://example.org/files/1".into(),
            },
            frame,
        ))
        .await
        .unwrap()
        .id;
    let g = f.ed.group(&[s]).await.unwrap();
    for id in [s, i, g] {
        invalid(f.ed.set_text(id, "no").await);
    }
    let note =
        f.ed.add(Element::new(f.page, Kind::Note { text: "a".into() }, frame))
            .await
            .unwrap()
            .id;
    f.ed.set_text(note, "b").await.unwrap();
    invalid(f.repo.create("   ").await);
}

#[tokio::test]
async fn a_locked_element_refuses_every_edit_but_unlocking() {
    let f = fixture().await;
    let (id, other) = (f.rect(0.0, 0.0).await, f.rect(30.0, 0.0).await);
    f.ed.set_locked(&[id], true).await.unwrap();
    let locked =
        |r: Result<(), Error>| assert!(matches!(r, Err(Error::Locked(l)) if l == id), "{r:?}");
    locked(f.ed.move_by(&[id], 1.0, 1.0).await);
    locked(f.ed.set_frame(id, Rect::new(0.0, 0.0, 1.0, 1.0)).await);
    locked(f.ed.rotate(id, 1.0).await);
    locked(
        f.ed.set_style(&[id], &[StyleField::Fill("#ff0000".into())])
            .await,
    );
    locked(f.ed.set_text(id, "x").await);
    locked(f.ed.delete(&[id]).await);
    locked(f.ed.reorder(&[id], None).await);
    locked(f.ed.align(&[id, other], Align::Right).await);
    locked(f.ed.set_parent(&[id], None, None).await);
    locked(f.ed.group(&[id, other]).await.map(|_| ()));
    assert_eq!(frame_of(&*f.ed, id).await.x, 0.0);
    assert_eq!(
        frame_of(&*f.ed, other).await.x,
        30.0,
        "the refused align moved nobody"
    );
    f.ed.set_locked(&[id], false).await.unwrap();
    f.ed.move_by(&[id], 1.0, 0.0).await.unwrap();
    let ghost = ObjectId::new();
    assert!(matches!(
        f.ed.move_by(&[ghost], 1.0, 0.0).await,
        Err(Error::Foreign(_))
    ));
}

#[tokio::test]
async fn an_element_needs_a_live_page_a_container_parent_and_known_ends() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let mut e = rect_element(ObjectId::new(), 0.0, 0.0);
    assert!(matches!(
        f.ed.add(e.clone()).await,
        Err(Error::NotFound { .. })
    ));
    e.page = f.page;
    e.parent = Some(id);
    invalid(f.ed.add(e.clone()).await);
    e.parent = Some(ObjectId::new());
    assert!(matches!(f.ed.add(e).await, Err(Error::Foreign(_))));
    invalid(
        f.ed.set_end(id, ConnectorEnd::Target, None, Point::default())
            .await,
    );
}
