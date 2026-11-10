//! Core values to the records of the description.
//!
//! Pattern: Adapter (functions) — one function per type; the field names are those of `fs_whiteboard_core::describe`.

use fs_admin_frontend::{Record, Values};
use fs_whiteboard_core::{Board, Element, Kind, Page, ShapeKind};

use crate::convert::{show, show_flag, show_points};
use crate::core::Core;

fn put(v: &mut Values, name: &str, text: impl Into<String>) {
    let text = text.into();
    if !text.is_empty() {
        v.insert(name.to_owned(), vec![text]);
    }
}

/// A board as a record.
pub fn board(core: &Core, b: &Board) -> Record {
    let mut v = Values::new();
    put(&mut v, "name", b.name.clone());
    let pages: Vec<String> = b
        .pages
        .iter()
        .filter(|p| !p.deleted)
        .map(|p| core.iri(p.id))
        .collect();
    if !pages.is_empty() {
        v.insert("pages".into(), pages);
    }
    Record {
        id: b.iri.clone(),
        values: v,
    }
}

/// A page as a record.
pub fn page(core: &Core, board: &Board, p: &Page) -> Record {
    let mut v = Values::new();
    put(&mut v, "board", board.iri.clone());
    put(&mut v, "name", p.name.clone());
    put(&mut v, "order", p.order.clone());
    put(&mut v, "deleted", show_flag(p.deleted));
    Record {
        id: core.iri(p.id),
        values: v,
    }
}

/// An element as a record.
pub fn element(core: &Core, el: &Element) -> Record {
    let mut v = Values::new();
    put(&mut v, "board", core.iri(el.board));
    put(&mut v, "page", core.iri(el.page));
    if let Some(p) = el.parent {
        put(&mut v, "parent", core.iri(p));
    }
    put(&mut v, "kind", el.kind.name());
    put(&mut v, "label", el.label.clone());
    for (name, n) in [
        ("x", el.frame.x),
        ("y", el.frame.y),
        ("width", el.frame.width),
        ("height", el.frame.height),
        ("angle", el.angle),
        ("lineWidth", el.style.line_width),
        ("opacity", f64::from(el.style.opacity)),
        ("fontSize", f64::from(el.style.font_size)),
    ] {
        put(&mut v, name, show(n));
    }
    put(&mut v, "stroke", el.style.stroke.clone());
    put(&mut v, "fill", el.style.fill.clone());
    put(&mut v, "fontFamily", el.style.font_family.clone());
    put(&mut v, "dashed", show_flag(el.style.dashed));
    put(&mut v, "locked", show_flag(el.locked));
    put(&mut v, "deleted", show_flag(el.deleted));
    put(&mut v, "order", el.order.clone());
    kind_values(core, el, &mut v);
    Record {
        id: core.iri(el.id),
        values: v,
    }
}

fn kind_values(core: &Core, el: &Element, v: &mut Values) {
    match &el.kind {
        Kind::Shape { shape } => {
            put(v, "shape", shape_key(*shape));
        }
        Kind::Text { text } | Kind::Note { text } | Kind::Area { text } => {
            put(v, "text", text.clone());
        }
        Kind::Stroke { points } => put(v, "points", show_points(points)),
        Kind::Image { iri } => put(v, "image", iri.clone()),
        Kind::Connector(c) => {
            put(v, "text", c.label.clone());
            put(v, "arrowEnd", show_flag(c.arrow_end));
            if let Some(s) = c.source.element {
                put(v, "source", core.iri(s));
            }
            if let Some(t) = c.target.element {
                put(v, "target", core.iri(t));
            }
            put(v, "sourceX", show(c.source.point.x));
            put(v, "sourceY", show(c.source.point.y));
            put(v, "targetX", show(c.target.point.x));
            put(v, "targetY", show(c.target.point.y));
        }
        Kind::Group => {}
    }
}

/// The choice key of a shape.
pub fn shape_key(shape: ShapeKind) -> &'static str {
    match shape {
        ShapeKind::Rectangle => "rectangle",
        ShapeKind::Ellipse => "ellipse",
        ShapeKind::Diamond => "diamond",
        ShapeKind::Line => "line",
    }
}
