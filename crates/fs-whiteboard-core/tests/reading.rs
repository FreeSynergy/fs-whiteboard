//! The reading rules: hiding, pages, cycles.

mod common;

use common::{add_rect, all, fixture, pair, replicate, setup};
use fs_objects::ObjectId;
use fs_whiteboard_core::{BoardEditor, Boards};

async fn ids(ed: &dyn BoardEditor, page: ObjectId) -> Vec<ObjectId> {
    all(ed, page).await.iter().map(|e| e.id).collect()
}

#[tokio::test]
async fn a_deleted_group_hides_its_members_nested_ones_too() {
    let f = fixture().await;
    let (a, b, c) = (
        f.rect(0.0, 0.0).await,
        f.rect(20.0, 0.0).await,
        f.rect(40.0, 0.0).await,
    );
    let inner = f.ed.group(&[a, b]).await.unwrap();
    let outer = f.ed.group(&[inner, c]).await.unwrap();
    assert_eq!(
        ids(&*f.ed, f.page).await.len(),
        5,
        "three rectangles and two groups"
    );
    f.ed.delete(&[outer]).await.unwrap();
    assert_eq!(ids(&*f.ed, f.page).await, [] as [fs_objects::ObjectId; 0]);
    assert!(
        f.ed.get_element(a).await.unwrap().is_some(),
        "still there as a marker"
    );
    assert!(f.ed.undo().await.unwrap());
    assert_eq!(
        ids(&*f.ed, f.page).await.len(),
        5,
        "undo brings them all back"
    );
}

#[tokio::test]
async fn a_member_that_got_its_parent_concurrently_is_hidden_too() {
    let p = pair().await;
    let (m, late) = (
        add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await,
        add_rect(&*p.a.ed, p.a.page, 50.0, 0.0).await,
    );
    let g = p.a.ed.group(&[m]).await.unwrap();
    p.sync().await;
    p.a.ed.delete(&[g]).await.unwrap();
    p.b.ed.set_parent(&[late], None, Some(g)).await.unwrap();
    p.sync().await;
    for f in [&p.a, &p.b] {
        assert!(
            ids(&*f.ed, f.page).await.is_empty(),
            "the late member is hidden by the deleted group"
        );
    }
}

#[tokio::test]
async fn a_removed_page_hides_its_elements() {
    let f = fixture().await;
    let (_, second) = f.ed.add_page("Second").await.unwrap();
    let on_second =
        f.ed.add(common::rect_element(second, 0.0, 0.0))
            .await
            .unwrap()
            .id;
    f.rect(0.0, 0.0).await;
    f.ed.remove_page(second).await.unwrap();
    assert_eq!(ids(&*f.ed, second).await, [] as [fs_objects::ObjectId; 0]);
    assert_eq!(ids(&*f.ed, f.page).await.len(), 1);
    assert!(f.ed.get_element(on_second).await.unwrap().is_some());
    assert!(f.ed.bounds(second).await.unwrap().is_none());
}

#[tokio::test]
async fn concurrent_removal_of_the_last_pages_keeps_the_highest_id() {
    let p = pair().await;
    let (_, second) = p.a.ed.add_page("Second").await.unwrap();
    p.sync().await;
    let first = p.a.page;
    let (e1, e2) = (
        add_rect(&*p.a.ed, first, 0.0, 0.0).await,
        p.a.ed
            .add(common::rect_element(second, 0.0, 0.0))
            .await
            .unwrap()
            .id,
    );
    p.sync().await;
    p.a.ed.remove_page(first).await.unwrap();
    p.b.ed.remove_page(second).await.unwrap();
    p.sync().await;
    let (keep, gone, kept_el) = if first > second {
        (first, second, e1)
    } else {
        (second, first, e2)
    };
    for f in [&p.a, &p.b] {
        let board = f.ed.get().await.unwrap();
        let live: Vec<_> = board
            .pages
            .iter()
            .filter(|x| !x.deleted)
            .map(|x| x.id)
            .collect();
        assert_eq!(live, vec![keep]);
        assert_eq!(ids(&*f.ed, keep).await, vec![kept_el]);
        assert_eq!(ids(&*f.ed, gone).await, [] as [fs_objects::ObjectId; 0]);
    }
    assert!(
        p.a.ed.remove_page(keep).await.is_err(),
        "and it is the last one now"
    );
}

#[tokio::test]
async fn a_concurrent_cycle_is_broken_the_same_way_whatever_the_order() {
    let p = pair().await;
    let (a, b, m) = (
        add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await,
        add_rect(&*p.a.ed, p.a.page, 30.0, 0.0).await,
        add_rect(&*p.a.ed, p.a.page, 60.0, 0.0).await,
    );
    let g1 = p.a.ed.group(&[a]).await.unwrap();
    let g2 = p.a.ed.group(&[b]).await.unwrap();
    p.sync().await;
    p.a.ed.set_parent(&[g1], None, Some(g2)).await.unwrap();
    p.b.ed.set_parent(&[g2], None, Some(g1)).await.unwrap();
    p.a.ed.set_parent(&[m], None, Some(g1)).await.unwrap(); // m sits in g1
    p.sync().await;
    let lo = g1.min(g2);
    // three replicas receive the two histories in opposite orders; the result must be the same
    let mut seen = Vec::new();
    for from_a_first in [true, false] {
        let (repo, store) = setup().await;
        let (first, second) = if from_a_first {
            (&p.a.store, &p.b.store)
        } else {
            (&p.b.store, &p.a.store)
        };
        replicate(first, &repo, p.a.board).await;
        replicate(second, &repo, p.a.board).await;
        let ed = repo.open(p.a.board).await.unwrap().unwrap();
        // the lowest id is the root: deleting it hides everything below, so the whole board
        ed.delete(&[lo]).await.unwrap();
        seen.push(ids(&*ed, p.a.page).await);
        drop(store);
    }
    assert_eq!(seen[0], seen[1]);
    assert!(
        seen[0].is_empty(),
        "deleting the root of the broken cycle hides all that is below"
    );
}
