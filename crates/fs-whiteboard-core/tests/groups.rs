//! Groups: moving, copying, ungrouping.

mod common;

use common::{all, fixture, frame_of};
use fs_whiteboard_core::{Anchor, Connector, Element, Kind, Point, Rect};

fn connector(
    page: fs_objects::ObjectId,
    from: fs_objects::ObjectId,
    to: fs_objects::ObjectId,
) -> Element {
    let k = Connector {
        source: Anchor {
            element: Some(from),
            point: Point::default(),
        },
        target: Anchor {
            element: Some(to),
            point: Point::default(),
        },
        label: "link".into(),
        arrow_end: true,
    };
    Element::new(page, Kind::Connector(k), Rect::new(0.0, 0.0, 10.0, 10.0))
}

#[tokio::test]
async fn moving_a_group_moves_each_member_once() {
    let fx = fixture().await;
    let (a, b, c) = (
        fx.rect(0.0, 0.0).await,
        fx.rect(20.0, 0.0).await,
        fx.rect(40.0, 0.0).await,
    );
    let inner = fx.ed.group(&[a, b]).await.unwrap();
    let outer = fx.ed.group(&[inner, c]).await.unwrap();
    // the group, its group inside, and members of both are all named, some twice
    fx.ed
        .move_by(&[outer, a, inner, c, a], 10.0, 5.0)
        .await
        .unwrap();
    for (id, x) in [(a, 10.0), (b, 30.0), (c, 50.0)] {
        let rc = frame_of(&*fx.ed, id).await;
        assert_eq!((rc.x, rc.y), (x, 5.0), "{id} moved exactly once");
    }
    let g = fx.ed.get_element(outer).await.unwrap().unwrap();
    assert_eq!(
        (g.frame.x, g.frame.right()),
        (10.0, 60.0),
        "the group's frame is the union of its members"
    );
    assert!(
        fx.ed.set_frame(outer, Rect::default()).await.is_err(),
        "a group has no frame of its own"
    );
}

#[tokio::test]
async fn copy_retargets_connectors_between_copies_and_leaves_others_alone() {
    let f = fixture().await;
    let (s, t, u) = (
        f.rect(0.0, 0.0).await,
        f.rect(50.0, 0.0).await,
        f.rect(100.0, 0.0).await,
    );
    let inside = f.ed.add(connector(f.page, s, t)).await.unwrap().id;
    let outside = f.ed.add(connector(f.page, s, u)).await.unwrap().id;
    let copies =
        f.ed.copy(&[s, t, inside, outside], 0.0, 200.0)
            .await
            .unwrap();
    assert_eq!(copies.len(), 4);
    let ed = &*f.ed;
    let get = |id| async move { ed.get_element(id).await.unwrap().unwrap() };
    let Kind::Connector(inner) = get(copies[2]).await.kind else {
        panic!()
    };
    assert_eq!(
        (inner.source.element, inner.target.element),
        (Some(copies[0]), Some(copies[1]))
    );
    let Kind::Connector(out) = get(copies[3]).await.kind else {
        panic!()
    };
    assert_eq!(
        (out.source.element, out.target.element),
        (Some(copies[0]), Some(u)),
        "u was not copied"
    );
    let Kind::Connector(orig) = get(inside).await.kind else {
        panic!()
    };
    assert_eq!(
        orig.source.element,
        Some(s),
        "the originals stay as they were"
    );
    assert_eq!(frame_of(&*f.ed, copies[0]).await.y, 200.0);
    assert_eq!(all(&*f.ed, f.page).await.len(), 9);
}

#[tokio::test]
async fn copying_a_group_copies_its_members_into_the_copy() {
    let f = fixture().await;
    let (a, b) = (f.rect(0.0, 0.0).await, f.rect(20.0, 0.0).await);
    let g = f.ed.group(&[a, b]).await.unwrap();
    let copy = f.ed.copy(&[g], 100.0, 0.0).await.unwrap()[0];
    let members: Vec<_> = all(&*f.ed, f.page)
        .await
        .into_iter()
        .filter(|e| e.parent == Some(copy))
        .collect();
    assert_eq!(members.len(), 2);
    assert!(
        members
            .iter()
            .all(|m| m.id != a && m.id != b && m.frame.x >= 100.0)
    );
    assert_eq!(f.ed.get_element(a).await.unwrap().unwrap().parent, Some(g));
}

#[tokio::test]
async fn ungroup_hands_the_members_to_the_groups_parent() {
    let f = fixture().await;
    let (a, b, c) = (
        f.rect(0.0, 0.0).await,
        f.rect(20.0, 0.0).await,
        f.rect(40.0, 0.0).await,
    );
    let inner = f.ed.group(&[a, b]).await.unwrap();
    let outer = f.ed.group(&[inner, c]).await.unwrap();
    let freed = f.ed.ungroup(inner).await.unwrap();
    assert_eq!(freed.len(), 2);
    for id in [a, b] {
        assert_eq!(
            f.ed.get_element(id).await.unwrap().unwrap().parent,
            Some(outer)
        );
    }
    assert!(f.ed.get_element(inner).await.unwrap().unwrap().deleted);
    assert!(f.ed.ungroup(a).await.is_err(), "a rectangle is no group");
}
