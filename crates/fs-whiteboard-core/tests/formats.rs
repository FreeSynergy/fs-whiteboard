//! The own format: round trip and selections.

mod common;

use common::{all, busy, exported, fixture};
use fs_whiteboard_core::{Boards, Error, Kind};
const OWN: &str = "application/vnd.freesynergy.whiteboard+json";
const SVG: &str = "image/svg+xml";

#[tokio::test]
async fn own_format_round_trip_keeps_everything_visible_and_gives_new_ids() {
    let f = fixture().await;
    let old = busy(&f).await;
    let blob = f.ed.export(None, &[], OWN).await.unwrap();
    assert_eq!(blob.media_type, OWN);
    let copy = f.repo.import(OWN, &blob.bytes).await.unwrap();
    assert_ne!(copy.id, f.board);
    assert_eq!((copy.name.as_str(), copy.pages.len()), ("Test board", 2));
    assert_eq!(copy.pages[1].name, "Second");
    let ed2 = f.repo.open(copy.id).await.unwrap().unwrap();
    let (p1, p2) = (copy.pages[0].id, copy.pages[1].id);
    assert!(!old.contains(&p1) && p1 != f.page);
    let (before, after) = (all(&*f.ed, f.page).await, all(&*ed2, p1).await);
    assert_eq!(before.len(), after.len(), "the deleted one stays out");
    assert_eq!(all(&*ed2, p2).await.len(), 1);
    let new_id: std::collections::BTreeMap<_, _> = before
        .iter()
        .zip(&after)
        .map(|(a, b)| (a.id, b.id))
        .collect();
    for (a, b) in before.iter().zip(&after) {
        assert!(!new_id.contains_key(&b.id) && a.id != b.id, "new ids");
        assert_eq!(
            (&a.order, a.frame, a.angle, &a.style, &a.label),
            (&b.order, b.frame, b.angle, &b.style, &b.label)
        );
        assert_eq!(
            a.parent.map(|p| new_id[&p]),
            b.parent,
            "groups keep their members"
        );
        assert_eq!(b.board, copy.id);
        match (&a.kind, &b.kind) {
            (Kind::Connector(x), Kind::Connector(y)) => {
                assert_eq!(x.source.element.map(|e| new_id[&e]), y.source.element);
                assert_eq!(
                    (x.target.element.map(|e| new_id[&e]), &x.label, x.arrow_end),
                    (y.target.element, &y.label, y.arrow_end)
                );
            }
            (x, y) => assert_eq!(x, y),
        }
    }
}

#[tokio::test]
async fn export_of_a_selection_takes_the_members_along() {
    let f = fixture().await;
    let ids = busy(&f).await;
    let g = ids[5];
    let v = exported(&f.ed.export(Some(f.page), &[g], OWN).await.unwrap().bytes);
    assert_eq!(
        v["elements"].as_array().unwrap().len(),
        3,
        "the group and its two members"
    );
    assert_eq!(v["pages"].as_array().unwrap().len(), 1);
    assert!(matches!(
        f.ed.export(None, &[g], OWN).await,
        Err(Error::Invalid(_))
    ));
    assert!(
        matches!(f.ed.export(None, &[], SVG).await, Err(Error::Invalid(_))),
        "SVG needs a page"
    );
}
