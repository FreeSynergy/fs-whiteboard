//! The rows the core cannot do (`supported: false`), each with its reason.
//!
//! Pattern: Registry (rows of the table in `table.rs`); Null Object on the wire — the command is described so a frontend
//! shows it locked with "coming", and the backend refuses it with `unsupported:<name>`. The reasons are the same as
//! the core's own `NOT_BUILT` list (`repo/support.rs`) where it has one.

use fs_types::BasicType::{Choice, Link, MultilineText, Reference, Text};
use fs_types::CommandKind::{Change, Check, Create, Find, Other};

use super::scope::{BOARD, Cmd, ELEMENT, PAGE};
use super::table::Row;

const NO_CONTRACT: &str = "not in the contract yet";
const PRESENCE: &str = "not in the contract yet; passing data on a channel of its own, never stored (no owner for the shared Presence interface)";
const LATER: &str = "optional capability of the requirements, no design yet";

fn board_other(name: &'static str, caps: &'static [&'static str], why: &'static str) -> Row {
    Row::no(BOARD, name, Other, caps, why, |c| (vec![c.id()], vec![]))
}

/// All unsupported rows.
pub fn unsupported() -> Vec<Row> {
    let mut rows = board();
    rows.extend(board_content());
    rows.extend(board_sessions());
    rows.extend(page());
    rows.extend(element());
    rows
}

fn board() -> Vec<Row> {
    let b = BOARD;
    vec![
        Row::no(
            b,
            "exportPng",
            Other,
            &["board.export"],
            "PNG export is Unsupported in the core",
            |c| (vec![c.id()], vec![c.f("data", Text)]),
        ),
        Row::no(
            b,
            "exportPdf",
            Other,
            &["board.export"],
            "PDF export is Unsupported in the core",
            |c| (vec![c.id()], vec![c.f("data", Text)]),
        ),
        Row::no(
            b,
            "subscribe",
            Other,
            &["board.subscribe"],
            "the core offers a change stream; the admin backend answers one request at a time",
            |c| (vec![c.id()], vec![]),
        ),
        board_other(
            "sync",
            &["sync"],
            "not in the contract yet; the hub delivers only other programs' activities",
        ),
        board_other(
            "syncLive",
            &["board.sync"],
            "not in the contract yet; a sync adapter will call apply_remote",
        ),
        Row::no(
            b,
            "share",
            Other,
            &["board.share"],
            "not in the contract yet; roles come from the platform identity",
            |c| {
                (
                    vec![
                        c.id(),
                        c.f("role", Choice).choices(&["view", "edit"]).required(),
                    ],
                    vec![c.f("link", Link)],
                )
            },
        ),
        board_other("embed", &["board.embed"], NO_CONTRACT),
        board_other("showPresence", &["presence.show"], PRESENCE),
        board_other("followPresence", &["presence.follow"], PRESENCE),
        board_other("point", &["presence.point"], PRESENCE),
        Row::no(b, "say", Other, &["presence.say"], PRESENCE, |c| {
            (vec![c.id(), c.f("message", Text).required()], vec![])
        }),
        Row::no(b, "react", Other, &["reaction.add"], PRESENCE, |c| {
            (vec![c.id(), c.f("stamp", Text).required()], vec![])
        }),
        Row::no(b, "comment", Create, &["comment.add"], NO_CONTRACT, |c| {
            (vec![c.id(), c.f("text", MultilineText).required()], vec![])
        }),
        Row::no(
            b,
            "history",
            Find,
            &["board.history"],
            "the store keeps the log; the core has no reading of it yet",
            |c| (vec![c.id()], vec![c.f("versions", Text).multiple()]),
        ),
        Row::no(
            b,
            "restore",
            Other,
            &["board.restore"],
            "needs board.history first",
            |c| (vec![c.id(), c.f("version", Text).required()], vec![]),
        ),
    ]
}

/// History, templates and libraries.
fn board_content() -> Vec<Row> {
    let b = BOARD;
    vec![
        Row::no(
            b,
            "templateApply",
            Create,
            &["template.apply"],
            NO_CONTRACT,
            |c| {
                (
                    vec![c.f("template", Text).required()],
                    vec![c.r#ref("id", b)],
                )
            },
        ),
        Row::no(
            b,
            "templateSave",
            Other,
            &["template.save"],
            NO_CONTRACT,
            |c| (vec![c.id(), c.f("name", Text).required()], vec![]),
        ),
        Row::no(
            b,
            "libraryLoad",
            Other,
            &["library.load"],
            NO_CONTRACT,
            |c| (vec![c.id(), c.f("library", Text).required()], vec![]),
        ),
        Row::no(b, "libraryAdd", Other, &["library.add"], NO_CONTRACT, |c| {
            (vec![c.id(), c.f("name", Text).required()], vec![])
        }),
        Row::no(b, "librarySearch", Find, &["library.search"], LATER, |c| {
            (
                vec![c.f("term", Text).required()],
                vec![c.f("found", Text).multiple()],
            )
        }),
        Row::no(
            b,
            "libraryMedia",
            Other,
            &["library.media"],
            "needs the file and media programs (live references)",
            |c| (vec![c.id(), c.f("media", Link).required()], vec![]),
        ),
        Row::no(b, "timer", Other, &["session.timer"], LATER, |c| {
            (vec![c.id(), c.num("seconds").required()], vec![])
        }),
        Row::no(b, "vote", Other, &["vote.cast"], LATER, |c| {
            (vec![c.id(), c.r#ref("element", ELEMENT).required()], vec![])
        }),
    ]
}

/// Facilitation, plug-ins and tools: all optional capabilities.
fn board_sessions() -> Vec<Row> {
    let b = BOARD;
    vec![
        board_other("sequence", &["sequence.run"], LATER),
        board_other(
            "present",
            &["board.present"],
            "presenting is the canvas component's job (not built); the core has the page order",
        ),
        board_other("layout", &["board.layout"], LATER),
        Row::no(b, "audit", Check, &["board.audit"], LATER, |c| {
            (vec![c.id()], vec![c.f("entries", Text).multiple()])
        }),
        board_other(
            "encrypt",
            &["board.encrypt"],
            "scheme not decided; encryption at rest is the store's (ADR 0024)",
        ),
        board_other(
            "installPlugin",
            &["plugin.install"],
            "the extension model of the platform is not decided",
        ),
        board_other(
            "measure",
            &["tool.measure"],
            "a tool of the frontend or a plug-in",
        ),
        board_other(
            "record",
            &["capture.record"],
            "a tool of the frontend or a plug-in",
        ),
    ]
}

fn page() -> Vec<Row> {
    vec![
        Row::no(
            PAGE,
            "change",
            Change,
            &[],
            "the core has no page rename (page `name` is a merge unit but no editor method sets it)",
            |_: &Cmd| (vec![], vec![]),
        ),
        Row::no(
            PAGE,
            "embedDocument",
            Other,
            &["document.embed"],
            "needs the documents program (live reference, ADR 0020)",
            |c| (vec![c.id(), c.f("document", Reference).required()], vec![]),
        ),
    ]
}

fn element() -> Vec<Row> {
    let e = ELEMENT;
    vec![
        Row::no(
            e,
            "select",
            Other,
            &["element.select"],
            "selection is state of the frontend (ADR 0037); the core has no capability for it",
            |c| (vec![c.id()], vec![]),
        ),
        Row::no(e, "addTable", Create, &["table.add"], LATER, |c| {
            (
                vec![c.r#ref("page", PAGE).required()],
                vec![c.r#ref("id", e)],
            )
        }),
        Row::no(
            e,
            "embed",
            Create,
            &["embed.add"],
            "live reference into other programs (ADR 0020), remote content off by default; not built",
            |c| {
                (
                    vec![
                        c.r#ref("page", PAGE).required(),
                        c.f("target", Link).required(),
                    ],
                    vec![c.r#ref("id", e)],
                )
            },
        ),
        Row::no(e, "annotate", Other, &["element.annotate"], LATER, |c| {
            (
                vec![c.id(), c.f("key", Text).required(), c.f("value", Text)],
                vec![],
            )
        }),
        Row::no(
            e,
            "search",
            Find,
            &["element.search"],
            "no full-text search over labels in the core",
            |c| {
                (
                    vec![c.f("term", Text).required()],
                    vec![c.r#ref("elements", e).multiple()],
                )
            },
        ),
        Row::no(
            e,
            "recognise",
            Other,
            &["shape.recognise"],
            "a local algorithm, not built",
            |c| (vec![c.id()], vec![]),
        ),
    ]
}
