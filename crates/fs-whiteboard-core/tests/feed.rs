//! The change feed of one board, and deleting a board.

mod common;

use std::time::Duration;

use common::{add_rect, fixture};
use fs_store::{ObjectId, Query};
use fs_whiteboard_core::{Boards, Change, Subscription, WhiteboardFeed};
use tokio::time::timeout;

async fn next(s: &mut Subscription) -> Option<Change> {
    timeout(Duration::from_millis(300), s.next())
        .await
        .ok()
        .flatten()
}

#[tokio::test]
async fn only_the_changes_of_that_board_arrive() {
    let f = fixture().await;
    let other = f.repo.create("Other").await.unwrap();
    let theirs = f.repo.open(other.id).await.unwrap().unwrap();
    let mut sub = f.repo.subscribe(f.board).await.unwrap();

    add_rect(&*theirs, other.pages[0].id, 0.0, 0.0).await;
    theirs.add_page("x").await.unwrap();
    f.repo.rename(other.id, "Renamed other").await.unwrap();
    let mine = add_rect(&*f.ed, f.page, 0.0, 0.0).await;
    f.ed.move_by(&[mine], 1.0, 1.0).await.unwrap();
    f.ed.delete(&[mine]).await.unwrap();
    f.repo.rename(f.board, "Renamed mine").await.unwrap();

    let Some(Change::Created(e)) = next(&mut sub).await else {
        panic!("created")
    };
    assert_eq!(e.id, mine);
    assert!(matches!(next(&mut sub).await, Some(Change::Updated(e)) if e.id == mine));
    assert!(matches!(next(&mut sub).await, Some(Change::Updated(e)) if e.id == mine));
    assert!(matches!(next(&mut sub).await, Some(Change::Updated(e)) if e.id == f.board));
    assert!(
        next(&mut sub).await.is_none(),
        "nothing of the other board came through"
    );
}

#[tokio::test]
async fn subscribing_to_a_board_that_is_not_there_fails() {
    let f = fixture().await;
    assert!(f.repo.subscribe(ObjectId::new()).await.is_err());
}

#[tokio::test]
async fn deleting_a_board_leaves_nothing_in_the_store() {
    let f = fixture().await;
    let keep = f.repo.create("Keep").await.unwrap();
    let keep_ed = f.repo.open(keep.id).await.unwrap().unwrap();
    let kept = add_rect(&*keep_ed, keep.pages[0].id, 0.0, 0.0).await;
    let ids = [f.rect(0.0, 0.0).await, f.rect(20.0, 0.0).await];
    f.ed.group(&ids).await.unwrap();
    f.ed.delete(&[ids[0]]).await.unwrap();
    let mut sub = f.repo.subscribe(f.board).await.unwrap();

    f.repo.delete(f.board).await.unwrap();

    let mine = Query::new().with_index("board", f.board.to_string());
    assert!(
        f.store.query(&mine).await.unwrap().is_empty(),
        "no element left, deleted ones included"
    );
    let marker = f.store.get(f.board).await.unwrap().unwrap();
    assert_eq!(
        marker.envelope.object_type, "WhiteboardTombstone",
        "only the marker is left"
    );
    for id in ids {
        assert!(f.store.get(id).await.unwrap().is_none());
    }
    let left: Vec<ObjectId> = f
        .store
        .query(&Query::new())
        .await
        .unwrap()
        .iter()
        .map(|o| o.envelope.id)
        .collect();
    assert_eq!(
        left.len(),
        3,
        "the other board, its element, the marker: {left:?}"
    );
    assert!(left.contains(&keep.id) && left.contains(&kept) && left.contains(&f.board));
    assert!(f.repo.open(f.board).await.unwrap().is_none());

    let mut gone = 0;
    while let Some(Change::Deleted(_)) = next(&mut sub).await {
        gone += 1;
    }
    assert_eq!(gone, 4, "three elements and the board");
}

#[tokio::test]
async fn boards_list_in_pages_and_rename() {
    let (repo, _) = common::setup().await;
    let repo = repo.with_page_size(2);
    let mut made = Vec::new();
    for i in 0..5 {
        made.push(repo.create(&format!("Board {i}")).await.unwrap().id);
    }
    let (mut seen, mut cursor) = (Vec::new(), None);
    loop {
        let p = repo.list(cursor.as_ref()).await.unwrap();
        seen.extend(p.items.iter().map(|b| b.id));
        match p.next {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(seen, made);
    let b = repo.rename(made[0], "New name").await.unwrap();
    assert_eq!(b.name, "New name");
    assert!(
        b.as2.contains("\"Collection\"") && b.iri.starts_with("https://example.org/whiteboard/")
    );
    assert!(repo.open(ObjectId::new()).await.unwrap().is_none());
}
