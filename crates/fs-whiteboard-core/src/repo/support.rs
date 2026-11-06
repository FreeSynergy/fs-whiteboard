//! What this provider reports it can do.
//!
//! Pattern: Null Object (report) — one place that lists every capability id the contract names,
//! and those it says are not in it yet; the ones not built say so plainly (ADR 0045).

use fs_interfaces::Capabilities;

use super::handle::WhiteboardRepo;

/// Capability ids that `WhiteboardRepo` provides (`whiteboard.capnp`, v0.26.0).
const SUPPORTED: [&str; 31] = [
    "board.create",
    "board.open",
    "board.delete",
    "board.import",
    "board.export",
    "board.save",
    "board.view",
    "board.subscribe",
    "canvas.navigate",
    "page.add",
    "page.remove",
    "shape.add",
    "text.add",
    "note.add",
    "stroke.draw",
    "connector.add",
    "image.add",
    "frame.add",
    "element.move",
    "element.style",
    "element.lock",
    "element.delete",
    "element.group",
    "element.ungroup",
    "element.reorder",
    "element.align",
    "text.edit",
    "connector.edit",
    "stroke.erase",
    "history.undo",
    "history.redo",
];

/// Named in the contract as not in it yet, or in the requirements without a built counterpart.
const NOT_BUILT: [(&str, &str); 14] = [
    (
        "sync",
        "not in the contract yet; the hub delivers only other programs' activities",
    ),
    (
        "board.sync",
        "not in the contract yet; a sync adapter will call apply_remote",
    ),
    (
        "board.share",
        "not in the contract yet; roles come from the platform identity",
    ),
    (
        "presence.*",
        "not in the contract yet; the shared Presence interface has no owner",
    ),
    (
        "presence.show",
        "not in the contract yet; passing data is never stored",
    ),
    (
        "presence.follow",
        "not in the contract yet; passing data is never stored",
    ),
    (
        "presence.point",
        "not in the contract yet; passing data is never stored",
    ),
    ("library.*", "not in the contract yet"),
    ("library.load", "not in the contract yet"),
    ("library.add", "not in the contract yet"),
    ("template.*", "not in the contract yet"),
    ("template.apply", "not in the contract yet"),
    ("template.save", "not in the contract yet"),
    ("comment.add", "not in the contract yet"),
];

impl WhiteboardRepo {
    pub(super) fn support() -> Capabilities {
        let mut c = Capabilities::new("fs-whiteboard-core");
        for id in SUPPORTED {
            c.supported(id);
        }
        for (id, why) in NOT_BUILT {
            c.unsupported(id, why);
        }
        c.unsupported(
            "board.export.png",
            "PNG export is not built; only the own format and SVG",
        );
        c.unsupported(
            "board.export.pdf",
            "PDF export is not built; only the own format and SVG",
        );
        c
    }
}
