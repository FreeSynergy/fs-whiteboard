//! The editor's align and reorder.

mod common;

use common::{all, fixture, frame_of};
use fs_whiteboard_core::Align;

#[tokio::test]
async fn editor_align_moves_the_elements_and_distributes() {
    let f = fixture().await;
    let ids = [
        f.rect(0.0, 0.0).await,
        f.rect(15.0, 40.0).await,
        f.rect(100.0, 80.0).await,
    ];
    f.ed.align(&ids, Align::DistributeX).await.unwrap();
    let xs: Vec<f64> = vec![
        frame_of(&*f.ed, ids[0]).await.x,
        frame_of(&*f.ed, ids[1]).await.x,
        frame_of(&*f.ed, ids[2]).await.x,
    ];
    assert_eq!(xs, vec![0.0, 50.0, 100.0]);
    f.ed.align(&ids, Align::CenterY).await.unwrap();
    for i in ids {
        assert_eq!(frame_of(&*f.ed, i).await.center().y, 45.0);
    }
}

#[tokio::test]
async fn new_elements_go_to_the_front_and_reorder_puts_them_directly_behind() {
    let f = fixture().await;
    let ids = [
        f.rect(0.0, 0.0).await,
        f.rect(1.0, 0.0).await,
        f.rect(2.0, 0.0).await,
        f.rect(3.0, 0.0).await,
    ];
    assert_eq!(
        common::order_of(&*f.ed, f.page).await,
        ids.to_vec(),
        "later is in front"
    );
    // 3 and 1 go directly behind 2, keeping their order (1 before 3)
    f.ed.reorder(&[ids[3], ids[1]], Some(ids[2])).await.unwrap();
    assert_eq!(
        common::order_of(&*f.ed, f.page).await,
        vec![ids[0], ids[1], ids[3], ids[2]]
    );
    f.ed.reorder(&[ids[0]], None).await.unwrap();
    assert_eq!(
        common::order_of(&*f.ed, f.page).await,
        vec![ids[1], ids[3], ids[2], ids[0]]
    );
    f.ed.reorder(&[ids[0], ids[2]], Some(ids[1])).await.unwrap();
    assert_eq!(
        common::order_of(&*f.ed, f.page).await,
        vec![ids[2], ids[0], ids[1], ids[3]]
    );
    assert!(f.ed.reorder(&[ids[1]], Some(ids[1])).await.is_err());
}

#[tokio::test]
async fn reorder_behind_an_element_with_the_same_key_still_lands_there() {
    let p = common::pair().await;
    let x = common::add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    let y = common::add_rect(&*p.b.ed, p.b.page, 1.0, 0.0).await; // the same key as x, made apart
    let z = common::add_rect(&*p.a.ed, p.a.page, 2.0, 0.0).await;
    p.sync().await;
    let (lo, hi) = if x < y { (x, y) } else { (y, x) };
    let moved = common::add_rect(&*p.a.ed, p.a.page, 3.0, 0.0).await;
    p.a.ed.reorder(&[moved], Some(hi)).await.unwrap();
    let list: Vec<_> = all(&*p.a.ed, p.a.page).await.iter().map(|e| e.id).collect();
    let (pm, ph) = (
        list.iter().position(|i| *i == moved).unwrap(),
        list.iter().position(|i| *i == hi).unwrap(),
    );
    assert_eq!(pm + 1, ph, "directly behind: {list:?} lo={lo} z={z}");
    assert!(list.iter().position(|i| *i == lo).unwrap() < pm);
}
