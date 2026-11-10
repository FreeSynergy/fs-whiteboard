//! `Backend` of the type `Board`.
//!
//! Pattern: Repository + Command — `list` is `find`; every other command is one call of `Boards` or `BoardEditor`.

use std::sync::Arc;

use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Values};
use fs_objects::ObjectId;
use fs_whiteboard_core::Boards;
use fs_whiteboard_core::describe::BOARD;
use fs_whiteboard_core::format::{JSON_MEDIA_TYPE, SVG_MEDIA_TYPE};

use super::{guard, is_supported, records};
use crate::convert::{id, need, need_choice, need_id, show_flag};
use crate::core::Core;

/// Boards over the core.
pub struct BoardBackend {
    core: Arc<Core>,
}

impl BoardBackend {
    /// Boards of `core`.
    pub fn new(core: Arc<Core>) -> Self {
        Self { core }
    }

    /// Whether the core can run `command` (the table's answer).
    pub fn supported(&self, command: &str) -> bool {
        is_supported(BOARD, command)
    }

    fn media(format: &str) -> &'static str {
        if format == "svg" {
            SVG_MEDIA_TYPE
        } else {
            JSON_MEDIA_TYPE
        }
    }

    fn history(&self, input: &Values, undo: bool) -> Result<Outcome, BackendError> {
        let board = need_id(input, "id")?;
        let editor = self.core.editor(board)?;
        let did = self.core.run(async {
            if undo {
                editor.undo().await
            } else {
                editor.redo().await
            }
        })?;
        let mut out = Outcome {
            id: Some(self.core.iri(board)),
            ..Outcome::default()
        };
        out.values.insert("done".into(), vec![show_flag(did)]);
        Ok(out)
    }

    fn export(&self, input: &Values) -> Result<Outcome, BackendError> {
        let board = need_id(input, "id")?;
        let page: Option<ObjectId> = id(input, "page")?;
        let media = Self::media(need_choice(
            input,
            "format",
            &fs_whiteboard_core::describe::EXPORT_FORMATS,
        )?);
        let editor = self.core.editor(board)?;
        let blob = self.core.run(editor.export(page, &[], media))?;
        let text =
            String::from_utf8(blob.bytes).map_err(|e| BackendError::Unavailable(e.to_string()))?;
        let mut out = Outcome {
            id: Some(self.core.iri(board)),
            ..Outcome::default()
        };
        out.values.insert("data".into(), vec![text]);
        Ok(out)
    }
}

impl Backend for BoardBackend {
    fn list(&self, _query: &Values) -> Result<Vec<Record>, BackendError> {
        Ok(self
            .core
            .boards()?
            .iter()
            .map(|b| records::board(&self.core, b))
            .collect())
    }

    fn run(&self, name: &str, input: &Values) -> Result<Outcome, BackendError> {
        guard(BOARD, name)?;
        let done = |b: &fs_whiteboard_core::Board| Outcome {
            id: Some(b.iri.clone()),
            ..Outcome::default()
        };
        match name {
            "create" => Ok(done(
                &self.core.run(self.core.repo.create(need(input, "name")?))?,
            )),
            "rename" => {
                let (b, n) = (need_id(input, "id")?, need(input, "name")?);
                Ok(done(&self.core.run(self.core.repo.rename(b, n))?))
            }
            "delete" => {
                let b = need_id(input, "id")?;
                self.core.run(self.core.repo.delete(b))?;
                self.core.forget(b);
                Ok(Outcome::default())
            }
            "import" => {
                need_choice(
                    input,
                    "format",
                    &fs_whiteboard_core::describe::BOARD_FORMATS,
                )?;
                let data = need(input, "data")?;
                Ok(done(&self.core.run(
                    self.core.repo.import(JSON_MEDIA_TYPE, data.as_bytes()),
                )?))
            }
            "export" => self.export(input),
            "undo" => self.history(input, true),
            "redo" => self.history(input, false),
            // `find` is the list; asked as a command it answers the same records' identities
            "find" => {
                let ids = self.core.boards()?.iter().map(|b| b.iri.clone()).collect();
                let mut out = Outcome::default();
                out.values.insert("boards".into(), ids);
                Ok(out)
            }
            _ => Err(BackendError::NotFound),
        }
    }
}
