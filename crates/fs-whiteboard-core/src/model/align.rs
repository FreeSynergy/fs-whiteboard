//! How selected elements are aligned.
//!
//! Pattern: Strategy (data) — the value picks the rule that `geometry::align` applies.

/// The eight ways of `align` in the contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    /// Left edges to the leftmost.
    Left,
    /// Centres to the middle of the selection, horizontally.
    CenterX,
    /// Right edges to the rightmost.
    Right,
    /// Top edges to the topmost.
    Top,
    /// Centres to the middle of the selection, vertically.
    CenterY,
    /// Bottom edges to the lowest.
    Bottom,
    /// Equal gaps between elements, horizontally; the outer two stay.
    DistributeX,
    /// Equal gaps between elements, vertically; the outer two stay.
    DistributeY,
}
