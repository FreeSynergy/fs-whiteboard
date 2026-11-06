//! The rules for reading a board: what is hidden, which page stays, how a cycle is broken. Pure.
//!
//! Pattern: Facade (declarations) — functions over the loaded elements; every replica that holds
//! the same elements computes the same answer, with no coordination.

mod connector;
mod pages;
mod parents;
mod view;

pub use connector::End;
pub use pages::effective_pages;
pub use parents::{depths, effective_parents};
pub use view::View;
