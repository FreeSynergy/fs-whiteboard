//! One editor's undo and redo stacks.
//!
//! Pattern: Command — an edit is a list of `(element, unit, old value, new value, clock it wrote)`.
//! Undo writes the old value as a new change with a new clock, unless the unit's clock is no longer
//! the one this edit wrote: someone else changed it since, and their work stays.

use fs_store::ObjectId;
use serde_json::Value;

use crate::clock::Clock;
use crate::unit::Unit;

/// One unit changed by an edit.
#[derive(Clone, Debug)]
pub(super) struct UnitChange {
    pub(super) element: ObjectId,
    pub(super) unit: Unit,
    pub(super) old: Value,
    pub(super) new: Value,
    /// The clock of the last write of this unit by this edit (or by its undo or redo).
    pub(super) clock: Clock,
}

pub(super) type Edit = Vec<UnitChange>;

const DEPTH: usize = 1000;

#[derive(Default)]
pub(super) struct History {
    pub(super) undo: Vec<Edit>,
    pub(super) redo: Vec<Edit>,
}

impl History {
    /// A new edit: redo is gone.
    pub(super) fn record(&mut self, edit: Edit) {
        if edit.is_empty() {
            return;
        }
        self.redo.clear();
        self.undo.push(edit);
        if self.undo.len() > DEPTH {
            self.undo.remove(0);
        }
    }
}
