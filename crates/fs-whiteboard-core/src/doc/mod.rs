//! The stored documents: an element or a board together with the clocks of its units.
//!
//! Pattern: Facade (declarations) — `ElementDoc` and `BoardDoc` are what `merge` works on and what
//! is written to the store under `fs:clocks`.

mod board;
mod element;
mod stored;

pub use board::BoardDoc;
pub use element::ElementDoc;
pub(crate) use stored::{
    board_to_stored, element_to_stored, iri, stored_to_board, stored_to_element,
};
