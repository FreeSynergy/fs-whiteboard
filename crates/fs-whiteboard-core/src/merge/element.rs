//! Merge of two element documents.
//!
//! Pattern: Strategy — for each unit the larger `(counter, actor)` wins; equal clocks fall back to
//! the value text. The identity (id, board, kind) never changes after `add`; if two documents
//! disagree on it (corrupt input), the larger identity text wins as a whole, so the result stays
//! a pure function of the two.

use serde_json::json;

use crate::clock::wins;
use crate::doc::ElementDoc;
use crate::unit::Unit;

fn identity(d: &ElementDoc) -> String {
    json!([d.element.id, d.element.board, d.element.kind]).to_string()
}

/// The merge of `local` and `remote`.
pub fn merge_element(local: &ElementDoc, remote: &ElementDoc) -> ElementDoc {
    let (a, b) = (identity(local), identity(remote));
    if a != b {
        return if b > a { remote.clone() } else { local.clone() };
    }
    let mut out = local.clone();
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
    out
}
