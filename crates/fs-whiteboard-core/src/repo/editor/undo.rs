//! `undo` and `redo` of this editor only.
//!
//! Pattern: Command (apply / invert) — an edit is taken back unit by unit. A unit whose current
//! clock is not the one this edit wrote was changed by someone else since: it is skipped, and
//! their work stays. A skipped locked element stays as it is too. When nothing was left to do the
//! answer is `false` and the edit is gone from the history.

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

    async fn step(&self, undo: bool) -> Result<bool> {
        let popped = {
            let mut h = self
                .history
                .lock()
                .map_err(|_| crate::Error::invalid("history poisoned"))?;
            if undo { h.undo.pop() } else { h.redo.pop() }
        };
        let Some(edit) = popped else { return Ok(false) };
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        let mut done: Vec<UnitChange> = Vec::new();
        for ch in edit.iter().rev() {
            let intact = st.clock(ch.element, ch.unit) == ch.clock;
            let locked =
                st.elements.get(&ch.element).is_none_or(|e| e.locked) && ch.unit != Unit::Locked;
            if !intact || locked {
                continue;
            }
            let value = if undo { ch.old.clone() } else { ch.new.clone() };
            tx.set(st, ch.element, vec![(ch.unit, value)])?;
            done.push(ch.clone());
        }
        if tx.changes.is_empty() {
            return Ok(false);
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
                h.redo.push(rewritten);
            } else {
                h.undo.push(rewritten);
            }
        }
        Ok(true)
    }
}
