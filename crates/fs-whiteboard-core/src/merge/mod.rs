//! Merging of two versions of the same object, per property. Pure: no I/O, no wall time.
//!
//! Pattern: Strategy (last-writer-wins register per unit, a state-based CRDT) — the result is
//! commutative, associative and idempotent, so replicas that saw the same changes in any order end
//! byte-identical.

mod board;
mod element;
mod objects;

pub use board::merge_board;
pub use element::merge_element;
pub use objects::merge_objects;
