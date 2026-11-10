//! The one switch between "the description carries everything" and "the description says what is supported".
//!
//! Pattern: Adapter — `emit` is the only place that turns a table [`Row`] into a `CommandDescription`.
//! TRANSITION: `types.capnp` has no field `supported` yet (Thread T adds it), and `fs-types` has none on
//! `CommandDescription`. Until then every row is emitted, supported or not, so a frontend can show the control
//! ("coming"); the flag lives in the table only. When `CommandDescription` has the field, set it HERE from
//! `row.supported` (and the reason, if the field takes one) — no other file changes, and the frontend does not change.

use fs_types::CommandDescription;

use super::scope::Ty;
use super::table::{Row, table};

/// The command description of a row (all rows are emitted, see the module head).
pub fn emit(row: &Row) -> CommandDescription {
    let label = format!("{}.{}", row.ty.label, super::scope::kebab(row.name));
    let mut c = CommandDescription::new(row.name, row.kind, &label);
    c.input.clone_from(&row.input);
    c.result.clone_from(&row.result);
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
