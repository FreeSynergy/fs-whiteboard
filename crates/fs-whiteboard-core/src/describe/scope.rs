//! Names and label keys of the described types.
//!
//! Pattern: Value object — a type is a name and a label prefix; a `Cmd` builds the fields of one command with labels
//! `<type>.<command>.<field>` (all lower case, `coding.md` section 2a).

use fs_types::{BasicType, FieldDescription};

/// The program that owns the types (ADR 0050).
pub const PROGRAM: &str = "fs-whiteboard";

/// A described data type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ty {
    /// The name in the catalog.
    pub name: &'static str,
    /// The label prefix.
    pub label: &'static str,
}

/// A board.
pub const BOARD: Ty = Ty {
    name: "Board",
    label: "board",
};
/// A page of a board.
pub const PAGE: Ty = Ty {
    name: "Page",
    label: "page",
};
/// An element on a page.
pub const ELEMENT: Ty = Ty {
    name: "Element",
    label: "element",
};

/// `lineWidth` becomes `line-width`.
pub(crate) fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Builds the fields of one command of one type.
pub struct Cmd {
    ty: Ty,
    key: String,
}

impl Cmd {
    pub(crate) fn new(ty: Ty, command: &str) -> Self {
        Self {
            ty,
            key: kebab(command),
        }
    }

    /// A field of this command.
    pub fn f(&self, name: &str, basic: BasicType) -> FieldDescription {
        FieldDescription::new(name, basic).label(&format!(
            "{}.{}.{}",
            self.ty.label,
            self.key,
            kebab(name)
        ))
    }

    /// A number. There is no numeric basic type yet (`map.md`: quantity is still to come), so it travels as text.
    pub fn num(&self, name: &str) -> FieldDescription {
        self.f(name, BasicType::Text)
    }

    /// Yes or no (no boolean basic type yet).
    pub fn flag(&self, name: &str) -> FieldDescription {
        self.f(name, BasicType::Choice).choices(&["yes", "no"])
    }

    /// A reference to a record of type `to`.
    pub fn r#ref(&self, name: &str, to: Ty) -> FieldDescription {
        self.f(name, BasicType::Reference).target(to.name)
    }

    /// The required reference to the record of this command's own type, named `id`.
    pub fn id(&self) -> FieldDescription {
        self.r#ref("id", self.ty).required()
    }
}
