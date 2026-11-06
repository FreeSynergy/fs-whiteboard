//! Paging: an opaque cursor and one page of results.
//!
//! Pattern: Iterator (cursor paging) — the caller hands back `next` until it is `None`. The cursor is
//! opaque to callers; this core encodes the last position seen, so edits between pages lose or
//! repeat nothing.

/// Opaque position in a list (`Cursor` in `common.capnp`; none = start).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor(pub String);

/// One page of results; `next` is `None` on the last page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paged<T> {
    /// The items of this page.
    pub items: Vec<T>,
    /// Where the next page starts.
    pub next: Option<Cursor>,
}
