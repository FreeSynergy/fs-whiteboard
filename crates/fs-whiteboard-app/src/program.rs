//! The window as a command: what `fs-whiteboard` and `fs-whiteboard app` run (ADR 0063).
//!
//! Pattern: Facade — open the store, make the core, build the window with one admin view per data type, launch. The
//! runtime lives as long as the window: the backends call the core on it.

use std::sync::Arc;

use fs_admin_frontend::view::AdminSurface;
use fs_admin_frontend::window::{TypeView, admin_window};
use fs_admin_frontend::{Backend, Session, SystemClock};
use fs_components_app::{AppWidget, WindowOptions, launch};
use fs_components_fields::{Look, Say};
use fs_components_styles::StyleId;
use fs_objects::{Store, TursoStore};
use fs_types::DescribesType;
use fs_whiteboard_core::{Board, Element, Page, WhiteboardRepo};

use crate::about::about;
use crate::args::{HELP, parse};
use crate::backend::{BoardBackend, ElementBackend, PageBackend};
use crate::core::Core;
use crate::words::{TITLE, Words};

/// Who acts on this store. Provisional: there is no sign-in yet.
const ACTOR: &str = "https://localhost/actors/owner";
/// The instance of the IRIs. Provisional, with the actor.
const INSTANCE: &str = "localhost";
/// Where the help is.
const HELP_URL: &str = "https://github.com/FreeSynergy/fs-whiteboard#readme";

/// Open the window with `argv`, the words after `app`; the exit code.
#[must_use]
pub fn app_command(argv: &[String]) -> i32 {
    match run(argv) {
        Ok(()) => 0,
        Err(message) => {
            eprintln!("fs-whiteboard: {message}");
            1
        }
    }
}

/// Where the boards are kept: `--store`, else the data folder (`$XDG_DATA_HOME`, on Windows `%LOCALAPPDATA%`, else
/// `~/.local/share`, then `freesynergy/whiteboard/app.db`), so a program started from the store never loses data by default.
fn store_file(store: Option<std::path::PathBuf>) -> Result<std::path::PathBuf, String> {
    if let Some(file) = store {
        return Ok(file);
    }
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .ok_or("cannot find a data folder (neither XDG_DATA_HOME, LOCALAPPDATA nor HOME is set); give --store <file>")?;
    let dir = base.join("freesynergy").join("whiteboard");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir.join("app.db"))
}

fn run(argv: &[String]) -> Result<(), String> {
    let options = parse(argv)?;
    if options.help {
        println!("{HELP}");
        return Ok(());
    }
    let file = store_file(options.store)?;
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let store: Arc<dyn Store> = runtime
        .block_on(TursoStore::open(&file))
        .map(|s| Arc::new(s) as Arc<dyn Store>)
        .map_err(|e| format!("cannot open the store {}: {e}", file.display()))?;
    let repo = WhiteboardRepo::new(store, ACTOR).with_instance(INSTANCE);
    let core = Core::new(repo, runtime.handle().clone(), INSTANCE);
    let app = build_window(&core).map_err(|e| e.to_string())?;
    launch(WindowOptions::default(), app).map_err(|e| e.to_string())
}

fn view(
    say: &Arc<dyn Say>,
    id: &str,
    nav: &str,
    description: fs_types::TypeDescription,
    backend: Arc<dyn Backend>,
) -> TypeView {
    let session = Session::new(description, backend, Arc::new(SystemClock::new()));
    TypeView {
        id: id.to_owned(),
        label: nav.to_owned(),
        surface: Arc::new(AdminSurface::new(
            session,
            Look::new(StyleId::Admin, false),
            Arc::clone(say),
            None,
        )),
    }
}

/// The window over `core`: one admin view per data type (Boards, Pages, Elements), the standard menus, About and Help.
///
/// # Errors
/// [`fs_components_app::AppError`] if the shell cannot be made.
pub fn build_window(core: &Arc<Core>) -> Result<AppWidget, fs_components_app::AppError> {
    let say: Arc<dyn Say> = Arc::new(Words::new());
    let views = vec![
        view(
            &say,
            "Board",
            "nav.board",
            Board::describe(),
            Arc::new(BoardBackend::new(Arc::clone(core))),
        ),
        view(
            &say,
            "Page",
            "nav.page",
            Page::describe(),
            Arc::new(PageBackend::new(Arc::clone(core))),
        ),
        view(
            &say,
            "Element",
            "nav.element",
            Element::describe(),
            Arc::new(ElementBackend::new(Arc::clone(core))),
        ),
    ];
    admin_window(TITLE, &say, views, about(), HELP_URL.to_owned())
}
