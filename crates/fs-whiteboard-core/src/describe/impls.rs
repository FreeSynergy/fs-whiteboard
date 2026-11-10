//! `DescribesType` for the three types, and the registry of them.
//!
//! Pattern: Registry entry (trait first) — each type names its standard, its fields and, from the table, ALL its commands.
//! Standards: a board is an ActivityStreams 2.0 `Collection` (W3C, as stored); a page and an element have no standard
//! (`data/standards.md`: none exists for whiteboard shapes, layers and z-order), so they follow the AS2 `Object` model
//! plus the contract's own spec, which the description says.

use fs_types::{DescribesType, DescriptionProblem, TypeDescription, TypeRegistry};

use super::scope::{BOARD, ELEMENT, PAGE, PROGRAM, Ty};
use super::{fields, switch};
use crate::model::{Board, Element, Page};

fn describe(ty: Ty, standard: &str, fields: Vec<fs_types::FieldDescription>) -> TypeDescription {
    let mut t = TypeDescription::new(ty.name, PROGRAM, standard, ty.label);
    t.fields = fields;
    t.commands = switch::commands(ty);
    t
}

impl DescribesType for Board {
    fn describe() -> TypeDescription {
        describe(BOARD, "W3C ActivityStreams 2.0 Collection", fields::board())
    }
}

impl DescribesType for Page {
    fn describe() -> TypeDescription {
        describe(
            PAGE,
            "W3C ActivityStreams 2.0 Object (own spec for pages, no standard exists)",
            fields::page(),
        )
    }
}

impl DescribesType for Element {
    fn describe() -> TypeDescription {
        describe(
            ELEMENT,
            "W3C ActivityStreams 2.0 Object (own spec for shapes, no standard exists)",
            fields::element(),
        )
    }
}

/// The registry of the three types.
///
/// # Errors
/// A defect in a description (a bug of this crate, caught by its tests).
pub fn registry() -> Result<TypeRegistry, DescriptionProblem> {
    let mut r = TypeRegistry::new();
    r.register::<Board>()?
        .register::<Page>()?
        .register::<Element>()?;
    Ok(r)
}
