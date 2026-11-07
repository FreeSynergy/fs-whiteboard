//! Align rules, pure.

use std::str::FromStr;

use fs_objects::ObjectId;
use fs_whiteboard_core::geometry::align;
use fs_whiteboard_core::{Align, Rect};

fn id(n: u8) -> ObjectId {
    ObjectId::from_str(&format!("01900000-0000-7000-8000-0000000000{n:02x}")).unwrap()
}

#[test]
fn align_rules() {
    let items = [
        (id(1), Rect::new(0.0, 0.0, 10.0, 10.0)),
        (id(2), Rect::new(30.0, 40.0, 20.0, 10.0)),
        (id(3), Rect::new(100.0, 5.0, 10.0, 30.0)),
    ];
    let d = |how| align(&items, how);
    assert_eq!(
        d(Align::Left),
        vec![(id(2), -30.0, 0.0), (id(3), -100.0, 0.0)]
    );
    assert_eq!(
        d(Align::Right),
        vec![(id(1), 100.0, 0.0), (id(2), 60.0, 0.0)]
    );
    assert_eq!(d(Align::Top), vec![(id(2), 0.0, -40.0), (id(3), 0.0, -5.0)]);
    assert_eq!(d(Align::Bottom).len(), 2);
    assert_eq!(d(Align::CenterX).len(), 3);
    assert_eq!(d(Align::CenterY).len(), 3);
    assert_eq!(
        d(Align::DistributeX),
        vec![(id(2), 15.0, 0.0)],
        "gaps of 35 between 0..10, 45..65 and 100..110"
    );
    assert!(
        align(&items[..2], Align::DistributeX).is_empty(),
        "two cannot be spread"
    );
    assert!(align(&[], Align::Left).is_empty());
}
