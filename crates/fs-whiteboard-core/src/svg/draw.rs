//! One element as SVG.
//!
//! Pattern: Strategy per kind — a `match` on the kind picks the drawing; the style and the rotation
//! are the same for all. Rotation is a `transform` around the middle of the frame.

use std::fmt::Write;

use super::escape;
use super::paint::{paint, text_at};
use crate::format::Scene;
use crate::model::{Anchor, Connector, Element, Kind, Point, Rect, ShapeKind};

fn rect(f: &Rect, paint: &str) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {paint}/>",
        f.x, f.y, f.width, f.height
    )
}

fn shape(shape: ShapeKind, f: &Rect, paint: &str) -> String {
    let c = f.center();
    match shape {
        ShapeKind::Rectangle => rect(f, paint),
        ShapeKind::Ellipse => format!(
            "<ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" {paint}/>",
            c.x,
            c.y,
            f.width / 2.0,
            f.height / 2.0
        ),
        ShapeKind::Diamond => format!(
            "<polygon points=\"{},{} {},{} {},{} {},{}\" {paint}/>",
            c.x,
            f.y,
            f.right(),
            c.y,
            c.x,
            f.bottom(),
            f.x,
            c.y
        ),
        ShapeKind::Line => format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" {paint}/>",
            f.x,
            f.y,
            f.right(),
            f.bottom()
        ),
    }
}

fn stroke(e: &Element, points: &[crate::model::StrokePoint], paint: &str) -> String {
    let f = e.frame;
    let mut d = String::new();
    for (i, p) in points.iter().enumerate() {
        let cmd = if i == 0 { "M" } else { " L" };
        let _ = write!(d, "{cmd}{} {}", f.x + p.x * f.width, f.y + p.y * f.height);
    }
    format!("<path d=\"{d}\" {paint}/>")
}

fn centre(a: &Anchor, scene: &Scene) -> Point {
    a.element
        .and_then(|id| scene.elements.iter().find(|e| e.id == id))
        .map_or(a.point, |e| e.frame.center())
}

/// Both ends: as resolved at reading (a deleted element's last frame), else from the anchors.
fn ends(e: &Element, k: &Connector, scene: &Scene) -> (Point, Point) {
    scene
        .ends
        .get(&e.id)
        .copied()
        .unwrap_or_else(|| (centre(&k.source, scene), centre(&k.target, scene)))
}

fn connector(e: &Element, k: &Connector, scene: &Scene, paint: &str) -> String {
    let (a, b) = ends(e, k, scene);
    let marker = if k.arrow_end {
        " marker-end=\"url(#arrow)\""
    } else {
        ""
    };
    let mid = Point {
        x: f64::midpoint(a.x, b.x),
        y: f64::midpoint(a.y, b.y),
    };
    let label = if k.label.is_empty() {
        String::new()
    } else {
        text_at(e, mid, &k.label, "middle")
    };
    format!(
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" {paint}{marker}/>{label}",
        a.x, a.y, b.x, b.y
    )
}

fn body(e: &Element, scene: &Scene) -> String {
    let (fill, dashed) = match &e.kind {
        Kind::Stroke { .. } | Kind::Connector(_) => (Some("none"), false),
        Kind::Area { .. } => (None, true),
        _ => (None, false),
    };
    let (f, paint) = (e.frame, paint(e, fill, dashed));
    let size = f64::from(e.style.font_size);
    match &e.kind {
        Kind::Shape { shape: s } => shape(*s, &f, &paint),
        Kind::Text { text } => text_at(
            e,
            Point {
                x: f.x,
                y: f.y + size,
            },
            text,
            "start",
        ),
        Kind::Note { text } => {
            let at = Point {
                x: f.x + 4.0,
                y: f.y + size,
            };
            rect(&f, &paint) + &text_at(e, at, text, "start")
        }
        Kind::Area { text } => {
            let at = Point {
                x: f.x,
                y: f.y - 4.0,
            };
            rect(&f, &paint) + &text_at(e, at, text, "start")
        }
        Kind::Stroke { points } => stroke(e, points, &paint),
        Kind::Connector(k) => connector(e, k, scene, &paint),
        Kind::Image { iri } => format!(
            "<image href=\"{}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"/>",
            escape(iri),
            f.x,
            f.y,
            f.width,
            f.height
        ),
        Kind::Group => String::new(),
    }
}

/// The element as a group with its rotation.
pub(super) fn element(e: &Element, scene: &Scene) -> String {
    let c = e.frame.center();
    let transform = if e.angle == 0.0 {
        String::new()
    } else {
        format!(
            " transform=\"rotate({} {} {})\"",
            e.angle.to_degrees(),
            c.x,
            c.y
        )
    };
    let label = if matches!(e.kind, Kind::Shape { .. }) && !e.label.is_empty() {
        text_at(e, c, &e.label, "middle")
    } else {
        String::new()
    };
    format!(
        "<g id=\"e-{}\"{transform}>{}{label}</g>\n",
        e.id,
        body(e, scene)
    )
}
