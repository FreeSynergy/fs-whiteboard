//! The store-backed whiteboard program: module map.
//!
//! Pattern: Facade (declarations) — `WhiteboardRepo` is the one entry point; each file is one concept
//! (state, store access, transactions, history, boards, remote changes, feed, the editor).

mod boards;
mod editor;
mod ends;
mod feed;
mod handle;
mod history;
mod io;
mod remote;
mod state;
mod support;
mod tx;

pub use handle::WhiteboardRepo;
