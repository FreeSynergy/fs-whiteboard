//! Typed text in, typed text out: the values of the description and the core's types.
//!
//! Pattern: Adapter (functions) — the description carries numbers as text and yes/no as a choice (there is no numeric or
//! boolean basic type yet); this file reads them, refusing with the finding the form marks (`Rejected`).

use fs_admin_frontend::{BackendError, Values};
use fs_objects::ObjectId;
use fs_types::{RecordIssue, RecordProblem, TypeProblem};
use fs_whiteboard_core::{Point, StrokePoint};

use crate::ids::parse_iri;

fn reject(field: &str, problem: RecordProblem) -> BackendError {
    BackendError::Rejected(vec![RecordIssue {
        field: field.to_owned(),
        index: 0,
        problem,
    }])
}

/// The one finding for a value that is not a number. `Character` (a character the type does not hold) is the nearest code
/// until there is a quantity basic type.
fn not_a_number(field: &str) -> BackendError {
    reject(field, RecordProblem::Value(TypeProblem::Character))
}

/// The first non-empty value of `field`, trimmed.
pub fn one<'a>(v: &'a Values, field: &str) -> Option<&'a str> {
    v.get(field)?
        .iter()
        .map(|t| t.trim())
        .find(|t| !t.is_empty())
}

/// Whether `field` was sent at all (an empty list means "clear" in a `change`).
pub fn sent(v: &Values, field: &str) -> bool {
    v.contains_key(field)
}

/// A required text.
pub fn need<'a>(v: &'a Values, field: &str) -> Result<&'a str, BackendError> {
    one(v, field).ok_or_else(|| reject(field, RecordProblem::Missing))
}

/// A finite number, if given.
pub fn num(v: &Values, field: &str) -> Result<Option<f64>, BackendError> {
    match one(v, field) {
        None => Ok(None),
        Some(t) => t
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(Some)
            .ok_or_else(|| not_a_number(field)),
    }
}

/// A required finite number.
pub fn need_num(v: &Values, field: &str) -> Result<f64, BackendError> {
    num(v, field)?.ok_or_else(|| reject(field, RecordProblem::Missing))
}

/// `yes` or `no`, if given.
pub fn flag(v: &Values, field: &str) -> Result<Option<bool>, BackendError> {
    match one(v, field) {
        None => Ok(None),
        Some("yes") => Ok(Some(true)),
        Some("no") => Ok(Some(false)),
        Some(_) => Err(reject(field, RecordProblem::Value(TypeProblem::NotInList))),
    }
}

/// A required `yes` or `no`.
pub fn need_flag(v: &Values, field: &str) -> Result<bool, BackendError> {
    flag(v, field)?.ok_or_else(|| reject(field, RecordProblem::Missing))
}

/// The id a reference names, if given.
pub fn id(v: &Values, field: &str) -> Result<Option<ObjectId>, BackendError> {
    match one(v, field) {
        None => Ok(None),
        Some(t) => parse_iri(t)
            .map(Some)
            .ok_or_else(|| reject(field, RecordProblem::Value(TypeProblem::BadUri))),
    }
}

/// A required reference.
pub fn need_id(v: &Values, field: &str) -> Result<ObjectId, BackendError> {
    id(v, field)?.ok_or_else(|| reject(field, RecordProblem::Missing))
}

/// All references of a multiple field, in order.
pub fn ids(v: &Values, field: &str) -> Result<Vec<ObjectId>, BackendError> {
    let mut out = Vec::new();
    for (i, t) in v.get(field).into_iter().flatten().enumerate() {
        let t = t.trim();
        if t.is_empty() {
            continue;
        }
        out.push(parse_iri(t).ok_or_else(|| {
            BackendError::Rejected(vec![RecordIssue {
                field: field.to_owned(),
                index: i,
                problem: RecordProblem::Value(TypeProblem::BadUri),
            }])
        })?);
    }
    if out.is_empty() {
        return Err(reject(field, RecordProblem::Missing));
    }
    Ok(out)
}

/// A choice key, checked against `allowed`.
pub fn choice<'a>(
    v: &'a Values,
    field: &str,
    allowed: &[&str],
) -> Result<Option<&'a str>, BackendError> {
    match one(v, field) {
        None => Ok(None),
        Some(t) if allowed.contains(&t) => Ok(Some(t)),
        Some(_) => Err(reject(field, RecordProblem::Value(TypeProblem::NotInList))),
    }
}

/// A required choice.
pub fn need_choice<'a>(
    v: &'a Values,
    field: &str,
    allowed: &[&str],
) -> Result<&'a str, BackendError> {
    choice(v, field, allowed)?.ok_or_else(|| reject(field, RecordProblem::Missing))
}

/// A number that fits the single precision of the wire (opacity, font size, pressure), if given.
pub fn num32(v: &Values, field: &str) -> Result<Option<f32>, BackendError> {
    match one(v, field) {
        None => Ok(None),
        Some(t) => t
            .parse::<f32>()
            .ok()
            .filter(|n| n.is_finite())
            .map(Some)
            .ok_or_else(|| not_a_number(field)),
    }
}

/// The points of a stroke: one line each, `x,y` or `x,y,pressure`, as fractions of the frame.
pub fn points(v: &Values, field: &str) -> Result<Vec<StrokePoint>, BackendError> {
    let Some(text) = one(v, field) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
        let number = |at: usize| {
            parts
                .get(at)
                .and_then(|t| t.parse::<f64>().ok())
                .filter(|n| n.is_finite())
        };
        let (Some(px), Some(py)) = (number(0), number(1)) else {
            return Err(not_a_number(field));
        };
        let pressure = match parts.get(2) {
            _ if parts.len() > 3 => return Err(not_a_number(field)),
            None => None,
            Some(t) => Some(
                t.parse::<f32>()
                    .ok()
                    .filter(|p| p.is_finite())
                    .ok_or_else(|| not_a_number(field))?,
            ),
        };
        out.push(StrokePoint {
            x: px,
            y: py,
            pressure,
        });
    }
    Ok(out)
}

/// A point from two fields, `(0, 0)` where missing.
pub fn point(v: &Values, x: &str, y: &str) -> Result<Point, BackendError> {
    Ok(Point {
        x: num(v, x)?.unwrap_or(0.0),
        y: num(v, y)?.unwrap_or(0.0),
    })
}

/// A number as typed text.
pub fn show(n: f64) -> String {
    format!("{n}")
}

/// A flag as its choice key.
pub fn show_flag(b: bool) -> String {
    (if b { "yes" } else { "no" }).to_owned()
}

/// The points of a stroke as the lines [`points`] reads.
pub fn show_points(points: &[StrokePoint]) -> String {
    points
        .iter()
        .map(|p| match p.pressure {
            Some(pr) => format!("{},{},{pr}", p.x, p.y),
            None => format!("{},{}", p.x, p.y),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
