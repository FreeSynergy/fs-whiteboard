//! The backends against the real core, part two: undo, formats, bounds, refusals.

mod common;

use common::{board_with_page, id_of, shape, v, world};
use fs_admin_frontend::{Backend, BackendError, Values};

#[test]
fn undo_takes_back_the_last_edit_of_this_session() {
    let w = world();
    let (board, page) = board_with_page(&w);
    let el = shape(&w, &page, "0");
    let out = w.boards.run("undo", &v(&[("id", &board)])).unwrap();
    assert_eq!(out.values["done"], ["yes"]);
    assert!(
        !w.elements
            .list(&Values::new())
            .unwrap()
            .iter()
            .any(|r| r.id == el)
    );
    w.boards.run("redo", &v(&[("id", &board)])).unwrap();
    assert!(
        w.elements
            .list(&Values::new())
            .unwrap()
            .iter()
            .any(|r| r.id == el)
    );
}

#[test]
fn export_and_import_round_trip_and_svg_is_svg() {
    let w = world();
    let (board, page) = board_with_page(&w);
    shape(&w, &page, "0");
    let own = w
        .boards
        .run("export", &v(&[("id", &board), ("format", "whiteboard")]))
        .unwrap();
    let svg = w
        .boards
        .run(
            "export",
            &v(&[("id", &board), ("page", &page), ("format", "svg")]),
        )
        .unwrap();
    assert!(svg.values["data"][0].contains("<svg"));
    let imported = w
        .boards
        .run(
            "import",
            &v(&[("format", "whiteboard"), ("data", &own.values["data"][0])]),
        )
        .unwrap();
    assert_ne!(id_of(&imported), board, "an import gets new ids");
    assert_eq!(w.boards.list(&Values::new()).unwrap().len(), 2);
}

#[test]
fn bounds_of_a_page_with_a_shape() {
    let w = world();
    let (_, page) = board_with_page(&w);
    shape(&w, &page, "10");
    let out = w.pages.run("bounds", &v(&[("id", &page)])).unwrap();
    assert_eq!(out.values["x"], ["10"]);
}

#[test]
fn commands_the_core_cannot_do_are_refused_by_name_and_described() {
    let w = world();
    let refused = w.boards.run(
        "share",
        &v(&[("id", "https://example.org/whiteboard/x"), ("role", "view")]),
    );
    assert_eq!(
        refused,
        Err(BackendError::Unavailable("unsupported:share".into()))
    );
    assert!(!w.boards.supported("share"));
    assert!(w.boards.supported("create"));
    assert_eq!(
        w.boards.run("nonsense", &Values::new()),
        Err(BackendError::NotFound)
    );
}

#[test]
fn bad_input_is_rejected_with_the_field_not_run() {
    let w = world();
    let (_, page) = board_with_page(&w);
    let bad = v(&[
        ("page", &page),
        ("kind", "shape"),
        ("x", "far"),
        ("y", "0"),
        ("width", "1"),
        ("height", "1"),
    ]);
    match w.elements.run("add", &bad) {
        Err(BackendError::Rejected(issues)) => assert_eq!(issues[0].field, "x"),
        other => panic!("{other:?}"),
    }
    let no_kind = v(&[
        ("page", &page),
        ("x", "0"),
        ("y", "0"),
        ("width", "1"),
        ("height", "1"),
    ]);
    assert!(matches!(
        w.elements.run("add", &no_kind),
        Err(BackendError::Rejected(_))
    ));
    assert!(matches!(
        w.elements.run("add", &v(&[("page", "not an iri")])),
        Err(BackendError::Rejected(_))
    ));
    assert!(w.elements.list(&Values::new()).unwrap().is_empty());
}

#[test]
fn a_missing_record_is_not_found() {
    let w = world();
    let ghost = "https://example.org/whiteboard/018f0000-0000-7000-8000-000000000000";
    assert_eq!(
        w.elements.run("delete", &v(&[("id", ghost)])),
        Err(BackendError::NotFound)
    );
    assert_eq!(
        w.boards.run("rename", &v(&[("id", ghost), ("name", "x")])),
        Err(BackendError::NotFound)
    );
}

#[test]
fn board_iri_is_the_one_the_core_gives() {
    let w = world();
    let (board, _) = board_with_page(&w);
    let core_board = w.core.boards().unwrap().remove(0);
    assert_eq!(core_board.iri, board);
    assert_eq!(fs_whiteboard_app::iri("example.org", core_board.id), board);
    assert_eq!(fs_whiteboard_app::parse_iri(&board), Some(core_board.id));
}
