//! Formats of a board: the own JSON format and SVG, one strategy per media type.
//!
//! Pattern: Strategy + Registry — a [`Format`] writes a [`Scene`] and may read one back; `find` picks
//! it by media type. Another media type is `Unsupported`. Pure: bytes in, bytes out.

mod import;
mod json;
mod registry;
mod strategy;

pub use json::{JSON_MEDIA_TYPE, JsonFormat};
pub use registry::{SVG_MEDIA_TYPE, SvgFormat, find, import};
pub use strategy::{Format, Imported, Scene};
