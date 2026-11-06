//! Style of an element and the single-property changes of it.
//!
//! Pattern: Flyweight-ready value type — typed properties, not a free string. Each member is a merge
//! unit, so `set_style` with `StyleField`s changes only what it names.

use serde::{Deserialize, Serialize};

/// Typed style. Colours are CSS hex text, empty = none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Style {
    /// Line colour.
    pub stroke: String,
    /// Fill colour.
    pub fill: String,
    /// Line width, at least 0.
    pub line_width: f64,
    /// Dashed line.
    pub dashed: bool,
    /// Opacity 0..1.
    pub opacity: f32,
    /// Font family.
    pub font_family: String,
    /// Font size.
    pub font_size: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            stroke: String::new(),
            fill: String::new(),
            line_width: 1.0,
            dashed: false,
            opacity: 1.0,
            font_family: String::new(),
            font_size: 16.0,
        }
    }
}

/// One property of a style.
#[derive(Clone, Debug, PartialEq)]
pub enum StyleField {
    /// Line colour.
    Stroke(String),
    /// Fill colour.
    Fill(String),
    /// Line width.
    LineWidth(f64),
    /// Dashed line.
    Dashed(bool),
    /// Opacity 0..1.
    Opacity(f32),
    /// Font family.
    FontFamily(String),
    /// Font size.
    FontSize(f32),
}
