//! [`ElementDoc`]: an element with the clock of each of its units.
//!
//! Pattern: Value object — the unit of merging for elements. The clocks are kept in the stored
//! content under `fs:clocks`; a local change takes one more than the highest counter seen on the object.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::clock::Clock;
use crate::error::Result;
use crate::model::Element;
use crate::unit::Unit;

/// An element and the clocks of its units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ElementDoc {
    /// The element.
    #[serde(flatten)]
    pub element: Element,
    /// Clock of each unit.
    #[serde(rename = "fs:clocks")]
    pub clocks: BTreeMap<Unit, Clock>,
}

impl ElementDoc {
    /// A new element written by `actor`: every unit has the clock `(1, actor)`.
    pub fn fresh(element: Element, actor: &str) -> Self {
        let clocks = Unit::ALL
            .into_iter()
            .filter(|u| u.applies_to(&element.kind))
            .map(|u| (u, Clock::after(0, actor)))
            .collect();
        Self { element, clocks }
    }

    /// Drops clocks of units this kind does not have, so a stray clock cannot break a merge.
    pub fn sanitize(&mut self) {
        let kind = &self.element.kind;
        self.clocks.retain(|u, _| u.applies_to(kind));
    }

    /// The clock of a unit; `(0, "")` if it was never written.
    pub fn clock(&self, unit: Unit) -> Clock {
        self.clocks.get(&unit).cloned().unwrap_or_default()
    }

    /// The highest counter seen on this object.
    pub fn highest(&self) -> u64 {
        self.clocks.values().map(Clock::counter).max().unwrap_or(0)
    }

    /// A local change of some units by `actor`, all under one new clock. Returns that clock.
    pub fn set(&mut self, changes: Vec<(Unit, Value)>, actor: &str) -> Result<Clock> {
        let clock = Clock::after(self.highest(), actor);
        for (unit, value) in changes {
            unit.write(&mut self.element, value)?;
            self.clocks.insert(unit, clock.clone());
        }
        Ok(clock)
    }
}
