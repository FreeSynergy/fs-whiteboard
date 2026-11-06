//! The spatial index against brute force; bounds.

mod common;

use common::{Fixture, Pair, all, fixture, pair, rect_element};
use fs_store::ObjectId;
use fs_whiteboard_core::geometry::{bounds, union_all};
use fs_whiteboard_core::{BoardEditor, Rect};
use proptest::prelude::*;
use std::collections::BTreeSet;
type Spec = (i16, i16, u16, u16, i16);
type Op = (u8, usize, i16, i16);

fn rect(s: &Spec) -> Rect {
    Rect::new(
        f64::from(s.0),
        f64::from(s.1),
        f64::from(s.2),
        f64::from(s.3),
    )
}

fn angle(a: i16) -> f64 {
    f64::from(a) / 40.0
}

/// What is on the page by brute force: not deleted, bounds after rotation touching the area.
async fn brute(
    ed: &dyn BoardEditor,
    ids: &[ObjectId],
    area: Option<Rect>,
) -> (BTreeSet<ObjectId>, Option<Rect>) {
    let mut found = BTreeSet::new();
    let mut rects = Vec::new();
    for id in ids {
        let e = ed.get_element(*id).await.unwrap().unwrap();
        if e.deleted {
            continue;
        }
        let b = bounds(&e.frame, e.angle);
        rects.push(b);
        if area.is_none_or(|a| b.intersects(&a)) {
            found.insert(e.id);
        }
    }
    (found, union_all(&rects))
}

async fn agree(f: &Fixture, ids: &[ObjectId], area: Rect) {
    let (want, all_bounds) = brute(&*f.ed, ids, Some(area)).await;
    let got: BTreeSet<ObjectId> =
        f.ed.elements(f.page, Some(area), None)
            .await
            .unwrap()
            .items
            .iter()
            .map(|e| e.id)
            .collect();
    assert_eq!(got, want, "area {area:?}");
    let (everything, _) = brute(&*f.ed, ids, None).await;
    let got: BTreeSet<ObjectId> = all(&*f.ed, f.page).await.iter().map(|e| e.id).collect();
    assert_eq!(got, everything);
    let (got, want) = (f.ed.bounds(f.page).await.unwrap(), all_bounds);
    assert_eq!(got.is_some(), want.is_some());
    if let (Some(g), Some(w)) = (got, want) {
        // the union is built in another order, so the last digits may differ
        for (a, b) in [
            (g.x, w.x),
            (g.y, w.y),
            (g.right(), w.right()),
            (g.bottom(), w.bottom()),
        ] {
            assert!((a - b).abs() < 1e-6, "{g:?} vs {w:?}");
        }
    }
}

async fn apply(ed: &dyn BoardEditor, ids: &[ObjectId], (kind, i, a, b): Op) {
    let id = ids[i % ids.len()];
    let _ = match kind % 4 {
        0 => ed.move_by(&[id], f64::from(a), f64::from(b)).await,
        1 => ed.rotate(id, angle(a)).await,
        2 => ed.delete(&[id]).await,
        _ => {
            ed.set_frame(id, Rect::new(f64::from(a), f64::from(b), 30.0, 5.0))
                .await
        }
    };
}

fn run<F: std::future::Future<Output = ()>>(f: F) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f);
}

proptest! {
    #![proptest_config(common::cfg(24))]

    #[test]
    fn index_matches_brute_force_after_edits(
        specs in prop::collection::vec((-200i16..200, -200i16..200, 0u16..80, 0u16..80, -80i16..80), 1..16),
        ops in prop::collection::vec((0u8..4, 0usize..16, -150i16..150, -150i16..150), 0..12),
        areas in prop::collection::vec((-250i16..250, -250i16..250, 0u16..200, 0u16..200), 1..4),
    ) {
        run(async {
            let f = fixture().await;
            let mut ids = Vec::new();
            for s in &specs {
                let mut e = rect_element(f.page, 0.0, 0.0);
                e.frame = rect(s);
                e.angle = angle(s.4);
                ids.push(f.ed.add(e).await.unwrap().id);
            }
            for op in ops {
                apply(&*f.ed, &ids, op).await;
            }
            for a in areas {
                agree(&f, &ids, rect(&(a.0, a.1, a.2, a.3, 0))).await;
            }
        });
    }

    #[test]
    fn index_matches_brute_force_after_apply_remote(
        specs in prop::collection::vec((-200i16..200, -200i16..200, 0u16..80, 0u16..80, -80i16..80), 1..10),
        ops in prop::collection::vec((0u8..4, 0usize..10, -150i16..150, -150i16..150), 1..10),
        area in (-250i16..250, -250i16..250, 0u16..300, 0u16..300),
    ) {
        run(async {
            let Pair { a, b } = pair().await;
            let mut ids = Vec::new();
            for s in &specs {
                let mut e = rect_element(a.page, 0.0, 0.0);
                e.frame = rect(s);
                e.angle = angle(s.4);
                ids.push(a.ed.add(e).await.unwrap().id);
            }
            // b has the board loaded (and indexed) before the elements arrive, then every edit comes
            // from b and travels back through apply_remote
            common::replicate(&a.store, &b.repo, a.board).await;
            for op in ops {
                apply(&*b.ed, &ids, op).await;
            }
            common::replicate(&b.store, &a.repo, a.board).await;
            agree(&a, &ids, rect(&(area.0, area.1, area.2, area.3, 0))).await;
            agree(&b, &ids, rect(&(area.0, area.1, area.2, area.3, 0))).await;
        });
    }
}

#[tokio::test]
async fn bounds_of_a_rotated_element_cover_its_corners() {
    let f = fixture().await;
    let mut e = rect_element(f.page, 0.0, 0.0);
    e.frame = Rect::new(0.0, 0.0, 100.0, 20.0);
    e.angle = std::f64::consts::FRAC_PI_2;
    f.ed.add(e).await.unwrap();
    let b = f.ed.bounds(f.page).await.unwrap().unwrap();
    // turned a quarter around (50, 10): 20 wide, 100 high
    for (got, want) in [
        (b.x, 40.0),
        (b.y, -40.0),
        (b.width, 20.0),
        (b.height, 100.0),
    ] {
        assert!((got - want).abs() < 1e-9, "{b:?}");
    }
    assert!(f.ed.bounds(f.page).await.unwrap().is_some());
    let empty = fixture().await;
    assert!(empty.ed.bounds(empty.page).await.unwrap().is_none());
}
