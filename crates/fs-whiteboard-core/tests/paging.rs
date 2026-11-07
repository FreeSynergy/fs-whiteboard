//! Paging is stable: nothing lost, nothing repeated.

mod common;

use std::collections::BTreeSet;

use common::all;
use fs_objects::ObjectId;

#[tokio::test]
async fn paging_is_stable_and_loses_or_repeats_nothing() {
    let (repo, _) = common::setup().await;
    let repo = repo.with_page_size(4);
    let board = fs_whiteboard_core::Boards::create(&repo, "paged")
        .await
        .unwrap();
    let page = board.pages[0].id;
    let ed = fs_whiteboard_core::Boards::open(&repo, board.id)
        .await
        .unwrap()
        .unwrap();
    let mut ids = Vec::new();
    for i in 0..23 {
        ids.push(common::add_rect(&*ed, page, f64::from(i) * 12.0, 0.0).await);
    }
    let whole: Vec<ObjectId> = {
        let big = repo.with_page_size(1000);
        let e2 = fs_whiteboard_core::Boards::open(&big, board.id)
            .await
            .unwrap()
            .unwrap();
        all(&*e2, page).await.iter().map(|e| e.id).collect()
    };
    assert_eq!(whole.len(), 23);
    let mut seen: Vec<ObjectId> = Vec::new();
    let mut cursor = None;
    let mut round = 0;
    loop {
        let p = ed.elements(page, None, cursor.as_ref()).await.unwrap();
        assert!(p.items.len() <= 4);
        seen.extend(p.items.iter().map(|e| e.id));
        round += 1;
        // between the pages: delete one already seen, one not yet seen, move one, add one at the front
        match round {
            1 => ed.delete(&[seen[0], whole[20]]).await.unwrap(),
            2 => ed
                .move_by(&[whole[10], whole[3]], 500.0, 0.0)
                .await
                .unwrap(),
            3 => drop(common::add_rect(&*ed, page, 0.0, 0.0).await),
            _ => {}
        }
        match p.next {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    let unique: BTreeSet<_> = seen.iter().collect();
    assert_eq!(unique.len(), seen.len(), "nothing repeated");
    for id in &whole {
        if *id != whole[20] {
            assert!(unique.contains(id), "{id} was lost");
        }
    }
    assert!(
        !unique.contains(&whole[20]),
        "deleted before it was reached"
    );
    assert_eq!(seen.len(), 23, "22 survivors and the one added");
    let _ = ids;
}
