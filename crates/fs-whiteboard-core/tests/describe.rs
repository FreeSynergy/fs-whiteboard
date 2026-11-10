//! Tests of the type descriptions against the trait `DescribesType`, the registry of `fs-types` and the table.

use std::collections::BTreeSet;

use fs_types::{BasicType, CommandKind, DescribesType};
use fs_whiteboard_core::describe::{self, BOARD, ELEMENT, PAGE, table};
use fs_whiteboard_core::{Board, Element, Page};

fn required_ids() -> BTreeSet<String> {
    let text = match std::env::var("FS_REQUIREMENTS") {
        Ok(path) => {
            // The live requirements.md: the ids in backticks in the first column of the Required and Optional tables.
            let md = std::fs::read_to_string(path).expect("FS_REQUIREMENTS must be readable");
            let md = md.split("## Not planned").next().unwrap().to_owned();
            let mut ids = BTreeSet::new();
            for line in md
                .lines()
                .filter(|l| l.starts_with('|') && !l.starts_with("|--"))
            {
                let first = line.split('|').nth(1).unwrap_or("");
                for part in first.split('`').skip(1).step_by(2) {
                    if part.chars().all(|c| c.is_ascii_lowercase() || c == '.') && !part.is_empty()
                    {
                        ids.insert(part.to_owned());
                    }
                }
            }
            return ids;
        }
        Err(_) => include_str!("requirements_ids.txt").to_owned(),
    };
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn table_has_every_requirement() {
    let known: BTreeSet<&str> = table()
        .iter()
        .flat_map(|r| r.capabilities.iter().copied())
        .collect();
    let missing: Vec<String> = required_ids()
        .into_iter()
        .filter(|id| !known.contains(id.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "capability ids of requirements.md missing from the table: {missing:?}"
    );
}

#[test]
fn table_names_no_id_the_requirements_lack() {
    let required = required_ids();
    for row in table() {
        for cap in row.capabilities {
            assert!(
                required.contains(*cap),
                "{}.{} names unknown capability {cap}",
                row.ty.name,
                row.name
            );
        }
    }
}

#[test]
fn supported_rows_are_what_the_core_reports() {
    // `Boards::capabilities` is the core's own report; a row marked supported may only carry ids it supports.
    use fs_whiteboard_core::Boards;
    let store = tokio::runtime::Runtime::new().unwrap().block_on(async {
        let s: std::sync::Arc<dyn fs_objects::Store> =
            std::sync::Arc::new(fs_objects::TursoStore::open_in_memory().await.unwrap());
        s
    });
    let caps =
        fs_whiteboard_core::WhiteboardRepo::new(store, "https://example.org/a").capabilities();
    for row in table().iter().filter(|r| r.supported) {
        for cap in row.capabilities {
            assert!(
                caps.is_supported(cap),
                "{}.{}: the core does not report {cap}",
                row.ty.name,
                row.name
            );
        }
    }
    // and the other way: what the core supports is carried by a supported command
    let carried: BTreeSet<&str> = table()
        .iter()
        .filter(|r| r.supported)
        .flat_map(|r| r.capabilities.iter().copied())
        .collect();
    // `board.subscribe` is a change stream of the core, not a request/answer command: its row is unsupported on purpose.
    for id in required_ids()
        .iter()
        .filter(|id| caps.is_supported(id) && *id != "board.subscribe")
    {
        assert!(
            carried.contains(id.as_str()),
            "the core supports {id}, no supported command carries it"
        );
    }
}

#[test]
fn every_unsupported_row_has_a_reason_and_every_supported_none() {
    for row in table() {
        assert_eq!(
            row.supported,
            row.reason.is_empty(),
            "{}.{}",
            row.ty.name,
            row.name
        );
    }
}

#[test]
fn describe_carries_every_row_supported_or_not() {
    for (ty, d) in [
        (BOARD, Board::describe()),
        (PAGE, Page::describe()),
        (ELEMENT, Element::describe()),
    ] {
        let want: Vec<&str> = table()
            .iter()
            .filter(|r| r.ty == ty)
            .map(|r| r.name)
            .collect();
        let got: Vec<&str> = d.commands.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(want, got, "{}", ty.name);
        assert_eq!(d.program, "fs-whiteboard");
        assert!(!d.standard.is_empty());
    }
}

#[test]
fn the_registry_accepts_all_three() {
    let r = describe::registry().expect("valid descriptions");
    assert_eq!(r.len(), 3);
    for name in ["Board", "Page", "Element"] {
        assert!(r.get(name).is_some(), "{name}");
    }
}

#[test]
fn every_type_can_be_listed_with_find() {
    for d in [Board::describe(), Page::describe(), Element::describe()] {
        let find = d.find_command("find").expect("find");
        assert_eq!(find.kind, CommandKind::Find);
        assert!(
            find.input.iter().all(|f| !f.required),
            "an empty query must list everything: {}",
            d.name
        );
    }
}

#[test]
fn record_commands_refer_to_the_own_type() {
    for (ty, d) in [
        (BOARD, Board::describe()),
        (PAGE, Page::describe()),
        (ELEMENT, Element::describe()),
    ] {
        for name in ["delete", "remove"] {
            if let Some(c) = d.find_command(name) {
                assert!(
                    c.input
                        .iter()
                        .any(|f| f.basic == BasicType::Reference && f.target == ty.name),
                    "{name}"
                );
            }
        }
        for c in d.commands.iter().filter(|c| c.kind == CommandKind::Change) {
            assert!(
                c.input
                    .iter()
                    .any(|f| f.basic == BasicType::Reference && f.target == ty.name),
                "{}.{} changes a record without naming it",
                ty.name,
                c.name
            );
        }
    }
}

#[test]
fn the_core_methods_are_all_described() {
    // Every editor / boards method of the traits has a command (names follow the methods).
    let element: BTreeSet<String> = Element::describe()
        .commands
        .into_iter()
        .map(|c| c.name)
        .collect();
    for m in [
        "add",
        "move",
        "setFrame",
        "rotate",
        "setStyle",
        "setText",
        "setEnd",
        "setLocked",
        "delete",
        "copy",
        "group",
        "ungroup",
        "reorder",
        "align",
        "setParent",
    ] {
        assert!(element.contains(m), "Element.{m}");
    }
    let page: BTreeSet<String> = Page::describe()
        .commands
        .into_iter()
        .map(|c| c.name)
        .collect();
    for m in ["add", "remove", "move", "bounds"] {
        assert!(page.contains(m), "Page.{m}");
    }
    let board: BTreeSet<String> = Board::describe()
        .commands
        .into_iter()
        .map(|c| c.name)
        .collect();
    for m in [
        "create", "rename", "delete", "import", "export", "undo", "redo", "find",
    ] {
        assert!(board.contains(m), "Board.{m}");
    }
}
