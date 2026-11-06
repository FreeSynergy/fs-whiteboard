//! Regression: floats come back bit for bit after a reload.

mod common;

use common::{rect_element, setup};
use fs_whiteboard_core::{Boards, Rect, WhiteboardRepo};

#[tokio::test]
async fn floats_come_back_bit_for_bit_after_a_reload() {
    let (repo, store) = setup().await;
    let b = repo.create("Floats").await.unwrap();
    let ed = repo.open(b.id).await.unwrap().unwrap();
    let id = ed
        .add(rect_element(b.pages[0].id, 0.0, 0.0))
        .await
        .unwrap()
        .id;
    let mut x = 0x2545_F491_4F6C_DD1Du64;
    let mut sent = Vec::new();
    for _ in 0..300 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        #[allow(clippy::cast_precision_loss)]
        let v = (x >> 11) as f64 / 9_007_199_254_740_992.0 * 1000.0 - 500.0;
        ed.set_frame(id, Rect::new(v, v / 3.0, 5.5 * v.abs(), 0.1 + v.abs()))
            .await
            .unwrap();
        sent.push(ed.get_element(id).await.unwrap().unwrap().frame);
    }
    let last = *sent.last().unwrap();
    let fresh = WhiteboardRepo::new(store, "me");
    let ed2 = fresh.open(b.id).await.unwrap().unwrap();
    let got = ed2.get_element(id).await.unwrap().unwrap().frame;
    assert_eq!(
        [
            got.x.to_bits(),
            got.y.to_bits(),
            got.width.to_bits(),
            got.height.to_bits()
        ],
        [
            last.x.to_bits(),
            last.y.to_bits(),
            last.width.to_bits(),
            last.height.to_bits()
        ]
    );
}
