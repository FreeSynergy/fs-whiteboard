//! The one place that turns a table [`Row`] into a `CommandDescription`.
//!
//! Pattern: Adapter — `emit` is the only such function. Every row is emitted, supported or not (the rule of
//! `types.capnp`, fs-interfaces v0.0.11): `supported` comes from the row, so a frontend shows the control locked
//! ("coming") and frees it when the core reports it. `find` is the standard one (`standard_find`: input `term`, empty =
//! the list) plus the filters of the row; `change` has no input list (its input is an `Edit`).
//! SWITCH (done): before v0.0.11 the flag lived in the table only; now `emit` sets it from `row.supported`.

use fs_types::{CommandDescription, CommandKind};

use super::scope::{Ty, kebab};
use super::table::{Row, table};

/// The command description of a row.
pub fn emit(row: &Row) -> CommandDescription {
    let label = format!("{}.{}", row.ty.label, kebab(row.name));
    let standard = row.kind == CommandKind::Find && row.name == "find";
    let mut c = if standard {
        CommandDescription::standard_find(&label, row.ty.name)
    } else {
        CommandDescription::new(row.name, row.kind, &label)
    };
    c.input.extend(row.input.iter().cloned());
    if !standard {
        c.result.extend(row.result.iter().cloned());
    }
    if !row.supported {
        c = c.unsupported();
    }
    c
}

/// The commands of a type, all of them.
pub fn commands(ty: Ty) -> Vec<CommandDescription> {
    table().iter().filter(|r| r.ty == ty).map(emit).collect()
}

/// The names of the commands of `ty` the core can do (for a backend that must refuse the others).
pub fn supported_commands(ty: Ty) -> Vec<&'static str> {
    table()
        .iter()
        .filter(|r| r.ty == ty && r.supported)
        .map(|r| r.name)
        .collect()
}
