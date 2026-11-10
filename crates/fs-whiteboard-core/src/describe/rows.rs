//! The rows the core can do today (`supported: true`).
//!
//! Pattern: Registry (rows of the table in `table.rs`); one function per type. Commands on one record have an `id`
//! reference to the own type (strategy decision R6); commands on several records (`group`, `align`) take `ids`.

use fs_types::BasicType::{Choice, MultilineText, Text};
use fs_types::CommandKind::{Change, Create, Delete, Find, Other};

use super::fields::{self, BOARD_FORMATS, EXPORT_FORMATS};
use super::scope::{BOARD, Cmd, ELEMENT, PAGE};
use super::table::Row;

/// All supported rows.
pub fn supported() -> Vec<Row> {
    let mut rows = board();
    rows.extend(page());
    rows.extend(element());
    rows.extend(element_structure());
    rows
}

fn done(c: &Cmd) -> Vec<fs_types::FieldDescription> {
    vec![c.flag("done")]
}

fn board() -> Vec<Row> {
    let b = BOARD;
    vec![
        // The list is `find` with empty input (strategy decision R2).
        Row::yes(b, "find", Find, &["board.open"], |_| (vec![], vec![])),
        Row::yes(b, "create", Create, &["board.create"], |c| {
            (
                vec![c.f("name", Text).required()],
                vec![c.r#ref("id", b).required()],
            )
        }),
        Row::yes(b, "change", Change, &["board.create"], |_| (vec![], vec![])),
        Row::yes(b, "delete", Delete, &["board.delete"], |c| {
            (vec![c.id()], vec![])
        }),
        Row::yes(b, "import", Create, &["board.import"], |c| {
            (
                vec![
                    c.f("format", Choice).choices(&BOARD_FORMATS).required(),
                    c.f("data", MultilineText).required(),
                ],
                vec![c.r#ref("id", b).required()],
            )
        }),
        Row::yes(b, "export", Other, &["board.export", "board.save"], |c| {
            (
                vec![
                    c.id(),
                    c.r#ref("page", PAGE),
                    c.f("format", Choice).choices(&EXPORT_FORMATS).required(),
                ],
                vec![c.f("data", MultilineText).required()],
            )
        }),
        Row::yes(b, "undo", Other, &["history.undo"], |c| {
            (vec![c.id()], done(c))
        }),
        Row::yes(b, "redo", Other, &["history.redo"], |c| {
            (vec![c.id()], done(c))
        }),
    ]
}

fn page() -> Vec<Row> {
    let p = PAGE;
    vec![
        // filter beyond `term`: the pages of one board
        Row::yes(p, "find", Find, &["board.open"], |c| {
            (vec![c.r#ref("board", BOARD)], vec![])
        }),
        Row::yes(p, "add", Create, &["page.add"], |c| {
            (
                vec![c.r#ref("board", BOARD).required(), c.f("name", Text)],
                vec![c.r#ref("id", p).required()],
            )
        }),
        Row::yes(p, "remove", Delete, &["page.remove"], |c| {
            (vec![c.id()], vec![])
        }),
        Row::yes(p, "move", Other, &["page.add"], |c| {
            (vec![c.id(), c.r#ref("before", p)], vec![])
        }),
        Row::yes(p, "bounds", Other, &["canvas.navigate"], |c| {
            (
                vec![c.id()],
                vec![c.num("x"), c.num("y"), c.num("width"), c.num("height")],
            )
        }),
    ]
}

/// The style fields every element can carry.
fn styles(c: &Cmd) -> Vec<fs_types::FieldDescription> {
    vec![
        c.f("stroke", Text),
        c.f("fill", Text),
        c.num("lineWidth"),
        c.flag("dashed"),
        c.num("opacity"),
        c.f("fontFamily", Text),
        c.num("fontSize"),
    ]
}

/// `add`: one command for every kind of element (the kind is a choice).
fn element_add() -> Row {
    let e = ELEMENT;
    Row::yes(
        e,
        "add",
        Create,
        &[
            "shape.add",
            "text.add",
            "note.add",
            "stroke.draw",
            "connector.add",
            "image.add",
            "frame.add",
        ],
        |c| {
            let mut input = vec![
                c.r#ref("page", PAGE).required(),
                c.r#ref("parent", e),
                c.f("kind", Choice).choices(&fields::KINDS).required(),
                c.f("shape", Choice).choices(&fields::SHAPES),
                c.f("text", MultilineText),
                c.f("label", MultilineText),
                c.num("x").required(),
                c.num("y").required(),
                c.num("width").required(),
                c.num("height").required(),
                c.num("angle"),
                c.f("image", fs_types::BasicType::Link),
                c.f("points", MultilineText),
                c.r#ref("source", e),
                c.r#ref("target", e),
                c.num("sourceX"),
                c.num("sourceY"),
                c.num("targetX"),
                c.num("targetY"),
                c.flag("arrowEnd"),
            ];
            input.extend(styles(c));
            (input, vec![c.r#ref("id", e).required()])
        },
    )
}

fn element() -> Vec<Row> {
    let e = ELEMENT;
    vec![
        // filter beyond `term`: the elements of one page
        Row::yes(e, "find", Find, &["board.view"], |c| {
            (vec![c.r#ref("page", PAGE)], vec![])
        }),
        // `change` edits the fields of the type (an `Edit`, no input list): frame, angle, style, text, label, ends, lock,
        // parent; each path maps to one method of `BoardEditor`.
        Row::yes(
            e,
            "change",
            Change,
            &[
                "element.move",
                "element.style",
                "text.edit",
                "connector.edit",
                "element.lock",
                "element.group",
            ],
            |_| (vec![], vec![]),
        ),
        element_add(),
        Row::yes(e, "move", Other, &["element.move"], |c| {
            (
                vec![c.id(), c.num("dx").required(), c.num("dy").required()],
                vec![],
            )
        }),
        Row::yes(e, "copy", Other, &["element.move"], |c| {
            (
                vec![c.id(), c.num("dx").required(), c.num("dy").required()],
                vec![c.r#ref("copy", e).required()],
            )
        }),
        Row::yes(
            e,
            "delete",
            Delete,
            &["element.delete", "stroke.erase"],
            |c| (vec![c.id()], vec![]),
        ),
    ]
}

/// Grouping, order and alignment.
fn element_structure() -> Vec<Row> {
    let e = ELEMENT;
    vec![
        Row::yes(e, "group", Other, &["element.group"], |c| {
            (
                vec![c.r#ref("ids", e).required().multiple()],
                vec![c.r#ref("group", e).required()],
            )
        }),
        Row::yes(e, "ungroup", Other, &["element.ungroup"], |c| {
            (vec![c.id()], vec![c.r#ref("members", e).multiple()])
        }),
        Row::yes(e, "reorder", Other, &["element.reorder"], |c| {
            (vec![c.id(), c.r#ref("before", e)], vec![])
        }),
        Row::yes(e, "align", Other, &["element.align"], |c| {
            (
                vec![
                    c.r#ref("ids", e).required().multiple(),
                    c.f("how", Choice)
                        .choices(&[
                            "left",
                            "center-x",
                            "right",
                            "top",
                            "center-y",
                            "bottom",
                            "distribute-x",
                            "distribute-y",
                        ])
                        .required(),
                ],
                vec![],
            )
        }),
    ]
}
