//! Where the ends of a connector are.

mod common;

use common::fixture;
use fs_whiteboard_core::reading::End;
use fs_whiteboard_core::{Anchor, Connector, Element, Kind, Point, Rect};

#[tokio::test]
async fn a_connector_end_on_a_deleted_element_reports_its_last_frame() {
    let f = fixture().await;
    let (s, t) = (f.rect(0.0, 0.0).await, f.rect(100.0, 0.0).await);
    let c = Connector {
        source: Anchor {
            element: Some(s),
            point: Point::default(),
        },
        target: Anchor {
            element: None,
            point: Point { x: 7.0, y: 8.0 },
        },
        ..Connector::default()
    };
    let conn =
        f.ed.add(Element::new(
            f.page,
            Kind::Connector(c),
            Rect::new(0.0, 0.0, 100.0, 10.0),
        ))
        .await
        .unwrap()
        .id;
    f.ed.move_by(&[s], 3.0, 4.0).await.unwrap();
    let moved = Rect::new(3.0, 4.0, 10.0, 10.0);
    let (src, tgt) = f.repo.connector_ends(f.board, conn).await.unwrap();
    assert_eq!(
        src,
        End::Attached {
            element: s,
            frame: moved,
            deleted: false
        }
    );
    assert_eq!(tgt, End::Free(Point { x: 7.0, y: 8.0 }));
    f.ed.delete(&[s]).await.unwrap();
    let (src, _) = f.repo.connector_ends(f.board, conn).await.unwrap();
    assert_eq!(
        src,
        End::Attached {
            element: s,
            frame: moved,
            deleted: true
        }
    );
    assert!(
        f.repo.connector_ends(f.board, t).await.is_err(),
        "a rectangle has no ends"
    );
}
