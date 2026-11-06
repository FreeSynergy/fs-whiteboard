//! The kind of an element and the data that belongs to one kind only.
//!
//! Pattern: Composite (typed variants) — the kind is fixed when an element is added; so are a
//! stroke's points, an image's IRI and a connector's arrow. Only text-like members change later.

use fs_store::ObjectId;
use serde::{Deserialize, Serialize};

use super::geom::{Point, StrokePoint};

/// The form of a basic shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapeKind {
    /// Rectangle.
    Rectangle,
    /// Ellipse.
    Ellipse,
    /// Diamond.
    Diamond,
    /// A line from the top-left to the bottom-right corner of the frame.
    Line,
}

/// Which end of a connector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectorEnd {
    /// Where the connector starts.
    Source,
    /// Where the connector ends.
    Target,
}

/// One end of a connector: follows `element`, or is free at `point` when `element` is `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Anchor {
    /// The element followed, if any.
    pub element: Option<ObjectId>,
    /// The free end.
    pub point: Point,
}

/// A line or arrow between two elements or points.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Connector {
    /// Start.
    pub source: Anchor,
    /// End.
    pub target: Anchor,
    /// Text on the line.
    pub label: String,
    /// Arrow head at the target; fixed when added.
    pub arrow_end: bool,
}

/// What an element is.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Kind {
    /// A basic shape.
    Shape {
        /// Which one.
        shape: ShapeKind,
    },
    /// Free plain text.
    Text {
        /// The text.
        text: String,
    },
    /// Text on a coloured shape.
    Note {
        /// The text.
        text: String,
    },
    /// A freehand line, at most 100 000 points; never changes after `add`.
    Stroke {
        /// Points relative to the frame.
        points: Vec<StrokePoint>,
    },
    /// A connector.
    Connector(Connector),
    /// An image, by file IRI.
    Image {
        /// IRI of the file.
        iri: String,
    },
    /// Holds the elements whose parent it is; has no position of its own.
    Group,
    /// A named, bounded frame on the board.
    Area {
        /// Its name.
        text: String,
    },
}

impl Kind {
    /// Short name, as in the index field `kind`.
    pub fn name(&self) -> &'static str {
        match self {
            Kind::Shape { .. } => "shape",
            Kind::Text { .. } => "text",
            Kind::Note { .. } => "note",
            Kind::Stroke { .. } => "stroke",
            Kind::Connector(_) => "connector",
            Kind::Image { .. } => "image",
            Kind::Group => "group",
            Kind::Area { .. } => "area",
        }
    }

    /// True for kinds that can be the `parent` of other elements.
    pub fn is_container(&self) -> bool {
        matches!(self, Kind::Group | Kind::Area { .. })
    }
}
