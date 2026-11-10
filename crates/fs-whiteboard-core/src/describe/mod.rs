//! The type descriptions of the three data types (ADR 0066): [`Board`](crate::Board), [`Page`](crate::Page),
//! [`Element`](crate::Element), reported through `fs_types::DescribesType`.
//!
//! Pattern: Registry entry (trait first, ADR 0043) — each type implements `DescribesType`, so
//! `TypeRegistry::register::<T>()` puts it in the catalog. The commands come from ONE table (`table`) that holds every
//! capability id of the requirements, supported or not; `switch::emit` is the one function that turns a row into a
//! `CommandDescription`. See `DESIGN.md`.

mod fields;
mod impls;
mod rows;
mod rows_future;
mod scope;
mod switch;
mod table;

pub use fields::{BOARD_FORMATS, EXPORT_FORMATS};
pub use impls::registry;
pub use scope::{BOARD, ELEMENT, PAGE, Ty};
pub use switch::supported_commands;
pub use table::{Row, table};
