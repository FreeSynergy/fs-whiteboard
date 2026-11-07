//! One editor's undo and redo stacks.
//!
//! Pattern: Command — an edit is a list of `(element, unit, old value, new value, clock it wrote)`.
//! Undo writes the old value as a new change with a new clock, unless the unit's clock is no longer
//! the one this edit wrote: someone else changed it since, and their work stays.

use fs_objects::ObjectId;
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

/// Most bytes one editor keeps for undo and redo; the oldest edits go first.
const CAP_BYTES: usize = 64 * 1024 * 1024;

fn size(edit: &Edit) -> usize {
    edit.iter()
        .map(|c| 160 + c.old.to_string().len() + c.new.to_string().len())
        .sum()
}

#[derive(Default)]
pub(super) struct History {
    pub(super) undo: Vec<Edit>,
    pub(super) redo: Vec<Edit>,
    bytes: usize,
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
        let dropped: usize = self.redo.drain(..).map(|e| size(&e)).sum();
        self.bytes = self.bytes.saturating_sub(dropped);
        self.push_undo(edit);
    }

    pub(super) fn push_undo(&mut self, edit: Edit) {
        self.bytes += size(&edit);
        self.undo.push(edit);
        self.trim();
    }

    pub(super) fn push_redo(&mut self, edit: Edit) {
        self.bytes += size(&edit);
        self.redo.push(edit);
        self.trim();
    }

    pub(super) fn pop(&mut self, undo: bool) -> Option<Edit> {
        let e = if undo {
            self.undo.pop()
        } else {
            self.redo.pop()
        }?;
        self.bytes = self.bytes.saturating_sub(size(&e));
        Some(e)
    }

    /// Drops the oldest edits while over the cap (the newest edit always stays).
    fn trim(&mut self) {
        while self.bytes > CAP_BYTES && self.undo.len() + self.redo.len() > 1 {
            let old = if self.undo.is_empty() {
                self.redo.remove(0)
            } else {
                self.undo.remove(0)
            };
            self.bytes = self.bytes.saturating_sub(size(&old));
        }
    }
}
