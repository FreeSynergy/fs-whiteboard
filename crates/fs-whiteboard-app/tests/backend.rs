//! The backends against the real core: what the admin frontend would do, without a window.

mod common;

use common::{board_with_page, id_of, shape, v, world};
use fs_admin_frontend::{Backend, BackendError, Values};

fn field(r: &fs_admin_frontend::Record, name: &str) -> String {
    r.values
        .get(name)
        .and_then(|v| v.first())
        .cloned()
        .unwrap_or_default()
}

#[test]
fn a_board_is_created_listed_renamed_and_deleted() {
    let w = world();
    let (board, _) = board_with_page(&w);
    let list = w.boards.list(&Values::new()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, board);
    assert_eq!(field(&list[0], "name"), "Roof plan");
    w.boards
        .run("rename", &v(&[("id", &board), ("name", "Garden")]))
        .unwrap();
    assert_eq!(
        field(&w.boards.list(&Values::new()).unwrap()[0], "name"),
        "Garden"
    );
    w.boards.run("delete", &v(&[("id", &board)])).unwrap();
    assert!(w.boards.list(&Values::new()).unwrap().is_empty());
}

#[test]
fn pages_are_added_moved_and_removed_and_the_last_one_stays() {
    let w = world();
    let (board, first) = board_with_page(&w);
    let second = id_of(
        &w.pages
            .run("add", &v(&[("board", &board), ("name", "Two")]))
            .unwrap(),
    );
    assert_eq!(w.pages.list(&Values::new()).unwrap().len(), 2);
    w.pages
        .run("move", &v(&[("id", &second), ("before", &first)]))
        .unwrap();
    let order: Vec<String> = w
        .pages
        .list(&Values::new())
        .unwrap()
        .into_iter()
        .map(|p| p.id)
        .collect();
    assert_eq!(order, [second.clone(), first.clone()]);
    w.pages.run("remove", &v(&[("id", &second)])).unwrap();
    assert!(matches!(
        w.pages.run("remove", &v(&[("id", &first)])),
        Err(BackendError::Unavailable(_))
    ));
    assert_eq!(w.pages.list(&v(&[("board", &board)])).unwrap().len(), 1);
}

#[test]
fn an_element_is_added_listed_and_changed_partly() {
    let w = world();
    let (_, page) = board_with_page(&w);
    let el = shape(&w, &page, "10");
    let rec = w.elements.list(&v(&[("page", &page)])).unwrap().remove(0);
    assert_eq!(
        (
            rec.id.as_str(),
            field(&rec, "kind").as_str(),
            field(&rec, "x").as_str()
        ),
        (el.as_str(), "shape", "10")
    );
    // a change sends only what changes: x stays, width is new
    w.elements
        .run("setFrame", &v(&[("id", &el), ("width", "200")]))
        .unwrap();
    let rec = w.elements.list(&Values::new()).unwrap().remove(0);
    assert_eq!(
        (
            field(&rec, "x").as_str(),
            field(&rec, "width").as_str(),
            field(&rec, "height").as_str()
        ),
        ("10", "200", "40")
    );
    w.elements
        .run("move", &v(&[("id", &el), ("dx", "5"), ("dy", "7")]))
        .unwrap();
    let rec = w.elements.list(&Values::new()).unwrap().remove(0);
    assert_eq!(
        (field(&rec, "x").as_str(), field(&rec, "y").as_str()),
        ("15", "7")
    );
    w.elements
        .run("rotate", &v(&[("id", &el), ("angle", "1.5")]))
        .unwrap();
    assert_eq!(
        field(&w.elements.list(&Values::new()).unwrap()[0], "angle"),
        "1.5"
    );
}

#[test]
fn style_changes_only_what_is_named_and_an_empty_list_resets() {
    let w = world();
    let (_, page) = board_with_page(&w);
    let el = shape(&w, &page, "0");
    w.elements
        .run(
            "setStyle",
            &v(&[("id", &el), ("fill", "#ff0000"), ("lineWidth", "4")]),
        )
        .unwrap();
    let rec = w.elements.list(&Values::new()).unwrap().remove(0);
    assert_eq!(
        (
            field(&rec, "fill").as_str(),
            field(&rec, "lineWidth").as_str(),
            field(&rec, "stroke").as_str()
        ),
        ("#ff0000", "4", "")
    );
    let mut clear = v(&[("id", &el)]);
    clear.insert("fill".into(), vec![]);
    w.elements.run("setStyle", &clear).unwrap();
    let rec = w.elements.list(&Values::new()).unwrap().remove(0);
    assert_eq!(
        (
            field(&rec, "fill").as_str(),
            field(&rec, "lineWidth").as_str()
        ),
        ("", "4")
    );
}

#[test]
fn text_connector_group_align_lock_copy_reorder_and_delete() {
    let w = world();
    let (_, page) = board_with_page(&w);
    let (a, b) = (shape(&w, &page, "0"), shape(&w, &page, "200"));
    let note = id_of(
        &w.elements
            .run(
                "add",
                &v(&[
                    ("page", &page),
                    ("kind", "note"),
                    ("text", "Hi"),
                    ("x", "0"),
                    ("y", "100"),
                    ("width", "50"),
                    ("height", "50"),
                ]),
            )
            .unwrap(),
    );
    w.elements
        .run("setText", &v(&[("id", &note), ("text", "Hello")]))
        .unwrap();
    let conn = id_of(
        &w.elements
            .run(
                "add",
                &v(&[
                    ("page", &page),
                    ("kind", "connector"),
                    ("source", &a),
                    ("target", &b),
                    ("arrowEnd", "yes"),
                    ("x", "0"),
                    ("y", "0"),
                    ("width", "1"),
                    ("height", "1"),
                ]),
            )
            .unwrap(),
    );
    w.elements
        .run(
            "setEnd",
            &v(&[("id", &conn), ("end", "target"), ("x", "9"), ("y", "9")]),
        )
        .unwrap();
    let group = id_of(
        &w.elements
            .run("group", &v(&[("ids", &a), ("ids", &b)]))
            .unwrap(),
    );
    w.elements.run("ungroup", &v(&[("id", &group)])).unwrap();
    w.elements
        .run("align", &v(&[("ids", &a), ("ids", &b), ("how", "left")]))
        .unwrap();
    w.elements.run("reorder", &v(&[("id", &a)])).unwrap();
    let copy = w
        .elements
        .run("copy", &v(&[("id", &a), ("dx", "10"), ("dy", "10")]))
        .unwrap();
    assert!(copy.values["copy"][0].starts_with("https://example.org/whiteboard/"));
    w.elements
        .run("setLocked", &v(&[("id", &b), ("locked", "yes")]))
        .unwrap();
    assert!(matches!(
        w.elements
            .run("move", &v(&[("id", &b), ("dx", "1"), ("dy", "1")])),
        Err(BackendError::Unavailable(_))
    ));
    w.elements.run("delete", &v(&[("id", &a)])).unwrap();
    assert!(
        !w.elements
            .list(&Values::new())
            .unwrap()
            .iter()
            .any(|r| r.id == a)
    );
}
