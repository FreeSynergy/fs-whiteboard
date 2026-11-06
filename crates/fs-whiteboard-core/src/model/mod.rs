//! The board model as plain typed values (the Rust form of `whiteboard.capnp`).
//!
//! Pattern: Facade (declarations) — one file per concept.

mod align;
mod board;
mod element;
mod geom;
mod kind;
mod paging;
mod style;

pub use align::Align;
pub use board::{Board, Page};
pub use element::Element;
pub use geom::{Point, Rect, StrokePoint};
pub use kind::{Anchor, Connector, ConnectorEnd, Kind, ShapeKind};
pub use paging::{Cursor, Paged};
pub use style::{Style, StyleField};
