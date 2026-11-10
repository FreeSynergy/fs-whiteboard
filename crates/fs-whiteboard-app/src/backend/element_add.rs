//! `add`: a new element of any kind from typed values.
//!
//! Pattern: Builder (a function) — reads the fields of the `add` command into an `Element`, the kind picks the
//! data that belongs to it. A group is not a choice here (`group` makes one); the core owns the rest of the checks.

use fs_admin_frontend::{BackendError, Outcome, Values};
use fs_types::{RecordIssue, RecordProblem};
use fs_whiteboard_core::{Anchor, Connector, Element, Kind, Rect, ShapeKind, Style};

use crate::convert::{
    choice, flag, id, need_choice, need_id, need_num, num, num32, one, point, points,
};
use crate::core::Core;

const KINDS: [&str; 7] = [
    "shape",
    "text",
    "note",
    "stroke",
    "connector",
    "image",
    "area",
];
const SHAPES: [&str; 4] = ["rectangle", "ellipse", "diamond", "line"];

fn shape(key: &str) -> ShapeKind {
    match key {
        "ellipse" => ShapeKind::Ellipse,
        "diamond" => ShapeKind::Diamond,
        "line" => ShapeKind::Line,
        _ => ShapeKind::Rectangle,
    }
}

fn kind(input: &Values) -> Result<Kind, BackendError> {
    let text = || one(input, "text").unwrap_or("").to_owned();
    match need_choice(input, "kind", &KINDS)? {
        "shape" => Ok(Kind::Shape {
            shape: shape(choice(input, "shape", &SHAPES)?.unwrap_or("rectangle")),
        }),
        "text" => Ok(Kind::Text { text: text() }),
        "note" => Ok(Kind::Note { text: text() }),
        "area" => Ok(Kind::Area { text: text() }),
        "stroke" => Ok(Kind::Stroke {
            points: points(input, "points")?,
        }),
        "image" => {
            let iri = one(input, "image").ok_or_else(|| missing("image"))?;
            Ok(Kind::Image {
                iri: iri.to_owned(),
            })
        }
        _ => Ok(Kind::Connector(Connector {
            source: Anchor {
                element: id(input, "source")?,
                point: point(input, "sourceX", "sourceY")?,
            },
            target: Anchor {
                element: id(input, "target")?,
                point: point(input, "targetX", "targetY")?,
            },
            label: text(),
            arrow_end: flag(input, "arrowEnd")?.unwrap_or(false),
        })),
    }
}

fn missing(field: &str) -> BackendError {
    BackendError::Rejected(vec![RecordIssue {
        field: field.to_owned(),
        index: 0,
        problem: RecordProblem::Missing,
    }])
}

/// Style fields that were given; the others keep the default.
pub fn style(input: &Values) -> Result<Style, BackendError> {
    let mut s = Style::default();
    if let Some(t) = one(input, "stroke") {
        t.clone_into(&mut s.stroke);
    }
    if let Some(t) = one(input, "fill") {
        t.clone_into(&mut s.fill);
    }
    if let Some(t) = one(input, "fontFamily") {
        t.clone_into(&mut s.font_family);
    }
    s.line_width = num(input, "lineWidth")?.unwrap_or(s.line_width);
    s.dashed = flag(input, "dashed")?.unwrap_or(s.dashed);
    s.opacity = num32(input, "opacity")?.unwrap_or(s.opacity);
    s.font_size = num32(input, "fontSize")?.unwrap_or(s.font_size);
    Ok(s)
}

/// `add`.
pub fn add(core: &Core, input: &Values) -> Result<Outcome, BackendError> {
    let page = need_id(input, "page")?;
    let board = core.board_of_page(page)?;
    let frame = Rect::new(
        need_num(input, "x")?,
        need_num(input, "y")?,
        need_num(input, "width")?,
        need_num(input, "height")?,
    );
    let mut element = Element::new(page, kind(input)?, frame);
    element.parent = id(input, "parent")?;
    element.angle = num(input, "angle")?.unwrap_or(0.0);
    element.style = style(input)?;
    if let Some(label) = one(input, "label") {
        label.clone_into(&mut element.label);
    }
    let editor = core.editor(board.id)?;
    let added = core.run(editor.add(element))?;
    Ok(Outcome {
        id: Some(core.iri(added.id)),
        ..Outcome::default()
    })
}
