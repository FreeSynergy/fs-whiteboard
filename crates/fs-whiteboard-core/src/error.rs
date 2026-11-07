//! Error type of the whiteboard core.
//!
//! Pattern: none (plain error enum via `thiserror`) — one `Error` for every public operation, with
//! `From` conversions for the store and JSON layers. Refusals are named, so callers can tell them apart.

use fs_objects::ObjectId;

/// Result of every fallible operation of this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong in the whiteboard core.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The object store failed.
    #[error("object store: {0}")]
    Store(#[from] fs_objects::Error),
    /// Stored or imported JSON could not be read or written.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// No board, page or element with this id.
    #[error("{what} {id} not found")]
    NotFound {
        /// What was looked for (`board`, `page`, `element`).
        what: &'static str,
        /// The id that was not found.
        id: ObjectId,
    },
    /// The id is not on this board (every id of an edit must be on the board of the editor).
    #[error("{0} is not on this board")]
    Foreign(ObjectId),
    /// The element is locked; only `set_locked` may touch it.
    #[error("element {0} is locked")]
    Locked(ObjectId),
    /// The change would make an element its own ancestor.
    #[error("refused: {parent} cannot be the parent of {child}, it would form a cycle")]
    Cycle {
        /// The element that was to get a parent.
        child: ObjectId,
        /// The proposed parent.
        parent: ObjectId,
    },
    /// The input breaks a rule (non-finite number, negative size, opacity outside 0..1, ...).
    #[error("invalid: {0}")]
    Invalid(String),
    /// The input is over a limit (stroke points, text length, import size).
    #[error("too large: {0}")]
    TooLarge(String),
    /// The last page of a board cannot be removed.
    #[error("the last page of a board cannot be removed")]
    LastPage,
    /// The explicit "not possible" of a provider (ADR 0045).
    #[error(transparent)]
    Unsupported(#[from] fs_interfaces::Unsupported),
}

impl Error {
    /// A refusal for a bad value.
    pub fn invalid(text: impl Into<String>) -> Self {
        Self::Invalid(text.into())
    }
}
