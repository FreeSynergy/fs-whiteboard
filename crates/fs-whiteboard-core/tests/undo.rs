//! Undo and redo belong to one editor; other people's work stays.

mod common;

use common::{all, fixture, frame_of};
use fs_whiteboard_core::{Align, BoardEditor, StyleField};

async fn fill(ed: &dyn BoardEditor, id: fs_store::ObjectId) -> String {
    ed.get_element(id).await.unwrap().unwrap().style.fill
}

fn red() -> [StyleField; 1] {
    [StyleField::Fill("#ff0000".into())]
}

#[tokio::test]
async fn another_persons_colour_stays_when_i_undo() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let (alice, bob) = (f.editor_of("alice").await, f.editor_of("bob").await);
    alice.set_style(&[id], &red()).await.unwrap();
    alice
        .set_style(&[id], &[StyleField::Fill("#0000ff".into())])
        .await
        .unwrap();
    bob.set_style(&[id], &[StyleField::Fill("#00ff00".into())])
        .await
        .unwrap();
    assert!(
        !alice.undo().await.unwrap(),
        "nothing was left to do: the colour is Bob's now"
    );
    assert_eq!(fill(&*alice, id).await, "#00ff00");
    assert!(
        !alice.undo().await.unwrap(),
        "the older edit of that colour is overtaken too"
    );
    assert_eq!(fill(&*alice, id).await, "#00ff00");
    assert!(
        !alice.undo().await.unwrap(),
        "and then the history is empty"
    );
}

#[tokio::test]
async fn without_interference_undo_and_redo_walk_back_and_forth() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    f.ed.set_style(&[id], &red()).await.unwrap();
    f.ed.set_style(&[id], &[StyleField::Fill("#0000ff".into())])
        .await
        .unwrap();
    assert!(f.ed.undo().await.unwrap());
    assert_eq!(fill(&*f.ed, id).await, "#ff0000");
    assert!(f.ed.undo().await.unwrap());
    assert_eq!(fill(&*f.ed, id).await, "");
    assert!(f.ed.redo().await.unwrap() && f.ed.redo().await.unwrap());
    assert_eq!(fill(&*f.ed, id).await, "#0000ff");
    assert!(!f.ed.redo().await.unwrap());
    f.ed.undo().await.unwrap();
    f.ed.set_style(&[id], &[StyleField::Fill("#123456".into())])
        .await
        .unwrap();
    assert!(!f.ed.redo().await.unwrap(), "a new edit ends the redo");
}

#[tokio::test]
async fn my_undo_takes_back_only_what_is_still_mine() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let (alice, bob) = (f.editor_of("alice").await, f.editor_of("bob").await);
    alice.move_by(&[id], 10.0, 0.0).await.unwrap();
    bob.set_style(&[id], &red()).await.unwrap();
    assert!(alice.undo().await.unwrap());
    assert_eq!(frame_of(&*alice, id).await.x, 0.0, "the move is gone");
    assert_eq!(fill(&*alice, id).await, "#ff0000", "the colour stays");
}

#[tokio::test]
async fn a_group_move_is_undone_for_the_members_nobody_else_touched() {
    let f = fixture().await;
    let ids = [
        f.rect(0.0, 0.0).await,
        f.rect(20.0, 0.0).await,
        f.rect(40.0, 0.0).await,
    ];
    let g = f.ed.group(&ids).await.unwrap();
    let (alice, bob) = (f.editor_of("alice").await, f.editor_of("bob").await);
    alice.move_by(&[g], 0.0, 100.0).await.unwrap();
    bob.move_by(&[ids[1]], 5.0, 0.0).await.unwrap();
    assert!(alice.undo().await.unwrap());
    assert_eq!(frame_of(&*alice, ids[0]).await.y, 0.0);
    assert_eq!(frame_of(&*alice, ids[2]).await.y, 0.0);
    let mid = frame_of(&*alice, ids[1]).await;
    assert_eq!(
        (mid.x, mid.y),
        (25.0, 100.0),
        "Bob's member stays where he put it"
    );
}

#[tokio::test]
async fn add_delete_align_reorder_group_and_copy_are_undoable_and_redoable() {
    let f = fixture().await;
    let ed = &*f.ed;
    let a = f.rect(0.0, 0.0).await;
    assert!(ed.undo().await.unwrap(), "undo of add");
    assert!(all(ed, f.page).await.is_empty());
    assert!(ed.redo().await.unwrap());
    assert_eq!(all(ed, f.page).await.len(), 1);

    ed.delete(&[a]).await.unwrap();
    assert!(all(ed, f.page).await.is_empty());
    assert!(ed.undo().await.unwrap() && ed.redo().await.unwrap() && ed.undo().await.unwrap());
    assert_eq!(all(ed, f.page).await.len(), 1);

    let b = f.rect(50.0, 30.0).await;
    ed.align(&[a, b], Align::Left).await.unwrap();
    assert_eq!(frame_of(ed, b).await.x, 0.0);
    ed.undo().await.unwrap();
    assert_eq!(frame_of(ed, b).await.x, 50.0);
    ed.redo().await.unwrap();
    assert_eq!(frame_of(ed, b).await.x, 0.0);

    let before: Vec<_> = all(ed, f.page).await.iter().map(|e| e.id).collect();
    ed.reorder(&[a], None).await.unwrap();
    let after: Vec<_> = all(ed, f.page).await.iter().map(|e| e.id).collect();
    assert_eq!(after, vec![b, a]);
    ed.undo().await.unwrap();
    assert_eq!(
        all(ed, f.page)
            .await
            .iter()
            .map(|e| e.id)
            .collect::<Vec<_>>(),
        before
    );

    let g = ed.group(&[a, b]).await.unwrap();
    assert_eq!(ed.get_element(a).await.unwrap().unwrap().parent, Some(g));
    ed.undo().await.unwrap();
    assert_eq!(ed.get_element(a).await.unwrap().unwrap().parent, None);
    assert_eq!(all(ed, f.page).await.len(), 2, "the group is gone again");
    ed.redo().await.unwrap();

    let copies = ed.copy(&[g], 5.0, 5.0).await.unwrap();
    assert_eq!(all(ed, f.page).await.len(), 6);
    ed.undo().await.unwrap();
    assert_eq!(all(ed, f.page).await.len(), 3);
    assert!(ed.get_element(copies[0]).await.unwrap().unwrap().deleted);
}

#[tokio::test]
async fn a_new_editor_has_an_empty_history() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    f.ed.move_by(&[id], 1.0, 1.0).await.unwrap();
    let fresh = f.editor_of("someone-else").await;
    assert!(!fresh.undo().await.unwrap());
    assert!(!fresh.redo().await.unwrap());
    assert_eq!(frame_of(&*fresh, id).await.x, 1.0, "nothing was taken back");
}

#[tokio::test]
async fn undo_leaves_a_locked_element_alone() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let (alice, bob) = (f.editor_of("alice").await, f.editor_of("bob").await);
    alice.move_by(&[id], 10.0, 0.0).await.unwrap();
    bob.set_locked(&[id], true).await.unwrap();
    assert!(!alice.undo().await.unwrap());
    assert_eq!(frame_of(&*alice, id).await.x, 10.0);
}
