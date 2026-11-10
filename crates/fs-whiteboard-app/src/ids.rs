//! Identities: the IRI a record is known by, and back.
//!
//! Pattern: Value object functions — a reference in the admin frontend is an absolute IRI (`reference` basic type), the
//! core's ids are UUIDs; `https://<instance>/whiteboard/<id>` is the form the core gives a board (ADR 0027), and pages
//! and elements get the same form.

use fs_objects::ObjectId;

/// The IRI of record `id` on `instance`.
pub fn iri(instance: &str, id: ObjectId) -> String {
    format!("https://{instance}/whiteboard/{id}")
}

/// The id in an IRI made by [`iri`] (the last path segment); `None` for anything else.
pub fn parse_iri(text: &str) -> Option<ObjectId> {
    let rest = text.trim().strip_prefix("https://")?;
    let (_, path) = rest.split_once("/whiteboard/")?;
    path.parse().ok()
}
