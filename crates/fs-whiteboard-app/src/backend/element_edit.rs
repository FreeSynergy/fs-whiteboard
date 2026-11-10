//! The commands that change or arrange existing elements.
//!
//! Pattern: Command (a function per name) — a `change` command is partial (a field not sent is unchanged, an empty list
//! clears it; the frontend's rule), so each one reads the element first where the core wants a whole value.

use fs_admin_frontend::{BackendError, Outcome, Values};
use fs_objects::ObjectId;
use fs_whiteboard_core::{Align, ConnectorEnd, Point, Rect, StyleField};

use super::element_add::style;
use crate::convert::{
    flag, id, ids, need, need_choice, need_flag, need_num, num, one as text, sent,
};
use crate::core::Core;

const ALIGN: [&str; 8] = [
    "left",
    "center-x",
    "right",
    "top",
    "center-y",
    "bottom",
    "distribute-x",
    "distribute-y",
];

fn align(key: &str) -> Align {
    match key {
        "center-x" => Align::CenterX,
        "right" => Align::Right,
        "top" => Align::Top,
        "center-y" => Align::CenterY,
        "bottom" => Align::Bottom,
        "distribute-x" => Align::DistributeX,
        "distribute-y" => Align::DistributeY,
        _ => Align::Left,
    }
}

/// Commands on several elements: `group`, `align`.
pub fn many(core: &Core, name: &str, input: &Values) -> Result<Outcome, BackendError> {
    let ids = ids(input, "ids")?;
    let (board, _) = core.find_element(ids[0])?;
    let editor = core.editor(board.id)?;
    let mut out = Outcome::default();
    if name == "group" {
        let group = core.run(editor.group(&ids))?;
        out.id = Some(core.iri(group));
        out.values.insert("group".into(), vec![core.iri(group)]);
    } else {
        let how = align(need_choice(input, "how", &ALIGN)?);
        core.run(editor.align(&ids, how))?;
    }
    Ok(out)
}

/// The style fields that were sent; a field sent empty goes back to the default.
fn style_fields(input: &Values) -> Result<Vec<StyleField>, BackendError> {
    let defaults = style(&Values::new())?;
    let given = style(input)?;
    let mut out = Vec::new();
    let pick = |name: &str| sent(input, name);
    if pick("stroke") {
        out.push(StyleField::Stroke(if text(input, "stroke").is_some() {
            given.stroke
        } else {
            defaults.stroke
        }));
    }
    if pick("fill") {
        out.push(StyleField::Fill(if text(input, "fill").is_some() {
            given.fill
        } else {
            defaults.fill
        }));
    }
    if pick("lineWidth") {
        out.push(StyleField::LineWidth(
            if text(input, "lineWidth").is_some() {
                given.line_width
            } else {
                defaults.line_width
            },
        ));
    }
    if pick("dashed") {
        out.push(StyleField::Dashed(
            flag(input, "dashed")?.unwrap_or(defaults.dashed),
        ));
    }
    if pick("opacity") {
        out.push(StyleField::Opacity(if text(input, "opacity").is_some() {
            given.opacity
        } else {
            defaults.opacity
        }));
    }
    if pick("fontFamily") {
        out.push(StyleField::FontFamily(
            if text(input, "fontFamily").is_some() {
                given.font_family
            } else {
                defaults.font_family
            },
        ));
    }
    if pick("fontSize") {
        out.push(StyleField::FontSize(if text(input, "fontSize").is_some() {
            given.font_size
        } else {
            defaults.font_size
        }));
    }
    Ok(out)
}

/// Commands on one element.
pub fn one(
    core: &Core,
    name: &str,
    target: ObjectId,
    input: &Values,
) -> Result<Outcome, BackendError> {
    let (board, el) = core.find_element(target)?;
    let editor = core.editor(board.id)?;
    let mut out = Outcome {
        id: Some(core.iri(target)),
        ..Outcome::default()
    };
    match name {
        "move" => {
            core.run(editor.move_by(&[target], need_num(input, "dx")?, need_num(input, "dy")?))?;
        }
        "setFrame" => {
            let f = el.frame;
            let frame = Rect::new(
                num(input, "x")?.unwrap_or(f.x),
                num(input, "y")?.unwrap_or(f.y),
                num(input, "width")?.unwrap_or(f.width),
                num(input, "height")?.unwrap_or(f.height),
            );
            core.run(editor.set_frame(target, frame))?;
        }
        "rotate" => core.run(editor.rotate(target, need_num(input, "angle")?))?,
        "copy" => {
            let copies =
                core.run(editor.copy(&[target], need_num(input, "dx")?, need_num(input, "dy")?))?;
            let first = copies.first().map(|c| core.iri(*c)).unwrap_or_default();
            out.values.insert("copy".into(), vec![first]);
        }
        "setStyle" => core.run(editor.set_style(&[target], &style_fields(input)?))?,
        "setText" => core.run(editor.set_text(target, need(input, "text")?))?,
        "setEnd" => {
            let end = if need_choice(input, "end", &["source", "target"])? == "source" {
                ConnectorEnd::Source
            } else {
                ConnectorEnd::Target
            };
            let point = Point {
                x: num(input, "x")?.unwrap_or(0.0),
                y: num(input, "y")?.unwrap_or(0.0),
            };
            core.run(editor.set_end(target, end, id(input, "element")?, point))?;
        }
        "setLocked" => core.run(editor.set_locked(&[target], need_flag(input, "locked")?))?,
        "delete" => core.run(editor.delete(&[target]))?,
        "ungroup" => {
            let members = core.run(editor.ungroup(target))?;
            out.values.insert(
                "members".into(),
                members.iter().map(|m| core.iri(*m)).collect(),
            );
        }
        "setParent" => {
            // a parent not sent stays; sent empty it is the page itself (the frontend's rule for changing)
            let parent = if sent(input, "parent") {
                id(input, "parent")?
            } else {
                el.parent
            };
            core.run(editor.set_parent(&[target], id(input, "page")?, parent))?;
        }
        "reorder" => core.run(editor.reorder(&[target], id(input, "before")?))?,
        _ => return Err(BackendError::NotFound),
    }
    Ok(out)
}
