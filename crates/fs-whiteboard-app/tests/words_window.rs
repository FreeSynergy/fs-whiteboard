//! The words and the window: every key of the descriptions has English text; the window has a view per type.

mod common;

use std::collections::BTreeSet;

use fs_components_fields::Say;
use fs_types::DescribesType;
use fs_whiteboard_app::{Words, about, build_window, keys};
use fs_whiteboard_core::{Board, Element, Page};

fn all_labels() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for d in [Board::describe(), Page::describe(), Element::describe()] {
        out.insert(d.label.clone());
        for f in d.fields.iter().chain(
            d.commands
                .iter()
                .flat_map(|c| c.input.iter().chain(&c.result)),
        ) {
            out.insert(f.label.clone());
            out.extend(f.choices.iter().map(|c| format!("{}.{c}", f.label)));
        }
        out.extend(d.commands.iter().map(|c| c.label.clone()));
    }
    out
}

#[test]
fn every_label_of_every_description_has_words() {
    let words = Words::new();
    for key in all_labels() {
        assert_ne!(words.say(&key), key, "no words for {key}");
    }
}

#[test]
fn the_keys_the_frontend_asks_for_have_words() {
    let words = Words::new();
    for key in keys() {
        assert_ne!(words.say(key), key, "no words for {key}");
    }
    assert_eq!(
        words.say("no.such.key"),
        "no.such.key",
        "a gap shows as the key"
    );
}

#[test]
fn about_names_the_program() {
    let a = about();
    assert_eq!(a.name, "FreeSynergy.Whiteboard");
    assert!(a.version.starts_with('v'));
    assert_eq!(a.licence, "MIT OR Apache-2.0");
    assert_eq!(a.repository, "https://github.com/FreeSynergy/fs-whiteboard");
}

#[test]
fn the_window_has_a_view_per_data_type() {
    let w = common::world();
    let app = build_window(&w.core).expect("window");
    app.with_shell(|s| {
        let view: Vec<&str> = s
            .menus()
            .iter()
            .filter(|m| m.label == "View")
            .flat_map(|m| m.entries.iter().map(|e| e.label.as_str()))
            .collect();
        assert_eq!(view, ["Boards", "Pages", "Elements"]);
    });
}

#[test]
fn a_session_of_the_frontend_lists_the_records_of_the_core() {
    use fs_admin_frontend::{Backend, Session, SystemClock};
    use fs_whiteboard_app::BoardBackend;
    let w = common::world();
    let (_, _) = common::board_with_page(&w);
    let backend: std::sync::Arc<dyn Backend> =
        std::sync::Arc::new(BoardBackend::new(std::sync::Arc::clone(&w.core)));
    let mut session = Session::new(
        Board::describe(),
        backend,
        std::sync::Arc::new(SystemClock::new()),
    );
    session.reload();
    assert!(!session.is_stale());
    assert_eq!(session.list().rows().len(), 1);
}
