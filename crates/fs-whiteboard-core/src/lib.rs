//! `fs-whiteboard-core`: program core of FreeSynergy boards (no GUI, no capnp server, no Blinc).
//!
//! A board is an AS2 `Collection` in the object store; every element on it is an object of its own
//! (`WhiteboardElement`). Each property of an element merges by itself with a logical clock, so one
//! person's colour change does not undo another's move (`merge`). What is on the board is computed
//! from the loaded elements by pure rules (`reading`, `geometry`, `order`), found quickly through an
//! R-tree (`spatial`), and written as SVG or the own JSON format (`format`, `svg`).
//! Start with [`WhiteboardRepo`] and the traits [`Boards`], [`BoardEditor`], [`WhiteboardFeed`].
//! See `DESIGN.md` for the pattern map.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Pedantic lints that would only add noise to this crate's small, documented API: every fallible fn
// returns the crate `Result` (documented once on `Error`), and results of pure getters are used anyway.
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod clock;
mod consts;
pub mod describe;
mod doc;
mod error;
pub mod format;
pub mod geometry;
pub mod merge;
pub mod model;
pub mod order;
pub mod reading;
mod repo;
pub mod spatial;
pub mod svg;
pub mod traits;
mod unit;
mod unsupported;
pub mod validate;

pub use clock::Clock;
pub use consts::{BOARD_CONTEXT, BOARD_TYPE, ELEMENT_TYPE};
pub use doc::{BoardDoc, ElementDoc};
pub use error::{Error, Result};
pub use model::{
    Align, Anchor, Board, Connector, ConnectorEnd, Cursor, Element, Kind, Page, Paged, Point, Rect,
    ShapeKind, StrokePoint, Style, StyleField,
};
pub use repo::WhiteboardRepo;
pub use traits::{BoardEditor, Boards, Change, Subscription, WhiteboardFeed};
pub use unit::Unit;
pub use unsupported::not_possible;
