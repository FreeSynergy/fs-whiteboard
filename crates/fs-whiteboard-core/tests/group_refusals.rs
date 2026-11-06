//! Groups: cycles, locks, ids of another board, pages.

mod common;

use common::{all, fixture, frame_of, setup};
use fs_whiteboard_core::{Boards, Error};

#[tokio::test]
async fn a_cycle_is_refused() {
    let f = fixture().await;
    let (a, b) = (f.rect(0.0, 0.0).await, f.rect(20.0, 0.0).await);
    let inner = f.ed.group(&[a]).await.unwrap();
    let outer = f.ed.group(&[inner, b]).await.unwrap();
    assert!(matches!(
        f.ed.set_parent(&[outer], None, Some(inner)).await,
        Err(Error::Cycle { .. })
    ));
    assert!(matches!(
        f.ed.set_parent(&[outer], None, Some(outer)).await,
        Err(Error::Cycle { .. })
    ));
    // two moves that are fine one by one but not together
    let (g1, g2) = (
        f.ed.group(&[b]).await.unwrap(),
        f.ed.group(&[a]).await.unwrap(),
    );
    let both = f.ed.set_parent(&[g1, g2], None, Some(g1)).await;
    assert!(matches!(both, Err(Error::Cycle { .. })));
    assert!(
        f.ed.set_parent(&[a], None, Some(b)).await.is_err(),
        "a rectangle holds nothing"
    );
    assert_eq!(
        f.ed.get_element(inner).await.unwrap().unwrap().parent,
        Some(outer),
        "nothing changed"
    );
}

#[tokio::test]
async fn locked_and_foreign_ids_are_refused_and_change_nothing() {
    let f = fixture().await;
    let (a, b) = (f.rect(0.0, 0.0).await, f.rect(20.0, 0.0).await);
    f.ed.set_locked(&[b], true).await.unwrap();
    assert!(matches!(f.ed.group(&[a, b]).await, Err(Error::Locked(id)) if id == b));
    let g = f.ed.group(&[a]).await.unwrap();
    assert!(matches!(
        f.ed.set_parent(&[b], None, None).await,
        Err(Error::Locked(_))
    ));
    f.ed.set_locked(&[b], false).await.unwrap();
    f.ed.set_parent(&[b], None, Some(g)).await.unwrap();
    f.ed.set_locked(&[b], true).await.unwrap();
    assert!(matches!(
        f.ed.move_by(&[g], 10.0, 0.0).await,
        Err(Error::Locked(_))
    ));
    assert_eq!(
        frame_of(&*f.ed, a).await.x,
        0.0,
        "the whole move was refused"
    );

    // an element of another board
    let other = f.repo.create("Other").await.unwrap();
    let theirs = f.repo.open(other.id).await.unwrap().unwrap();
    let alien = common::add_rect(&*theirs, other.pages[0].id, 0.0, 0.0).await;
    assert!(matches!(f.ed.group(&[a, alien]).await, Err(Error::Foreign(id)) if id == alien));
    assert!(matches!(
        f.ed.move_by(&[alien], 1.0, 1.0).await,
        Err(Error::Foreign(_))
    ));
    assert!(matches!(
        f.ed.set_parent(&[a], None, Some(alien)).await,
        Err(Error::Foreign(_))
    ));
    assert!(matches!(
        f.ed.copy(&[alien], 0.0, 0.0).await,
        Err(Error::Foreign(_))
    ));
    assert!(matches!(
        f.ed.elements(other.pages[0].id, None, None).await,
        Err(Error::Foreign(_))
    ));
    let _ = setup;
}

#[tokio::test]
async fn set_parent_to_another_page_takes_the_members_along() {
    let f = fixture().await;
    let (_, second) = f.ed.add_page("Second").await.unwrap();
    let (a, b) = (f.rect(0.0, 0.0).await, f.rect(20.0, 0.0).await);
    let g = f.ed.group(&[a]).await.unwrap();
    f.ed.set_parent(&[g], Some(second), None).await.unwrap();
    assert_eq!(f.ed.get_element(a).await.unwrap().unwrap().page, second);
    assert_eq!(all(&*f.ed, second).await.len(), 2);
    assert_eq!(all(&*f.ed, f.page).await.len(), 1);
    assert!(f.ed.set_parent(&[b], Some(second), Some(g)).await.is_ok());
}
