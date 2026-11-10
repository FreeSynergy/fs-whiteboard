//! Which pages count as removed.
//!
//! Pattern: Strategy (conflict rule) — a removed page hides its elements; if concurrent removals
//! leave no page, the page with the highest id stays.

use crate::model::Page;

/// The pages in order (by `order` as bytes, then id), with `deleted` as it counts at reading: the
/// highest-id page is kept when all are removed. Empty only if there are no pages at all.
pub fn effective_pages(pages: &[Page]) -> Vec<Page> {
    let mut out = pages.to_vec();
    out.sort_by(|a, b| (a.order.as_bytes(), a.id).cmp(&(b.order.as_bytes(), b.id)));
    if out.iter().all(|p| p.deleted)
        && let Some(keep) = out.iter_mut().max_by_key(|p| p.id)
    {
        keep.deleted = false;
    }
    out
}
