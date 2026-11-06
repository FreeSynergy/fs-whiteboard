//! The contract as Rust traits: boards, the editor of one board, the change feed.
//!
//! Pattern: Facade (declarations) — callers depend on these traits, never on `WhiteboardRepo`.

mod boards;
mod editor;
mod feed;

pub use boards::Boards;
pub use editor::BoardEditor;
pub use feed::{Change, Subscription, WhiteboardFeed};
