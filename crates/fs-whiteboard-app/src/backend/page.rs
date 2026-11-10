//! `Backend` of the type `Page`.
//!
//! Pattern: Repository + Command — `list` is `find` (optionally of one board); the rest are calls of `BoardEditor`.

use std::sync::Arc;

use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Values};
use fs_whiteboard_core::describe::PAGE;

use super::{guard, is_supported, records};
use crate::convert::{id, need_id, one, show};
use crate::core::Core;

/// Pages over the core.
pub struct PageBackend {
    core: Arc<Core>,
}

impl PageBackend {
    /// Pages of `core`.
    pub fn new(core: Arc<Core>) -> Self {
        Self { core }
    }

    /// Whether the core can run `command` (the table's answer).
    pub fn supported(&self, command: &str) -> bool {
        is_supported(PAGE, command)
    }

    fn bounds(&self, input: &Values) -> Result<Outcome, BackendError> {
        let page = need_id(input, "id")?;
        let board = self.core.board_of_page(page)?;
        let editor = self.core.editor(board.id)?;
        let mut out = Outcome {
            id: Some(self.core.iri(page)),
            ..Outcome::default()
        };
        // A page with nothing shown has no bounds: the result is empty, which is an answer.
        if let Some(r) = self.core.run(editor.bounds(page))? {
            for (k, n) in [
                ("x", r.x),
                ("y", r.y),
                ("width", r.width),
                ("height", r.height),
            ] {
                out.values.insert(k.into(), vec![show(n)]);
            }
        }
        Ok(out)
    }
}

impl Backend for PageBackend {
    fn list(&self, query: &Values) -> Result<Vec<Record>, BackendError> {
        let only = id(query, "board")?;
        Ok(self
            .core
            .pages()?
            .iter()
            .filter(|(b, _)| only.is_none_or(|o| o == b.id))
            .map(|(b, p)| records::page(&self.core, b, p))
            .collect())
    }

    fn run(&self, name: &str, input: &Values) -> Result<Outcome, BackendError> {
        guard(PAGE, name)?;
        match name {
            "add" => {
                let board = need_id(input, "board")?;
                let editor = self.core.editor(board)?;
                let (_, page) = self
                    .core
                    .run(editor.add_page(one(input, "name").unwrap_or("")))?;
                Ok(Outcome {
                    id: Some(self.core.iri(page)),
                    ..Outcome::default()
                })
            }
            "remove" => {
                let page = need_id(input, "id")?;
                let board = self.core.board_of_page(page)?;
                self.core
                    .run(self.core.editor(board.id)?.remove_page(page))?;
                Ok(Outcome::default())
            }
            "move" => {
                let page = need_id(input, "id")?;
                let before = id(input, "before")?;
                let board = self.core.board_of_page(page)?;
                self.core
                    .run(self.core.editor(board.id)?.move_page(page, before))?;
                Ok(Outcome {
                    id: Some(self.core.iri(page)),
                    ..Outcome::default()
                })
            }
            "bounds" => self.bounds(input),
            "find" => {
                let mut out = Outcome::default();
                out.values.insert(
                    "pages".into(),
                    self.core
                        .pages()?
                        .iter()
                        .map(|(_, p)| self.core.iri(p.id))
                        .collect(),
                );
                Ok(out)
            }
            _ => Err(BackendError::NotFound),
        }
    }
}
