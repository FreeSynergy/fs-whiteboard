//! The registry of formats by media type.
//!
//! Pattern: Registry — a media type (case and parameters ignored) finds its strategy; unknown ones
//! are `Unsupported`, never guessed.

use fs_interfaces::Unsupported;

use super::{Format, Imported, JsonFormat, Scene};
use crate::error::{Error, Result};
use crate::svg;

/// Media type of SVG.
pub const SVG_MEDIA_TYPE: &str = "image/svg+xml";

/// SVG: export only.
pub struct SvgFormat;

impl Format for SvgFormat {
    fn media_type(&self) -> &'static str {
        SVG_MEDIA_TYPE
    }
    fn export(&self, scene: &Scene) -> Result<Vec<u8>> {
        Ok(svg::render(scene).into_bytes())
    }
}

static FORMATS: [&dyn Format; 2] = [&JsonFormat, &SvgFormat];

/// The format of a media type.
pub fn find(media_type: &str) -> Option<&'static dyn Format> {
    let base = media_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    FORMATS.iter().copied().find(|f| f.media_type() == base)
}

/// Reads a board in the given media type; others are `Unsupported`.
pub fn import(media_type: &str, data: &[u8], actor: &str) -> Result<Imported> {
    match find(media_type) {
        Some(f) => f.import(data, actor),
        None => Err(Error::Unsupported(Unsupported::new(
            "board.import",
            format!("no import of {media_type}"),
        ))),
    }
}
