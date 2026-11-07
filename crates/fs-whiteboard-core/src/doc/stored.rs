//! Documents to store objects and back.
//!
//! Pattern: Adapter — `ElementDoc` and `BoardDoc` are written as the content of a `StoredObject`
//! with the index fields the queries use; the envelope is rebuilt from the document, never taken
//! from a remote object, so no wall time can enter a merge.

use fs_objects::{Envelope, ObjectId, StoredObject};

use super::{BoardDoc, ElementDoc};
use crate::consts::{BOARD_CONTENT, BOARD_CONTEXT, BOARD_TYPE, ELEMENT_CONTENT, ELEMENT_TYPE};
use crate::error::Result;

/// IRI of an object of this program: `https://<instance>/whiteboard/<id>`.
pub(crate) fn iri(instance: &str, id: ObjectId) -> String {
    format!("https://{instance}/whiteboard/{id}")
}

fn envelope(existing: Option<&StoredObject>, id: ObjectId, kind: &str, name: String) -> Envelope {
    let mut e = existing.map_or_else(|| Envelope::new(kind, name.clone()), |o| o.envelope.clone());
    e.id = id;
    e.name = name;
    e
}

fn name_of(doc: &ElementDoc) -> String {
    let text: String = doc
        .element
        .text()
        .unwrap_or(&doc.element.label)
        .chars()
        .take(80)
        .collect();
    format!("{} {}", doc.element.kind.name(), text.replace('\n', " "))
        .trim()
        .to_string()
}

/// The stored form of an element; `existing` keeps `published` and the like.
pub(crate) fn element_to_stored(
    doc: &ElementDoc,
    instance: &str,
    existing: Option<&StoredObject>,
) -> Result<StoredObject> {
    let el = &doc.element;
    let mut env = envelope(existing, el.id, ELEMENT_TYPE, name_of(doc));
    env.iri = Some(iri(instance, el.id));
    env.context = Some(iri(instance, el.board));
    let mut o = StoredObject::new(env, ELEMENT_CONTENT, serde_json::to_value(doc)?)
        .with_index("board", el.board.to_string())
        .with_index("page", el.page.to_string())
        .with_index("kind", el.kind.name());
    if let Some(p) = el.parent {
        o = o.with_index("parent", p.to_string());
    }
    Ok(o)
}

/// The element document in a stored object.
pub(crate) fn stored_to_element(o: &StoredObject) -> Result<ElementDoc> {
    let mut d: ElementDoc = serde_json::from_value(o.content.clone())?;
    d.sanitize();
    Ok(d)
}

/// The stored form of a board.
pub(crate) fn board_to_stored(
    doc: &BoardDoc,
    id: ObjectId,
    instance: &str,
    actor: &str,
    existing: Option<&StoredObject>,
) -> Result<StoredObject> {
    let mut env = envelope(existing, id, BOARD_TYPE, doc.name.clone());
    env.iri = Some(iri(instance, id));
    env.context = Some(BOARD_CONTEXT.into());
    env.attributed_to.get_or_insert_with(|| actor.to_string());
    Ok(StoredObject::new(
        env,
        BOARD_CONTENT,
        serde_json::to_value(doc)?,
    ))
}

/// The board document in a stored object.
pub(crate) fn stored_to_board(o: &StoredObject) -> Result<BoardDoc> {
    let mut d: BoardDoc = serde_json::from_value(o.content.clone())?;
    d.sanitize();
    Ok(d)
}
