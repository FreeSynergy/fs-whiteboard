//! Constants of the stored objects.
//!
//! Pattern: Registry (constants) — AS2 types, context and media types used by this program in one place.

/// Store type of an element envelope.
pub const ELEMENT_TYPE: &str = "WhiteboardElement";
/// AS2 type of a board envelope.
pub const BOARD_TYPE: &str = "Collection";
/// `context` of the board objects.
pub const BOARD_CONTEXT: &str = "whiteboard";
pub(crate) const ELEMENT_CONTENT: &str = "application/vnd.freesynergy.whiteboard-element+json";
pub(crate) const BOARD_CONTENT: &str = "application/vnd.freesynergy.whiteboard-board+json";
