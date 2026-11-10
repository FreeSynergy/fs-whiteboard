//! The About page (ADR 0065 point 2).
//!
//! Pattern: Value object — name, version, origin, licence and repository, read from the package, not typed twice.

use fs_admin_frontend::view::AboutInfo;

/// What the About page says.
pub fn about() -> AboutInfo {
    AboutInfo {
        name: "FreeSynergy.Whiteboard".into(),
        version: format!("v{}", env!("CARGO_PKG_VERSION")),
        origin: "catalog".into(),
        licence: env!("CARGO_PKG_LICENSE").into(),
        repository: env!("CARGO_PKG_REPOSITORY").into(),
    }
}
