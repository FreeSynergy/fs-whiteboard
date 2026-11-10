//! The fields of the three types.
//!
//! Pattern: Builder (of `fs-types`) over a plain list — one function per type; a command that edits a record reuses the
//! field names of the type, so a generated form fills them from the record.

use fs_types::{BasicType, FieldDescription};

use super::scope::{BOARD, ELEMENT, PAGE, kebab};

/// Keys of the formats a board is imported from.
pub const BOARD_FORMATS: [&str; 1] = ["whiteboard"];
/// Keys of the formats a board or page is exported to that the core can write.
pub const EXPORT_FORMATS: [&str; 2] = ["whiteboard", "svg"];
/// The kinds of element (the contract's union).
pub const KINDS: [&str; 8] = [
    "shape",
    "text",
    "note",
    "stroke",
    "connector",
    "image",
    "group",
    "area",
];
/// The shapes.
pub const SHAPES: [&str; 4] = ["rectangle", "ellipse", "diamond", "line"];

fn f(ty: &str, name: &str, basic: BasicType) -> FieldDescription {
    FieldDescription::new(name, basic).label(&format!("{ty}.{}", kebab(name)))
}

fn flag(ty: &str, name: &str) -> FieldDescription {
    f(ty, name, BasicType::Choice).choices(&["yes", "no"])
}

fn num(ty: &str, name: &str) -> FieldDescription {
    f(ty, name, BasicType::Text)
}

/// Fields of a board: the AS2 `Collection` it is stored as.
pub fn board() -> Vec<FieldDescription> {
    let t = BOARD.label;
    vec![
        f(t, "name", BasicType::Text).required(),
        f(t, "pages", BasicType::Reference)
            .target(PAGE.name)
            .multiple(),
    ]
}

/// Fields of a page.
pub fn page() -> Vec<FieldDescription> {
    let t = PAGE.label;
    vec![
        f(t, "board", BasicType::Reference)
            .target(BOARD.name)
            .required(),
        f(t, "name", BasicType::Text),
        f(t, "order", BasicType::Text),
        flag(t, "deleted"),
    ]
}

/// Fields of an element. Numbers are text and yes/no is a choice until the basic types have them.
pub fn element() -> Vec<FieldDescription> {
    let t = ELEMENT.label;
    let r = |n: &str, to: &str| f(t, n, BasicType::Reference).target(to);
    vec![
        r("board", BOARD.name).required(),
        r("page", PAGE.name).required(),
        r("parent", ELEMENT.name),
        f(t, "kind", BasicType::Choice).choices(&KINDS).required(),
        f(t, "shape", BasicType::Choice).choices(&SHAPES),
        f(t, "text", BasicType::MultilineText),
        f(t, "label", BasicType::MultilineText),
        num(t, "x").required(),
        num(t, "y").required(),
        num(t, "width").required(),
        num(t, "height").required(),
        num(t, "angle"),
        f(t, "stroke", BasicType::Text),
        f(t, "fill", BasicType::Text),
        num(t, "lineWidth"),
        flag(t, "dashed"),
        num(t, "opacity"),
        f(t, "fontFamily", BasicType::Text),
        num(t, "fontSize"),
        flag(t, "locked"),
        flag(t, "deleted"),
        f(t, "image", BasicType::Link),
        f(t, "points", BasicType::MultilineText),
        r("source", ELEMENT.name),
        r("target", ELEMENT.name),
        num(t, "sourceX"),
        num(t, "sourceY"),
        num(t, "targetX"),
        num(t, "targetY"),
        flag(t, "arrowEnd"),
        f(t, "order", BasicType::Text),
    ]
}
