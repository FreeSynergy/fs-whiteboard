//! The `Backend` of each data type over the core.
//!
//! Pattern: Repository (the frontend's `Backend` trait) + Adapter — one struct per type over the shared [`Core`]; the
//! list is `find` with an empty query; a command that the table marks unsupported answers `unsupported:<name>` (it is
//! described so the frontend can show it, but it must not run). `supported` is the table's answer, ready for the
//! frontend's `Backend::supported` once the pinned revision has it.

mod board;
mod element;
mod element_add;
mod element_edit;
mod page;
mod records;

pub use board::BoardBackend;
pub use element::ElementBackend;
pub use page::PageBackend;

use fs_admin_frontend::BackendError;
use fs_whiteboard_core::describe::{Ty, supported_commands, table};

/// Whether `command` of `ty` can run now (the table's `supported`).
pub(crate) fn is_supported(ty: Ty, command: &str) -> bool {
    supported_commands(ty).contains(&command)
}

/// `Ok` for a command that exists and is supported; `NotFound` for an unknown name; the refusal for the rest.
pub(crate) fn guard(ty: Ty, command: &str) -> Result<(), BackendError> {
    if is_supported(ty, command) {
        Ok(())
    } else if table().iter().any(|r| r.ty == ty && r.name == command) {
        Err(BackendError::Unavailable(format!("unsupported:{command}")))
    } else {
        Err(BackendError::NotFound)
    }
}
