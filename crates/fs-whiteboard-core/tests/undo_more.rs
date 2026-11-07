//! Regression: undo goes on to older edits, respects equal clocks, and is capped.

mod common;

use common::{ME, fixture, frame_of, pair};
use fs_whiteboard_core::{Boards, Element, Kind, Rect, StyleField};

async fn fill(ed: &dyn fs_whiteboard_core::BoardEditor, id: fs_objects::ObjectId) -> String {
    ed.get_element(id).await.unwrap().unwrap().style.fill
}

#[tokio::test]
async fn an_edit_with_nothing_left_to_do_hands_over_to_the_older_one() {
    let f = fixture().await;
    let id = f.rect(0.0, 0.0).await;
    let (alice, bob) = (f.editor_of("alice").await, f.editor_of("bob").await);
    alice.move_by(&[id], 10.0, 0.0).await.unwrap();
    alice
        .set_style(&[id], &[StyleField::Fill("#ff0000".into())])
        .await
        .unwrap();
    bob.set_style(&[id], &[StyleField::Fill("#00ff00".into())])
        .await
        .unwrap();
    assert!(
        alice.undo().await.unwrap(),
        "the colour is Bob's, but the move is still hers"
    );
    assert_eq!(frame_of(&*alice, id).await.x, 0.0);
    assert_eq!(fill(&*alice, id).await, "#00ff00");
    assert!(!alice.undo().await.unwrap());
}

#[tokio::test]
async fn the_same_actor_on_two_devices_does_not_undo_the_other_devices_value() {
    let p = pair().await;
    let id = common::add_rect(&*p.a.ed, p.a.page, 0.0, 0.0).await;
    p.sync().await;
    // both devices act as the same person, from the same state: the same clock, two values
    let (one, two) = (p.a.repo.as_actor(ME), p.b.repo.as_actor(ME));
    let (ed_one, ed_two) = (
        one.open(p.a.board).await.unwrap().unwrap(),
        two.open(p.a.board).await.unwrap().unwrap(),
    );
    ed_one
        .set_style(&[id], &[StyleField::Fill("#aaaaaa".into())])
        .await
        .unwrap();
    ed_two
        .set_style(&[id], &[StyleField::Fill("#bbbbbb".into())])
        .await
        .unwrap();
    p.sync().await;
    assert_eq!(fill(&*ed_one, id).await, "#bbbbbb");
    assert!(
        !ed_one.undo().await.unwrap(),
        "the value is the other device's"
    );
    assert_eq!(fill(&*ed_one, id).await, "#bbbbbb");
    assert_eq!(fill(&*ed_two, id).await, "#bbbbbb");
    assert!(
        ed_two.undo().await.unwrap(),
        "while the device that wrote it may take it back"
    );
    assert_eq!(fill(&*ed_two, id).await, "");
}

#[tokio::test]
async fn the_history_is_capped_and_drops_the_oldest_edits() {
    let f = fixture().await;
    let e =
        f.ed.add(Element::new(
            f.page,
            Kind::Text {
                text: String::new(),
            },
            Rect::new(0.0, 0.0, 1.0, 1.0),
        ))
        .await
        .unwrap();
    let big = |c: char| c.to_string().repeat(1024 * 1024);
    for i in 0..45 {
        f.ed.set_text(e.id, &big(if i % 2 == 0 { 'a' } else { 'b' }))
            .await
            .unwrap();
    }
    let mut undone = 0;
    while f.ed.undo().await.unwrap() {
        undone += 1;
    }
    // each edit keeps two copies of a megabyte: 64 MiB hold about 32 of them
    assert!((20..=33).contains(&undone), "{undone} edits were kept");
}
