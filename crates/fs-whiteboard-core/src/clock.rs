//! The logical clock of one merge unit.
//!
//! Pattern: Value object (Lamport clock) — `(counter, actor)`, larger wins. No wall time is ever
//! read, so changing a machine's clock cannot change a merge.

use serde::{Deserialize, Serialize};

/// `(counter, actor)` of the last change of a unit. Ordered by counter, then by actor text.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Clock(pub u64, pub String);

impl Clock {
    /// The clock of a change by `actor` after the highest counter seen so far.
    pub fn after(seen: u64, actor: &str) -> Self {
        Self(seen + 1, actor.to_string())
    }

    /// The counter part.
    pub fn counter(&self) -> u64 {
        self.0
    }
}

/// True if the candidate `(clock, value text)` beats the current one. Equal clocks (the same actor on
/// two devices) fall back to the value text, so the result never depends on the order of arrival.
pub(crate) fn wins(cur: &Clock, cur_val: &str, cand: &Clock, cand_val: &str) -> bool {
    (cand, cand_val) > (cur, cur_val)
}
