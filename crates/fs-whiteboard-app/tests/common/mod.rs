//! A real core over an in-memory store, driven through the backends.
#![allow(dead_code)]

use std::sync::Arc;

use fs_admin_frontend::{Backend, Outcome, Values};
use fs_objects::{Store, TursoStore};
use fs_whiteboard_app::{BoardBackend, Core, ElementBackend, PageBackend};
use fs_whiteboard_core::WhiteboardRepo;

pub struct World {
    pub rt: tokio::runtime::Runtime,
    pub core: Arc<Core>,
    pub boards: BoardBackend,
    pub pages: PageBackend,
    pub elements: ElementBackend,
}

pub fn world() -> World {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store: Arc<dyn Store> = Arc::new(rt.block_on(TursoStore::open_in_memory()).unwrap());
    let repo =
        WhiteboardRepo::new(store, "https://example.org/people/ada").with_instance("example.org");
    let core = Core::new(repo, rt.handle().clone(), "example.org");
    World {
        boards: BoardBackend::new(Arc::clone(&core)),
        pages: PageBackend::new(Arc::clone(&core)),
        elements: ElementBackend::new(Arc::clone(&core)),
        core,
        rt,
    }
}

/// Typed values from `(field, value)` pairs; a value with `\n` stays one value.
pub fn v(pairs: &[(&str, &str)]) -> Values {
    let mut out = Values::new();
    for (k, t) in pairs {
        out.entry((*k).to_owned())
            .or_default()
            .push((*t).to_owned());
    }
    out
}

pub fn id_of(o: &Outcome) -> String {
    o.id.clone().expect("the command names the record")
}

/// A board with its first page: `(board iri, page iri)`.
pub fn board_with_page(w: &World) -> (String, String) {
    let b = id_of(
        &w.boards
            .run("create", &v(&[("name", "Roof plan")]))
            .unwrap(),
    );
    let pages = w.pages.list(&Values::new()).unwrap();
    assert_eq!(pages.len(), 1);
    (b, pages[0].id.clone())
}

pub fn shape(w: &World, page: &str, x: &str) -> String {
    id_of(
        &w.elements
            .run(
                "add",
                &v(&[
                    ("page", page),
                    ("kind", "shape"),
                    ("shape", "rectangle"),
                    ("x", x),
                    ("y", "0"),
                    ("width", "80"),
                    ("height", "40"),
                ]),
            )
            .unwrap(),
    )
}
