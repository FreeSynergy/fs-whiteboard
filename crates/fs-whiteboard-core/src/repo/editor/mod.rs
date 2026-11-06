//! The editor of one board: module map.
//!
//! Pattern: Facade (declarations) — `Editor` implements `BoardEditor`; each file is one group of
//! edits, and every edit is a list of unit changes that the history can take back.

mod add;
mod copy;
mod export;
mod handle;
mod pages;
mod props;
mod provide;
mod read;
mod reorder;
mod selection;
mod transform;
mod tree;
mod undo;

pub(super) use handle::Editor;
