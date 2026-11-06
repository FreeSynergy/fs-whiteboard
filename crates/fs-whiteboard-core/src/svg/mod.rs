//! SVG export: our own writer, XML-escaped. Pure.
//!
//! Pattern: Strategy (export format) — rotation as `transform`, a stroke as a path, an image by its
//! file IRI, text and IRIs escaped. Only what the scene holds is drawn; the component decides how a
//! board looks on screen.

mod draw;
mod escape;
mod paint;
mod render;

pub use escape::escape;
pub use render::render;
