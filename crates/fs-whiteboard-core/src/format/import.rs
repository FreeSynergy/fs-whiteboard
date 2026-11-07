//! Import of the own format: checked first, then given new ids.
//!
//! Pattern: Template method (read -> check -> rewrite) — nothing is returned unless every check
//! passed: limits, finite numbers, valid order keys, and every reference (page, parent, connector
//! end) pointing at something in the file, with no cycle of parents.

use std::collections::{BTreeMap, BTreeSet};

use fs_objects::ObjectId;

use super::Imported;
use super::json::{Document, JSON_MEDIA_TYPE, VERSION};
use crate::doc::{BoardDoc, ElementDoc};
use crate::error::{Error, Result};
use crate::model::{Anchor, Kind};
use crate::order;
use crate::reading::effective_parents;
use crate::validate;

fn check(doc: &Document) -> Result<()> {
    if doc.format != JSON_MEDIA_TYPE || doc.version != VERSION {
        return Err(Error::invalid("not version 1 of the whiteboard format"));
    }
    if doc.name.trim().is_empty() || doc.pages.iter().all(|p| p.deleted) {
        return Err(Error::invalid(
            "a board needs a name and a page that is not removed",
        ));
    }
    let pages: BTreeSet<ObjectId> = doc.pages.iter().map(|p| p.id).collect();
    let ids: BTreeSet<ObjectId> = doc.elements.iter().map(|e| e.id).collect();
    if pages.len() != doc.pages.len() || ids.len() != doc.elements.len() {
        return Err(Error::invalid("an id is used twice"));
    }
    for p in &doc.pages {
        validate::text(&p.name)?;
        if !order::is_valid(&p.order) {
            return Err(Error::invalid("a page has no valid order key"));
        }
    }
    for e in &doc.elements {
        validate::element(e)?;
        if !order::is_valid(&e.order) {
            return Err(Error::invalid("an element has no valid order key"));
        }
        let ends = match &e.kind {
            Kind::Connector(c) => [c.source.element, c.target.element],
            _ => [None, None],
        };
        let refs = e.parent.into_iter().chain(ends.into_iter().flatten());
        if !pages.contains(&e.page) || refs.into_iter().any(|r| !ids.contains(&r)) {
            return Err(Error::invalid(
                "a reference points at something that is not in the file",
            ));
        }
    }
    let by_id: BTreeMap<ObjectId, _> = doc.elements.iter().map(|e| (e.id, e.clone())).collect();
    if effective_parents(&by_id)
        .iter()
        .any(|(id, p)| *p != by_id[id].parent)
    {
        return Err(Error::invalid("the parents form a cycle"));
    }
    Ok(())
}

pub(super) fn read(data: &[u8], actor: &str) -> Result<Imported> {
    if data.len() > validate::MAX_IMPORT_BYTES {
        return Err(Error::TooLarge(format!(
            "import of {} bytes, at most {}",
            data.len(),
            validate::MAX_IMPORT_BYTES
        )));
    }
    let doc: Document = serde_json::from_slice(data)?;
    check(&doc)?;
    let pages: BTreeMap<ObjectId, ObjectId> =
        doc.pages.iter().map(|p| (p.id, ObjectId::new())).collect();
    let ids: BTreeMap<ObjectId, ObjectId> = doc
        .elements
        .iter()
        .map(|e| (e.id, ObjectId::new()))
        .collect();
    let mut pages_iter = doc.pages.iter().map(|p| {
        let mut p = p.clone();
        p.id = pages[&p.id];
        p
    });
    let first = pages_iter.next().ok_or_else(|| Error::invalid("no page"))?;
    let mut board = BoardDoc::fresh(&doc.name, first, actor);
    for p in pages_iter {
        board.add_page(p, actor);
    }
    let anchor = |a: &mut Anchor| a.element = a.element.map(|id| ids[&id]);
    let elements = doc
        .elements
        .into_iter()
        .map(|mut e| {
            (e.id, e.page, e.parent) = (ids[&e.id], pages[&e.page], e.parent.map(|p| ids[&p]));
            if let Kind::Connector(c) = &mut e.kind {
                anchor(&mut c.source);
                anchor(&mut c.target);
            }
            ElementDoc::fresh(e, actor)
        })
        .collect();
    Ok(Imported { board, elements })
}
