//! Regression: a locked element is not moved, reparented or deleted through any path.

mod common;

use common::{all, fixture, frame_of};
use fs_whiteboard_core::Error;

#[tokio::test]
async fn a_locked_member_blocks_the_whole_edit_through_its_group() {
    let fx = fixture().await;
    let (a, b) = (fx.rect(0.0, 0.0).await, fx.rect(20.0, 0.0).await);
    let g = fx.ed.group(&[a, b]).await.unwrap();
    fx.ed.set_locked(&[b], true).await.unwrap();
    let (_, second) = fx.ed.add_page("Second").await.unwrap();

    let res = fx.ed.set_parent(&[g], Some(second), None).await;
    assert!(matches!(res, Err(Error::Locked(id)) if id == b), "{res:?}");
    assert_eq!(fx.ed.get_element(b).await.unwrap().unwrap().page, fx.page);
    assert_eq!(fx.ed.get_element(a).await.unwrap().unwrap().page, fx.page);

    let res = fx.ed.delete(&[g]).await;
    assert!(matches!(res, Err(Error::Locked(id)) if id == b), "{res:?}");
    assert_eq!(all(&*fx.ed, fx.page).await.len(), 3, "nothing was hidden");
    assert!(!fx.ed.get_element(g).await.unwrap().unwrap().deleted);

    let res = fx.ed.move_by(&[g], 5.0, 0.0).await;
    assert!(matches!(res, Err(Error::Locked(_))));
    assert_eq!(frame_of(&*fx.ed, a).await.x, 0.0);

    fx.ed.set_locked(&[b], false).await.unwrap();
    fx.ed.delete(&[g]).await.unwrap();
    assert_eq!(
        all(&*fx.ed, fx.page).await,
        [] as [fs_whiteboard_core::Element; 0]
    );
}

#[tokio::test]
async fn a_locked_anchor_of_reorder_is_allowed_because_it_is_not_edited() {
    let f = fixture().await;
    let ids = [
        f.rect(0.0, 0.0).await,
        f.rect(1.0, 0.0).await,
        f.rect(2.0, 0.0).await,
    ];
    f.ed.set_locked(&[ids[1]], true).await.unwrap();
    f.ed.reorder(&[ids[2]], Some(ids[1])).await.unwrap();
    let order: Vec<_> = common::order_of(&*f.ed, f.page).await;
    assert_eq!(order, vec![ids[0], ids[2], ids[1]]);
    assert!(
        f.ed.reorder(&[ids[1]], None).await.is_err(),
        "but a locked element itself stays"
    );
}
