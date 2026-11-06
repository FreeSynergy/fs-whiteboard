//! The SVG document around the elements: header, view box, marker, title.
//!
//! Pattern: Template method — fixed frame, the elements are drawn by `draw`.

use std::fmt::Write;

use super::{draw, escape};

use crate::format::Scene;
use crate::geometry::{bounds, union_all};
use crate::model::Rect;

const PAD: f64 = 10.0;

/// The scene as an SVG document.
pub fn render(scene: &Scene) -> String {
    let rects: Vec<_> = scene
        .elements
        .iter()
        .map(|e| match scene.ends.get(&e.id) {
            Some((a, b)) => Rect::new(
                a.x.min(b.x),
                a.y.min(b.y),
                (a.x - b.x).abs(),
                (a.y - b.y).abs(),
            ),
            None => bounds(&e.frame, e.angle),
        })
        .collect();
    let (x, y, w, h) = union_all(&rects).map_or((0.0, 0.0, 100.0, 100.0), |r| {
        (
            r.x - PAD,
            r.y - PAD,
            r.width + 2.0 * PAD,
            r.height + 2.0 * PAD,
        )
    });
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{x} {y} {w} {h}\" width=\"{w}\" height=\"{h}\">"
    );
    let _ = writeln!(out, "<title>{}</title>", escape(&scene.name));
    out.push_str("<defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" markerWidth=\"8\" markerHeight=\"8\" orient=\"auto-start-reverse\"><path d=\"M0,0 L10,5 L0,10 z\"/></marker></defs>\n");
    for e in &scene.elements {
        out.push_str(&draw::element(e, scene));
    }
    out.push_str("</svg>\n");
    out
}
