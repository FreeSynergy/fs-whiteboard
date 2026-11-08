//! Named concurrent cases through two real in-memory stores and `apply_remote`.

mod common;

use chrono::{TimeZone, Utc};
use common::{add_rect, frame_of, pair, raw};
use fs_whiteboard_core::{BoardEditor, Boards, StyleField};

async fn fill_of(ed: &dyn BoardEditor, id: fs_objects::ObjectId) -> String {
    ed.get_element(id).await.unwrap().unwrap().style.fill
}

#[tokio::test]
async fn concurrent_move_and_colour_both_survive() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    p.a.ed.move_by(&[id], 5.0, 7.0).await.unwrap();
    p.b.ed
        .set_style(&[id], &[StyleField::Fill("#ff0000".into())])
        .await
        .unwrap();
    p.sync().await;
    for f in [&p.a, &p.b] {
        assert_eq!(common::frame_of(&*f.ed, id).await.x, 5.0);
        assert_eq!(fill_of(&*f.ed, id).await, "#ff0000");
    }
    assert_eq!(
        raw(&p.a.store, id).await,
        raw(&p.b.store, id).await,
        "byte-identical"
    );
}

#[tokio::test]
async fn concurrent_delete_and_move_keep_the_move_for_undo() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    p.a.ed.delete(&[id]).await.unwrap();
    p.b.ed.move_by(&[id], 20.0, 0.0).await.unwrap();
    p.sync().await;
    for f in [&p.a, &p.b] {
        let e = f.ed.get_element(id).await.unwrap().unwrap();
        assert!(e.deleted, "deleted wins");
        assert_eq!(e.frame.x, 20.0, "the move is kept");
        assert_eq!(
            common::all(&*f.ed, f.page).await,
            [] as [fs_whiteboard_core::Element; 0]
        );
    }
    assert_eq!(raw(&p.a.store, id).await, raw(&p.b.store, id).await);
    // the person who deleted takes it back: the element is there, where the other put it
    assert!(p.a.ed.undo().await.unwrap());
    assert_eq!(frame_of(&*p.a.ed, id).await.x, 20.0);
    assert_eq!(common::all(&*p.a.ed, p.a.page).await.len(), 1);
}

#[tokio::test]
async fn clocks_decide_not_the_times_in_the_envelope() {
    let p = pair().await;
    let id = add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    // both set the fill from the same state: equal counters, the actor text decides ("replica-b" > "https")
    p.a.ed
        .set_style(&[id], &[StyleField::Fill("#aaaaaa".into())])
        .await
        .unwrap();
    p.b.ed
        .set_style(&[id], &[StyleField::Fill("#bbbbbb".into())])
        .await
        .unwrap();
    let (mut from_a, mut from_b) = (
        p.a.store.get(id).await.unwrap().unwrap(),
        p.b.store.get(id).await.unwrap().unwrap(),
    );
    // a envelope claims the far future, b's the distant past: neither is read
    from_a.envelope.updated = Utc.with_ymd_and_hms(2999, 1, 1, 0, 0, 0).unwrap();
    from_b.envelope.updated = Utc.with_ymd_and_hms(1971, 1, 1, 0, 0, 0).unwrap();
    p.a.repo.apply_remote(from_b.clone()).await.unwrap();
    p.b.repo.apply_remote(from_a.clone()).await.unwrap();
    assert_eq!(fill_of(&*p.a.ed, id).await, "#bbbbbb");
    assert_eq!(fill_of(&*p.b.ed, id).await, "#bbbbbb");
    // and the other way round: the larger clock wins although its envelope is older
    let q = pair().await;
    let id = add_rect(&*q.a.ed, q.a.page, 0.0, 0.0).await;
    q.sync().await;
    q.b.ed
        .set_style(&[id], &[StyleField::Fill("#111111".into())])
        .await
        .unwrap();
    q.b.ed
        .set_style(&[id], &[StyleField::Fill("#222222".into())])
        .await
        .unwrap();
    q.a.ed
        .set_style(&[id], &[StyleField::Fill("#333333".into())])
        .await
        .unwrap();
    let mut newer_clock_older_time = q.b.store.get(id).await.unwrap().unwrap();
    newer_clock_older_time.envelope.updated = Utc.with_ymd_and_hms(1971, 1, 1, 0, 0, 0).unwrap();
    q.a.repo.apply_remote(newer_clock_older_time).await.unwrap();
    assert_eq!(fill_of(&*q.a.ed, id).await, "#222222");
}

#[tokio::test]
async fn applying_twice_or_in_another_order_changes_nothing() {
    let p = pair().await;
    let ids = [
        add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await,
        add_rect(&*p.a.ed, p.a.page, 9.0, 9.0).await,
    ];
    p.sync().await;
    p.a.ed.move_by(&ids, 1.0, 1.0).await.unwrap();
    p.b.ed.rotate(ids[0], 1.0).await.unwrap();
    p.sync().await;
    let before: Vec<String> = vec![raw(&p.a.store, ids[0]).await, raw(&p.a.store, ids[1]).await];
    p.sync().await;
    p.sync().await;
    assert_eq!(
        before,
        vec![raw(&p.a.store, ids[0]).await, raw(&p.a.store, ids[1]).await]
    );
    assert_eq!(before[0], raw(&p.b.store, ids[0]).await);
}

#[tokio::test]
async fn pages_and_names_merge_per_unit() {
    let p = pair().await;
    let (_, extra) = p.a.ed.add_page("Second").await.unwrap();
    p.sync().await;
    p.a.repo.rename(p.a.board, "Renamed by a").await.unwrap();
    p.b.ed.move_page(extra, Some(p.b.page)).await.unwrap();
    p.sync().await;
    for f in [&p.a, &p.b] {
        let b = f.ed.get().await.unwrap();
        assert_eq!(b.name, "Renamed by a");
        assert_eq!(b.pages[0].id, extra, "moved before the first page");
    }
    assert_eq!(
        raw(&p.a.store, p.a.board).await,
        raw(&p.b.store, p.a.board).await
    );
}
