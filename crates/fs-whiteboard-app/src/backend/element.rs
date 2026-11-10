//! `Backend` of the type `Element`.
//!
//! Pattern: Repository + Command — `list` is `find` (optionally of one page); each command is one method of
//! `BoardEditor`, found through the board that holds the element. The work is split by what it does: `element_add`
//! builds a new element from the typed values, `element_edit` changes one.

use std::sync::Arc;

use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Values};
use fs_whiteboard_core::describe::ELEMENT;

use super::{element_add, element_edit, guard, is_supported, records};
use crate::convert::{id, need_id};
use crate::core::Core;

/// Elements over the core.
pub struct ElementBackend {
    core: Arc<Core>,
}

impl ElementBackend {
    /// Elements of `core`.
    pub fn new(core: Arc<Core>) -> Self {
        Self { core }
    }

    /// Whether the core can run `command` (the table's answer).
    pub fn supported(&self, command: &str) -> bool {
        is_supported(ELEMENT, command)
    }

    fn all(
        &self,
        only: Option<fs_objects::ObjectId>,
    ) -> Result<Vec<fs_whiteboard_core::Element>, BackendError> {
        let mut out = Vec::new();
        for (board, page) in self.core.pages()? {
            if only.is_none_or(|p| p == page.id) {
                out.extend(self.core.elements(board.id, page.id)?);
            }
        }
        Ok(out)
    }
}

impl Backend for ElementBackend {
    fn list(&self, query: &Values) -> Result<Vec<Record>, BackendError> {
        let only = id(query, "page")?;
        Ok(self
            .all(only)?
            .iter()
            .map(|e| records::element(&self.core, e))
            .collect())
    }

    fn run(&self, name: &str, input: &Values) -> Result<Outcome, BackendError> {
        guard(ELEMENT, name)?;
        let core = &self.core;
        match name {
            "add" => element_add::add(core, input),
            "find" => {
                let found = self.all(id(input, "page")?)?;
                let mut out = Outcome::default();
                out.values.insert(
                    "elements".into(),
                    found.iter().map(|e| core.iri(e.id)).collect(),
                );
                Ok(out)
            }
            "group" | "align" => element_edit::many(core, name, input),
            _ => {
                let target = need_id(input, "id")?;
                element_edit::one(core, name, target, input)
            }
        }
    }
}
