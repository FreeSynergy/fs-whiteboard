//! Merge of two board documents.
//!
//! Pattern: Strategy — pages are a grow-only set by id; `name`, `order` and `deleted` of each page
//! and the board's `name` are registers with clocks, the larger `(counter, actor)` wins.

use crate::clock::wins;
use crate::doc::BoardDoc;

/// The merge of `local` and `remote`.
pub fn merge_board(local: &BoardDoc, remote: &BoardDoc) -> BoardDoc {
    let (mut out, mut remote) = (local.clone(), remote.clone());
    out.sanitize();
    remote.sanitize();
    let remote = &remote;
    let (lc, rc) = (out.clock("name"), remote.clock("name"));
    if wins(&lc, &out.name, &rc, &remote.name) {
        out.name.clone_from(&remote.name);
        out.clocks.insert("name".into(), rc);
    }
    for (id, rp) in &remote.pages {
        let Some(lp) = out.pages.get(id).cloned() else {
            out.pages.insert(*id, rp.clone());
            for part in ["name", "order", "deleted"] {
                out.clocks.insert(
                    format!("{id}/{part}"),
                    remote.clock(&format!("{id}/{part}")),
                );
            }
            continue;
        };
        let mut page = lp.clone();
        for part in ["name", "order", "deleted"] {
            let key = format!("{id}/{part}");
            let (lc, rc) = (out.clock(&key), remote.clock(&key));
            let (lv, rv) = (
                BoardDoc::page_value(&lp, part),
                BoardDoc::page_value(rp, part),
            );
            if wins(&lc, &lv, &rc, &rv) {
                match part {
                    "name" => page.name.clone_from(&rp.name),
                    "order" => page.order.clone_from(&rp.order),
                    _ => page.deleted = rp.deleted,
                }
                out.clocks.insert(key, rc);
            }
        }
        out.pages.insert(*id, page);
    }
    out
}
