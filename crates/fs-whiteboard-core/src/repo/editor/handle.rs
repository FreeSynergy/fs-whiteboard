//! [`Editor`]: one person's session on one board.
//!
//! Pattern: Command invoker — holds the undo and redo stacks of this session; the state of the
//! board is shared with the other editors through the repository.

use std::sync::Mutex;

use fs_objects::ObjectId;

use crate::repo::handle::WhiteboardRepo;
use crate::repo::history::{Edit, History};

/// An open board with its own history. It ends when it is dropped.
pub(in crate::repo) struct Editor {
    pub(super) repo: WhiteboardRepo,
    pub(super) board: ObjectId,
    pub(super) history: Mutex<History>,
}

impl Editor {
    pub(in crate::repo) fn new(repo: WhiteboardRepo, board: ObjectId) -> Self {
        Self {
            repo,
            board,
            history: Mutex::new(History::default()),
        }
    }

    /// Remembers an edit for undo.
    pub(super) fn record(&self, edit: Edit) {
        if let Ok(mut h) = self.history.lock() {
            h.record(edit);
        }
    }
}
