//! Merge of two element documents.
//!
//! Pattern: Strategy — for each unit the larger `(counter, actor)` wins; equal clocks fall back to
//! the value text. The identity (id, board, kind) never changes after `add`; two documents that
//! disagree on it are refused.

use serde_json::json;

use crate::clock::wins;
use crate::doc::ElementDoc;
use crate::error::{Error, Result};
use crate::model::Kind;
use crate::unit::Unit;

/// What never changes after `add`: ids, the kind, a stroke's points, an image's IRI, a connector's arrow.
fn identity(d: &ElementDoc) -> String {
    let e = &d.element;
    let fixed = match &e.kind {
        Kind::Shape { shape } => json!(shape),
        Kind::Stroke { points } => json!(points),
        Kind::Image { iri } => json!(iri),
        Kind::Connector(c) => json!(c.arrow_end),
        _ => json!(null),
    };
    json!([e.id, e.board, e.kind.name(), fixed]).to_string()
}

/// The merge of `local` and `remote`; refused if they are not versions of the same element.
pub fn merge_element(local: &ElementDoc, remote: &ElementDoc) -> Result<ElementDoc> {
    if identity(local) != identity(remote) {
        return Err(Error::invalid("not versions of the same element"));
    }
    let (mut remote, mut out) = (remote.clone(), local.clone());
    remote.sanitize();
    out.sanitize();
    for unit in Unit::ALL {
        let Some(rv) = unit.read(&remote.element) else {
            continue;
        };
        let lv = unit
            .read(&out.element)
            .map(|v| v.to_string())
            .unwrap_or_default();
        let (lc, rc) = (out.clock(unit), remote.clock(unit));
        if wins(&lc, &lv, &rc, &rv.to_string()) {
            // A value read from a document of the same kind always fits back in.
            if unit.write(&mut out.element, rv).is_ok() {
                out.clocks.insert(unit, rc);
            }
        }
    }
    out.sanitize();
    Ok(out)
}
