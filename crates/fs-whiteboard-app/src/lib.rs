//! `fs-whiteboard-app`: the window of the whiteboard program (ADR 0065, 0066).
//!
//! The app is the window shell plus the generated admin frontend over the descriptions of `fs-whiteboard-core`. It has
//! no render code and no look: the board canvas is a future component of `fs-components` (`DESIGN.md`, Unsupported).
//! It imports no `blinc_*` crate. The core is called in process, with the local store opened as `DESIGN.md` says.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::module_name_repetitions
)]

mod about;
mod args;
mod backend;
mod convert;
mod core;
mod ids;
mod program;
mod words;

pub use about::about;
pub use args::{Args, HELP, parse};
pub use backend::{BoardBackend, ElementBackend, PageBackend};
pub use core::Core;
pub use ids::{iri, parse_iri};
pub use program::{app_command, build_window};
pub use words::{TITLE, Words, keys};
