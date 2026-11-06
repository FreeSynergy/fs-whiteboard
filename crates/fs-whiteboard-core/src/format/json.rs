//! The own format: JSON, with a version, every page and element, no clocks.
//!
//! Pattern: Strategy — `application/vnd.freesynergy.whiteboard+json`. Export writes what is shown;
//! import (see `import.rs`) gives new ids and keeps the references inside the file.

use serde::{Deserialize, Serialize};

use super::{Format, Imported, Scene};
use crate::error::Result;
use crate::model::{Element, Page};

/// Media type of the own format.
pub const JSON_MEDIA_TYPE: &str = "application/vnd.freesynergy.whiteboard+json";
pub(super) const VERSION: u32 = 1;

/// The file.
#[derive(Serialize, Deserialize)]
pub(super) struct Document {
    pub(super) format: String,
    pub(super) version: u32,
    pub(super) name: String,
    pub(super) pages: Vec<Page>,
    pub(super) elements: Vec<Element>,
}

/// The own format.
pub struct JsonFormat;

impl Format for JsonFormat {
    fn media_type(&self) -> &'static str {
        JSON_MEDIA_TYPE
    }

    fn whole_board(&self) -> bool {
        true
    }

    fn export(&self, scene: &Scene) -> Result<Vec<u8>> {
        let doc = Document {
            format: JSON_MEDIA_TYPE.into(),
            version: VERSION,
            name: scene.name.clone(),
            pages: scene.pages.clone(),
            elements: scene.elements.clone(),
        };
        Ok(serde_json::to_vec_pretty(&doc)?)
    }

    fn import(&self, data: &[u8], actor: &str) -> Result<Imported> {
        super::import::read(data, actor)
    }
}
