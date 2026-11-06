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
    /// The clock that write replaced.
    pub(super) before: Clock,
}

pub(super) type Edit = Vec<UnitChange>;

const DEPTH: usize = 1000;

#[derive(Default)]
pub(super) struct History {
    pub(super) undo: Vec<Edit>,
    pub(super) redo: Vec<Edit>,
}

impl History {
    /// After one unit was undone (`undone`) or redone: the other entries about the same unit follow the
    /// clock it now has, so the older edit of that unit can still be taken back. `before` is what
    /// the edit replaced, `replaced` what the undo or redo replaced, `new` what it wrote.
    pub(super) fn rewire(&mut self, c: &UnitChange, undone: bool, new: &Clock) {
        let same = |e: &&mut UnitChange| e.element == c.element && e.unit == c.unit;
        let follow = |list: &mut Vec<Edit>, from: &Clock| {
            for e in list.iter_mut().flatten().filter(same) {
                if e.clock == *from {
                    e.clock = new.clone();
                }
            }
        };
        if undone {
            follow(&mut self.undo, &c.before);
        }
        follow(&mut self.redo, &c.clock);
    }

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
