//! A board and its pages as the reader sees them.
//!
//! Pattern: Composite (read model) — a board is a name and an ordered list of pages.

use fs_objects::ObjectId;
use serde::{Deserialize, Serialize};

/// A page of a board. `deleted` pages stay as markers, so a removal merges.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    /// Id of the page.
    pub id: ObjectId,
    /// Display name.
    pub name: String,
    /// Position key (see `order`).
    pub order: String,
    /// Removed.
    pub deleted: bool,
}

/// A board: AS2 `Collection` with its pages in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    /// Id of the board object.
    pub id: ObjectId,
    /// External IRI.
    pub iri: String,
    /// Display name.
    pub name: String,
    /// The AS2 `Collection` as JSON text.
    pub as2: String,
    /// Pages in order; removed ones are marked, and the page kept by the "highest id stays" rule is
    /// reported as not removed. At least one is not removed.
    pub pages: Vec<Page>,
}
