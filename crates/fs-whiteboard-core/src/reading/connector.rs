//! Where the ends of a connector are.
//!
//! Pattern: Null Object (for a missing element) — an end on a deleted element stays at that element's
//! last frame, which a deleted element keeps; an end on nothing is free.

use fs_store::ObjectId;

use crate::model::{Point, Rect};

/// One resolved end of a connector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum End {
    /// Free at a point.
    Free(Point),
    /// Following an element.
    Attached {
        /// The element.
        element: ObjectId,
        /// Its frame; the last one if it is deleted.
        frame: Rect,
        /// True if the element is deleted.
        deleted: bool,
    },
}

impl End {
    /// Where the end is drawn: the free point, or the middle of the frame (the last one if deleted).
    pub fn point(&self) -> Point {
        match self {
            End::Free(p) => *p,
            End::Attached { frame, .. } => frame.center(),
        }
    }
}
