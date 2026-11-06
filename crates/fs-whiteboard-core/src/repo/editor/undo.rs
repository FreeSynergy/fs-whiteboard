//! `undo` and `redo` of this editor only.
//!
//! Pattern: Command (apply / invert) — an edit is taken back unit by unit. A unit is skipped if its
//! current clock and value are not the ones this editor last wrote: someone else changed it since
//! (even on the same clock, from another device), and their work stays. A locked element stays as it
//! is too. An edit with nothing left to do is dropped and the next one is tried; when no edit is left
//! the answer is `false`.

use super::handle::Editor;
use crate::error::Result;
use crate::repo::history::{Edit, UnitChange};
use crate::repo::tx::Tx;
use crate::unit::Unit;

impl Editor {
    pub(super) async fn undo(&self) -> Result<bool> {
        self.step(true).await
    }

    pub(super) async fn redo(&self) -> Result<bool> {
        self.step(false).await
    }

    fn pop(&self, undo: bool) -> Option<Edit> {
        self.history.lock().ok()?.pop(undo)
    }

    async fn step(&self, undo: bool) -> Result<bool> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        while let Some(edit) = self.pop(undo) {
            let mut tx = Tx::new(&self.repo.actor);
            let mut done: Vec<UnitChange> = Vec::new();
            for ch in edit.iter().rev() {
                // what this editor last wrote for the unit: the new value, or after an undo the old one
                let (expect, target) = if undo {
                    (&ch.new, &ch.old)
                } else {
                    (&ch.old, &ch.new)
                };
                let Some(e) = st.elements.get(&ch.element) else {
                    continue;
                };
                let intact = st.clock(ch.element, ch.unit) == ch.clock
                    && ch.unit.read(e).as_ref() == Some(expect);
                if !intact || (e.locked && ch.unit != Unit::Locked) {
                    continue;
                }
                tx.set(st, ch.element, vec![(ch.unit, target.clone())])?;
                done.push(ch.clone());
            }
            if tx.changes.is_empty() {
                continue;
            }
            self.repo.commit(st, tx).await?;
            let mut rewritten: Edit = Vec::new();
            if let Ok(mut h) = self.history.lock() {
                for c in done {
                    let new = st.clock(c.element, c.unit);
                    h.rewire(&c, undo, &new);
                    rewritten.push(UnitChange {
                        clock: new,
                        before: c.clock.clone(),
                        ..c
                    });
                }
                if undo {
                    h.push_redo(rewritten);
                } else {
                    h.push_undo(rewritten);
                }
            }
            return Ok(true);
        }
        Ok(false)
    }
}
