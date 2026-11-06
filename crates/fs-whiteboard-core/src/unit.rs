//! The merge units of an element and typed access to their values.
//!
//! Pattern: Registry of properties — every member of `Element` other than the kind is a unit with a
//! clock of its own; the text-like one depends on the kind. A unit is read and written as a JSON
//! value, so merge, undo and the stored form need no code per property.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};

use crate::error::{Error, Result};
use crate::model::{Anchor, Element, Kind};

/// One property that merges by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Unit {
    /// `page`
    Page,
    /// `parent`
    Parent,
    /// `order`
    Order,
    /// `frame`
    Frame,
    /// `angle`
    Angle,
    /// `locked`
    Locked,
    /// `deleted`
    Deleted,
    /// Text of a text, note or area, or the label of a connector.
    Text,
    /// `label`, the text bound to a shape.
    Label,
    /// The source end of a connector.
    Source,
    /// The target end of a connector.
    Target,
    /// `style.stroke`
    StyleStroke,
    /// `style.fill`
    StyleFill,
    /// `style.line_width`
    StyleLineWidth,
    /// `style.dashed`
    StyleDashed,
    /// `style.opacity`
    StyleOpacity,
    /// `style.font_family`
    StyleFontFamily,
    /// `style.font_size`
    StyleFontSize,
}

impl Unit {
    /// Every unit, in a fixed order.
    pub const ALL: [Unit; 18] = [
        Unit::Page,
        Unit::Parent,
        Unit::Order,
        Unit::Frame,
        Unit::Angle,
        Unit::Locked,
        Unit::Deleted,
        Unit::Text,
        Unit::Label,
        Unit::Source,
        Unit::Target,
        Unit::StyleStroke,
        Unit::StyleFill,
        Unit::StyleLineWidth,
        Unit::StyleDashed,
        Unit::StyleOpacity,
        Unit::StyleFontFamily,
        Unit::StyleFontSize,
    ];

    /// True if elements of this kind have the unit.
    pub fn applies_to(self, kind: &Kind) -> bool {
        match self {
            Unit::Text => matches!(
                kind,
                Kind::Text { .. } | Kind::Note { .. } | Kind::Area { .. } | Kind::Connector(_)
            ),
            Unit::Source | Unit::Target => matches!(kind, Kind::Connector(_)),
            _ => true,
        }
    }

    /// The value of the unit, or `None` if the kind has none.
    pub fn read(self, e: &Element) -> Option<Value> {
        if !self.applies_to(&e.kind) {
            return None;
        }
        let s = &e.style;
        let v = match self {
            Unit::Page => to_value(e.page),
            Unit::Parent => to_value(e.parent),
            Unit::Order => to_value(&e.order),
            Unit::Frame => to_value(e.frame),
            Unit::Angle => to_value(e.angle),
            Unit::Locked => to_value(e.locked),
            Unit::Deleted => to_value(e.deleted),
            Unit::Text => to_value(e.text()),
            Unit::Label => to_value(&e.label),
            Unit::Source | Unit::Target => to_value(anchor(e, self == Unit::Source)),
            Unit::StyleStroke => to_value(&s.stroke),
            Unit::StyleFill => to_value(&s.fill),
            Unit::StyleLineWidth => to_value(s.line_width),
            Unit::StyleDashed => to_value(s.dashed),
            Unit::StyleOpacity => to_value(s.opacity),
            Unit::StyleFontFamily => to_value(&s.font_family),
            Unit::StyleFontSize => to_value(s.font_size),
        };
        v.ok()
    }

    /// Sets the unit from a JSON value.
    pub fn write(self, e: &mut Element, v: Value) -> Result<()> {
        if !self.applies_to(&e.kind) {
            return Err(Error::invalid(format!(
                "{self:?} does not exist on this kind"
            )));
        }
        match self {
            Unit::Page => e.page = parse(v)?,
            Unit::Parent => e.parent = parse(v)?,
            Unit::Order => e.order = parse(v)?,
            Unit::Frame => e.frame = parse(v)?,
            Unit::Angle => e.angle = parse(v)?,
            Unit::Locked => e.locked = parse(v)?,
            Unit::Deleted => e.deleted = parse(v)?,
            Unit::Text => set_text(e, parse(v)?),
            Unit::Label => e.label = parse(v)?,
            Unit::Source | Unit::Target => {
                let a: Anchor = parse(v)?;
                if let Kind::Connector(c) = &mut e.kind {
                    *(if self == Unit::Source {
                        &mut c.source
                    } else {
                        &mut c.target
                    }) = a;
                }
            }
            Unit::StyleStroke => e.style.stroke = parse(v)?,
            Unit::StyleFill => e.style.fill = parse(v)?,
            Unit::StyleLineWidth => e.style.line_width = parse(v)?,
            Unit::StyleDashed => e.style.dashed = parse(v)?,
            Unit::StyleOpacity => e.style.opacity = parse(v)?,
            Unit::StyleFontFamily => e.style.font_family = parse(v)?,
            Unit::StyleFontSize => e.style.font_size = parse(v)?,
        }
        Ok(())
    }
}

fn parse<T: DeserializeOwned>(v: Value) -> Result<T> {
    Ok(serde_json::from_value(v)?)
}

fn anchor(e: &Element, source: bool) -> Anchor {
    match &e.kind {
        Kind::Connector(c) if source => c.source,
        Kind::Connector(c) => c.target,
        _ => Anchor::default(),
    }
}

fn set_text(e: &mut Element, text: String) {
    match &mut e.kind {
        Kind::Text { text: t } | Kind::Note { text: t } | Kind::Area { text: t } => *t = text,
        Kind::Connector(c) => c.label = text,
        _ => {}
    }
}
