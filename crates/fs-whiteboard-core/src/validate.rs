//! The refusals at every entry: non-finite numbers, negative sizes, opacity, limits. Pure.
//!
//! Pattern: Specification — small checks that every entry point (editor, import) calls, so nothing
//! unusable reaches the store.

mod docs;

pub use docs::{board_doc, counter, element_doc, page, remote_element};

use crate::error::{Error, Result};
use crate::model::{Anchor, Element, Kind, Point, Rect, StrokePoint, Style, StyleField};

/// Most points in one stroke; a longer line is several strokes.
pub const MAX_STROKE_POINTS: usize = 100_000;
/// Most bytes of text in one element.
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;
/// Most bytes of an import.
pub const MAX_IMPORT_BYTES: usize = 32 * 1024 * 1024;
/// Most bytes of an `order` key.
pub const MAX_ORDER_BYTES: usize = 1024;

/// A finite number.
pub fn finite(what: &str, v: f64) -> Result<()> {
    if v.is_finite() {
        Ok(())
    } else {
        Err(Error::invalid(format!("{what} must be a finite number")))
    }
}

/// A point with finite coordinates.
pub fn point(p: &Point) -> Result<()> {
    finite("x", p.x)?;
    finite("y", p.y)
}

/// A rectangle with finite numbers and no negative size.
pub fn rect(r: &Rect) -> Result<()> {
    for (n, v) in [
        ("x", r.x),
        ("y", r.y),
        ("width", r.width),
        ("height", r.height),
    ] {
        finite(n, v)?;
    }
    if r.width < 0.0 || r.height < 0.0 {
        return Err(Error::invalid("width and height must not be negative"));
    }
    finite("right edge", r.right())?;
    finite("bottom edge", r.bottom())
}

/// Text within the limit.
pub fn text(s: &str) -> Result<()> {
    if s.len() > MAX_TEXT_BYTES {
        return Err(Error::TooLarge(format!(
            "text of {} bytes, at most {MAX_TEXT_BYTES}",
            s.len()
        )));
    }
    Ok(())
}

/// A colour: empty (none) or CSS hex `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`.
pub fn colour(c: &str) -> Result<()> {
    let hex = c.strip_prefix('#').unwrap_or("");
    let ok = c.is_empty()
        || (matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|b| b.is_ascii_hexdigit()));
    if ok {
        Ok(())
    } else {
        Err(Error::invalid("a colour must be CSS hex like #1a2b3c"))
    }
}

/// An image IRI: `https` or `http`, within the text limit.
pub fn image_iri(iri: &str) -> Result<()> {
    text(iri)?;
    let l = iri.to_ascii_lowercase();
    if (l.starts_with("https://") || l.starts_with("http://")) && !iri.chars().any(char::is_control)
    {
        Ok(())
    } else {
        Err(Error::invalid("an image IRI must be http or https"))
    }
}

/// An `order` key made by `order`, at most 1 KiB.
pub fn order_key(k: &str) -> Result<()> {
    if k.len() > MAX_ORDER_BYTES || !crate::order::is_valid(k) {
        return Err(Error::invalid(
            "not a valid order key (base 62, at most 1 KiB)",
        ));
    }
    Ok(())
}

/// One style property.
pub fn style_field(f: &StyleField) -> Result<()> {
    match f {
        StyleField::Stroke(s) | StyleField::Fill(s) => colour(s),
        StyleField::FontFamily(s) => text(s),
        StyleField::LineWidth(w) => {
            finite("lineWidth", *w)?;
            if *w < 0.0 {
                return Err(Error::invalid("lineWidth must not be negative"));
            }
            Ok(())
        }
        StyleField::Opacity(o) if !(o.is_finite() && (0.0..=1.0).contains(o)) => {
            Err(Error::invalid("opacity must be between 0 and 1"))
        }
        StyleField::FontSize(s) if !(s.is_finite() && *s >= 0.0) => Err(Error::invalid(
            "fontSize must be a finite number, not negative",
        )),
        _ => Ok(()),
    }
}

/// A whole style.
pub fn style(s: &Style) -> Result<()> {
    for f in [
        StyleField::Stroke(s.stroke.clone()),
        StyleField::Fill(s.fill.clone()),
        StyleField::LineWidth(s.line_width),
        StyleField::Opacity(s.opacity),
        StyleField::FontFamily(s.font_family.clone()),
        StyleField::FontSize(s.font_size),
    ] {
        style_field(&f)?;
    }
    Ok(())
}

fn stroke_points(points: &[StrokePoint]) -> Result<()> {
    if points.len() > MAX_STROKE_POINTS {
        return Err(Error::TooLarge(format!(
            "stroke of {} points, at most {MAX_STROKE_POINTS}; use several strokes",
            points.len()
        )));
    }
    for p in points {
        finite("x", p.x)?;
        finite("y", p.y)?;
        if p.pressure
            .is_some_and(|v| !(v.is_finite() && (0.0..=1.0).contains(&v)))
        {
            return Err(Error::invalid("pressure must be between 0 and 1"));
        }
    }
    Ok(())
}

fn anchor(a: &Anchor) -> Result<()> {
    point(&a.point)
}

/// Everything about an element that does not depend on the board: numbers (also what follows from
/// them, the bounds after rotation), limits, text, colours.
pub fn element(e: &Element) -> Result<()> {
    rect(&e.frame)?;
    finite("angle", e.angle)?;
    finite("angle in degrees", e.angle.to_degrees())?;
    rect(&crate::geometry::bounds(&e.frame, e.angle))?;
    style(&e.style)?;
    text(&e.label)?;
    match &e.kind {
        Kind::Stroke { points } => stroke_points(points),
        Kind::Connector(c) => {
            text(&c.label)?;
            anchor(&c.source)?;
            anchor(&c.target)
        }
        Kind::Text { text: t } | Kind::Note { text: t } | Kind::Area { text: t } => text(t),
        Kind::Image { iri } => image_iri(iri),
        Kind::Shape { .. } | Kind::Group => Ok(()),
    }
}
