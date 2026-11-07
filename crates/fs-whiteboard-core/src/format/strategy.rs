//! The [`Format`] strategy and the values it works on.
//!
//! Pattern: Strategy — one implementation per media type; `Scene` is what is written, `Imported` what
//! is read.

use crate::doc::{BoardDoc, ElementDoc};
use crate::error::Result;
use crate::model::{Element, Page};
use crate::unsupported::not_possible;

/// What is shown, ready to be written: the pages that count, and the elements in order, back to
/// front, with a group's frame as the union of its members.
#[derive(Clone, Debug)]
pub struct Scene {
    /// Name of the board.
    pub name: String,
    /// The pages in order.
    pub pages: Vec<Page>,
    /// The elements, back to front.
    pub elements: Vec<Element>,
    /// Where the two ends of each connector are (a deleted element's last frame included).
    pub ends: std::collections::BTreeMap<
        fs_objects::ObjectId,
        (crate::model::Point, crate::model::Point),
    >,
}

/// A board read from a file: new ids, fresh clocks, references inside the file kept.
#[derive(Clone, Debug)]
pub struct Imported {
    /// The board with its pages.
    pub board: BoardDoc,
    /// Its elements (the `board` field is filled in by the store side).
    pub elements: Vec<ElementDoc>,
}

/// One media type.
pub trait Format: Sync {
    /// The media type.
    fn media_type(&self) -> &'static str;
    /// True if the format can hold several pages (the whole board).
    fn whole_board(&self) -> bool {
        false
    }
    /// The scene as bytes.
    fn export(&self, scene: &Scene) -> Result<Vec<u8>>;
    /// A board from bytes. `actor` writes the first clocks.
    fn import(&self, _data: &[u8], _actor: &str) -> Result<Imported> {
        not_possible("board.import")
    }
}
