//! One element as SVG.
//!
//! Pattern: Strategy per kind — a `match` on the kind picks the drawing; the style and the rotation
//! are the same for all. Rotation is a `transform` around the middle of the frame.

use std::fmt::Write;

use super::escape;
use crate::format::Scene;
use crate::model::{Anchor, Connector, Element, Kind, Point, Rect, ShapeKind};

fn colour(c: &str) -> String {
    if c.is_empty() {
        "none".into()
    } else {
        escape(c)
    }
}

fn paint(e: &Element) -> String {
    let s = &e.style;
    let dash = if s.dashed {
        " stroke-dasharray=\"6 4\""
    } else {
        ""
    };
    format!(
        "stroke=\"{}\" fill=\"{}\" stroke-width=\"{}\"{dash} opacity=\"{}\"",
        colour(&s.stroke),
        colour(&s.fill),
        s.line_width,
        s.opacity
    )
}

fn text_at(e: &Element, at: Point, text: &str, anchor: &str) -> String {
    let size = e.style.font_size;
    let mut spans = String::new();
    for (i, l) in text.lines().enumerate() {
        let dy = if i == 0 { 0.0 } else { f64::from(size) * 1.2 };
        let _ = write!(
            spans,
            "<tspan x=\"{}\" dy=\"{dy}\">{}</tspan>",
            at.x,
            escape(l)
        );
    }
    format!(
        "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{size}\" text-anchor=\"{anchor}\" stroke=\"none\" fill=\"{}\">{spans}</text>",
        at.x,
        at.y,
        escape(&e.style.font_family),
        colour(&e.style.stroke)
    )
}

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
    format!("<path d=\"{d}\" {paint} fill=\"none\"/>")
}

fn centre(a: &Anchor, scene: &Scene) -> Point {
    a.element
        .and_then(|id| scene.elements.iter().find(|e| e.id == id))
        .map_or(a.point, |e| e.frame.center())
}

fn connector(e: &Element, k: &Connector, scene: &Scene, paint: &str) -> String {
    let (a, b) = (centre(&k.source, scene), centre(&k.target, scene));
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
    let (f, paint) = (e.frame, paint(e));
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
            rect(&f, &format!("{paint} stroke-dasharray=\"6 4\"")) + &text_at(e, at, text, "start")
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
