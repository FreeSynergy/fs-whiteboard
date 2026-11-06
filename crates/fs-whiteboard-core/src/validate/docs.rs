//! Checks of whole documents as received from outside (`apply_remote`, import).
//!
//! Pattern: Specification — an element or a board from outside is checked like a local entry, plus
//! a valid `order` and clock counters that leave room to count on.

use super::{element, order_key, text};
use crate::error::{Error, Result};
use crate::model::Element;

/// The largest clock counter accepted from outside, so the next local counter cannot run out.
pub const MAX_COUNTER: u64 = u64::MAX / 2;

/// A clock counter from outside.
pub fn counter(c: u64) -> Result<()> {
    if c > MAX_COUNTER {
        return Err(Error::invalid("a clock counter is out of range"));
    }
    Ok(())
}

/// A page as received from outside.
pub fn page(p: &crate::model::Page) -> Result<()> {
    text(&p.name)?;
    order_key(&p.order)
}

/// An element as received from outside: everything `element` checks, and a valid `order`.
pub fn remote_element(e: &Element) -> Result<()> {
    order_key(&e.order)?;
    element(e)
}

/// An element document as received from outside: the element, and clocks within range.
pub fn element_doc(d: &crate::doc::ElementDoc) -> Result<()> {
    remote_element(&d.element)?;
    d.clocks.values().try_for_each(|c| counter(c.counter()))
}

/// A board document as received from outside: a name, at least one page, valid pages and clocks.
pub fn board_doc(d: &crate::doc::BoardDoc) -> Result<()> {
    if d.name.trim().is_empty() {
        return Err(Error::invalid("a board needs a name"));
    }
    text(&d.name)?;
    if d.pages.is_empty() {
        return Err(Error::invalid("a board needs a page"));
    }
    if d.pages.iter().any(|(k, p)| *k != p.id) {
        return Err(Error::invalid("a page is listed under another id"));
    }
    d.pages.values().try_for_each(page)?;
    d.clocks.values().try_for_each(|c| counter(c.counter()))
}
