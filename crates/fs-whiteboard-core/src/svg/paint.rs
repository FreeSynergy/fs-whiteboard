//! Paint and text attributes of an element.
//!
//! Pattern: Strategy helpers for `draw` — each attribute is written exactly once.

use std::fmt::Write;

use super::escape;
use crate::model::{Element, Point};

pub(super) fn colour(c: &str) -> String {
    if c.is_empty() {
        "none".into()
    } else {
        escape(c)
    }
}

/// The paint attributes, each exactly once: `fill` overrides the style's fill, `dashed` forces dashes.
pub(super) fn paint(e: &Element, fill: Option<&str>, dashed: bool) -> String {
    let s = &e.style;
    let dash = if s.dashed || dashed {
        " stroke-dasharray=\"6 4\""
    } else {
        ""
    };
    format!(
        "stroke=\"{}\" fill=\"{}\" stroke-width=\"{}\"{dash} opacity=\"{}\"",
        colour(&s.stroke),
        fill.map_or_else(|| colour(&s.fill), str::to_string),
        s.line_width,
        s.opacity
    )
}

pub(super) fn text_at(e: &Element, at: Point, text: &str, anchor: &str) -> String {
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
