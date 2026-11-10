//! THE table of capabilities: every capability id of `requirements.md`, the command that carries it, and whether the
//! core can do it.
//!
//! Pattern: Registry (a table of rows) — one place says what exists and what does not; `describe()` of the three types,
//! the app's `Backend` and the test against `requirements.md` all read this table and nothing else. A capability the
//! core cannot do stays here with `supported: false` and a reason; it is never dropped and never faked.

use fs_types::{CommandKind, FieldDescription};

use super::scope::{Cmd, Ty};

/// One command of one type.
#[derive(Clone, Debug)]
pub struct Row {
    /// The type the command belongs to.
    pub ty: Ty,
    /// The command name (lower camel case).
    pub name: &'static str,
    /// What it does to the data.
    pub kind: CommandKind,
    /// The capability ids of `requirements.md` the command carries.
    pub capabilities: &'static [&'static str],
    /// The input.
    pub input: Vec<FieldDescription>,
    /// The result; empty = nothing but success.
    pub result: Vec<FieldDescription>,
    /// Whether the core can do it today.
    pub supported: bool,
    /// Why not (English, for developers and the About page); empty when supported.
    pub reason: &'static str,
}

type Fields = (Vec<FieldDescription>, Vec<FieldDescription>);

impl Row {
    /// A supported command; `fields` gets the helper of the command and returns `(input, result)`.
    pub fn yes(
        ty: Ty,
        name: &'static str,
        kind: CommandKind,
        capabilities: &'static [&'static str],
        fields: impl FnOnce(&Cmd) -> Fields,
    ) -> Self {
        let (input, result) = fields(&Cmd::new(ty, name));
        Self {
            ty,
            name,
            kind,
            capabilities,
            input,
            result,
            supported: true,
            reason: "",
        }
    }

    /// A command the core cannot do yet, with the reason.
    pub fn no(
        ty: Ty,
        name: &'static str,
        kind: CommandKind,
        capabilities: &'static [&'static str],
        reason: &'static str,
        fields: impl FnOnce(&Cmd) -> Fields,
    ) -> Self {
        let mut row = Self::yes(ty, name, kind, capabilities, fields);
        row.supported = false;
        row.reason = reason;
        row
    }
}

/// All rows, in the order the commands are described.
pub fn table() -> Vec<Row> {
    let mut rows = super::rows::supported();
    rows.extend(super::rows_future::unsupported());
    rows
}
