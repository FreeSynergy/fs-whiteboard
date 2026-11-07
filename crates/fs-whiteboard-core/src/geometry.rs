//! Geometry of elements: bounds after rotation, and alignment. Pure.
//!
//! Pattern: Strategy (alignment rule chosen by [`Align`]) — functions over plain values, so the
//! same rules serve the spatial index, `bounds`, and the component that draws.

use fs_objects::ObjectId;

use crate::model::{Align, Rect};

/// The smallest upright rectangle holding `frame` turned by `angle` radians around its middle.
pub fn bounds(frame: &Rect, angle: f64) -> Rect {
    if angle == 0.0 {
        return *frame;
    }
    let (sin, cos) = angle.sin_cos();
    let (hw, hh) = (frame.width / 2.0, frame.height / 2.0);
    let (ex, ey) = (
        cos.abs() * hw + sin.abs() * hh,
        sin.abs() * hw + cos.abs() * hh,
    );
    let c = frame.center();
    Rect::new(c.x - ex, c.y - ey, 2.0 * ex, 2.0 * ey)
}

/// The smallest rectangle holding all of `rects`, or `None` if there are none.
pub fn union_all<'a>(rects: impl IntoIterator<Item = &'a Rect>) -> Option<Rect> {
    rects.into_iter().copied().reduce(|a, b| a.union(&b))
}

/// The moves `(id, dx, dy)` that align `items` (id and bounds after rotation) as `how` says, relative
/// to the selection as a whole. Elements that stay put are not listed.
pub fn align(items: &[(ObjectId, Rect)], how: Align) -> Vec<(ObjectId, f64, f64)> {
    let Some(all) = union_all(items.iter().map(|(_, r)| r)) else {
        return Vec::new();
    };
    let deltas: Vec<(f64, f64)> = match how {
        Align::Left => items.iter().map(|(_, r)| (all.x - r.x, 0.0)).collect(),
        Align::Right => items
            .iter()
            .map(|(_, r)| (all.right() - r.right(), 0.0))
            .collect(),
        Align::CenterX => items
            .iter()
            .map(|(_, r)| (all.center().x - r.center().x, 0.0))
            .collect(),
        Align::Top => items.iter().map(|(_, r)| (0.0, all.y - r.y)).collect(),
        Align::Bottom => items
            .iter()
            .map(|(_, r)| (0.0, all.bottom() - r.bottom()))
            .collect(),
        Align::CenterY => items
            .iter()
            .map(|(_, r)| (0.0, all.center().y - r.center().y))
            .collect(),
        Align::DistributeX => distribute(items, &all, true),
        Align::DistributeY => distribute(items, &all, false),
    };
    items
        .iter()
        .zip(deltas)
        .filter(|(_, (dx, dy))| *dx != 0.0 || *dy != 0.0)
        .map(|((id, _), (dx, dy))| (*id, dx, dy))
        .collect()
}

type Pick = fn(&Rect) -> f64;

/// Equal gaps along one axis; the outermost two stay. Fewer than three elements: nothing moves.
#[allow(clippy::cast_precision_loss)] // item counts are small
fn distribute(items: &[(ObjectId, Rect)], all: &Rect, horizontal: bool) -> Vec<(f64, f64)> {
    let (start, size): (Pick, Pick) = if horizontal {
        (|r| r.x, |r| r.width)
    } else {
        (|r| r.y, |r| r.height)
    };
    let mut out = vec![(0.0, 0.0); items.len()];
    if items.len() < 3 {
        return out;
    }
    let mut idx: Vec<usize> = (0..items.len()).collect();
    idx.sort_by(|&a, &b| {
        start(&items[a].1)
            .total_cmp(&start(&items[b].1))
            .then(items[a].0.cmp(&items[b].0))
    });
    let span = if horizontal { all.width } else { all.height };
    let sizes: f64 = items.iter().map(|(_, r)| size(r)).sum();
    let gap = (span - sizes) / (items.len() - 1) as f64;
    let mut at = start(all);
    for i in idx {
        let d = at - start(&items[i].1);
        out[i] = if horizontal { (d, 0.0) } else { (0.0, d) };
        at += size(&items[i].1) + gap;
    }
    out
}
