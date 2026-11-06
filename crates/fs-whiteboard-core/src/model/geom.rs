//! Points, rectangles and stroke points.
//!
//! Pattern: Newtype-like value types — coordinates are `f64` board units; the entry points refuse
//! non-finite numbers (see `validate`), so a stored value is always finite.

use serde::{Deserialize, Serialize};

/// A point on the board.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
}

/// A rectangle: position of the top-left corner and size (never negative).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Width, at least 0.
    pub width: f64,
    /// Height, at least 0.
    pub height: f64,
}

impl Rect {
    /// A rectangle from position and size.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Right edge.
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// Bottom edge.
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// The middle of the rectangle.
    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    /// True if the two rectangles share at least a point (touching counts).
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.bottom()
            && other.y <= self.bottom()
    }

    /// The smallest rectangle holding both.
    #[must_use]
    pub fn union(&self, other: &Rect) -> Rect {
        let (x, y) = (self.x.min(other.x), self.y.min(other.y));
        Rect::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }

    /// Moved by `(dx, dy)`.
    #[must_use]
    pub fn moved(&self, dx: f64, dy: f64) -> Rect {
        Rect::new(self.x + dx, self.y + dy, self.width, self.height)
    }
}

/// One point of a stroke, relative to the frame: `x` and `y` are fractions of its width and height,
/// so the stroke follows when the frame moves or scales.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    /// Fraction of the frame width.
    pub x: f64,
    /// Fraction of the frame height.
    pub y: f64,
    /// Pen pressure 0..1; `None` = not known (NaN on the wire).
    pub pressure: Option<f32>,
}
