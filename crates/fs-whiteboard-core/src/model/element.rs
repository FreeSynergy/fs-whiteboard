//! [`Element`]: one object on a board.
//!
//! Pattern: Composite (leaf and container share one type) — a typed value with a `Kind`. Every
//! member except the kind is a merge unit (see `Unit`); the kind never changes after `add`.

use fs_objects::ObjectId;
use serde::{Deserialize, Serialize};

use super::geom::Rect;
use super::kind::Kind;
use super::style::Style;

/// An element of a board.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    /// Id (ignored by `add`, which gives a new one).
    pub id: ObjectId,
    /// The board it is on (set by `add`).
    pub board: ObjectId,
    /// The page it is on.
    pub page: ObjectId,
    /// Group or frame it is in; `None` = directly on the page.
    pub parent: Option<ObjectId>,
    /// Position front to back: sorted as bytes, then by id; a later one is in front.
    pub order: String,
    /// Position and size before rotation. A group's frame is the union of its members, at reading.
    pub frame: Rect,
    /// Radians, around the middle of `frame`.
    pub angle: f64,
    /// Style.
    pub style: Style,
    /// Locked elements refuse every edit but `set_locked`.
    pub locked: bool,
    /// A deleted element stays as a marker, so the delete merges and undo can bring it back.
    pub deleted: bool,
    /// Text bound to a shape; other kinds ignore it.
    pub label: String,
    /// What it is.
    pub kind: Kind,
}

impl Element {
    /// A new element value to hand to `add`: default style, not locked, angle 0, no parent. The ids
    /// `id` and `board` and the `order` are placeholders that `add` replaces.
    pub fn new(page: ObjectId, kind: Kind, frame: Rect) -> Self {
        Self {
            id: ObjectId::new(),
            board: ObjectId::new(),
            page,
            parent: None,
            order: String::new(),
            frame,
            angle: 0.0,
            style: Style::default(),
            locked: false,
            deleted: false,
            label: String::new(),
            kind,
        }
    }

    /// The text of a text, note or area element, or the label of a connector.
    pub fn text(&self) -> Option<&str> {
        match &self.kind {
            Kind::Text { text } | Kind::Note { text } | Kind::Area { text } => Some(text),
            Kind::Connector(c) => Some(&c.label),
            _ => None,
        }
    }
}
