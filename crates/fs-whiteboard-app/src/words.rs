//! The words of the program: the English table behind the translation keys.
//!
//! Pattern: Strategy behind `Say` (from `fs-components-fields`) — the window asks for keys, this is the English
//! answer; translation (ADR 0053) replaces this file's content, not its users. The table has three parts: the keys the
//! admin frontend asks for (`FRONTEND`), the names of types and commands (`NAMES`), and the labels of fields, which
//! are made from the last part of their key (`line-width` becomes "Line width") unless `NAMES` says otherwise. A key
//! nobody wrote comes back as the key itself, so a gap shows.

use std::collections::BTreeMap;

use fs_components_fields::Say;
use fs_whiteboard_core::describe::registry;

/// The title of the window.
pub const TITLE: &str = "FreeSynergy Whiteboard";

/// The keys the admin frontend asks for (`fs-admin-frontend/DESIGN.md`), and the codes of its messages and problems.
const FRONTEND: &[(&str, &str)] = &[
    ("action.add", "Add"),
    ("action.back", "Back"),
    ("action.cancel", "Cancel"),
    ("action.close", "Close"),
    ("action.remove", "Remove"),
    ("action.save", "Save"),
    ("menu.program", "Program"),
    ("menu.view", "View"),
    ("menu.help", "Help"),
    ("menu.quit", "Quit"),
    ("menu.about", "About"),
    ("message.done", "Done."),
    ("message.rejected", "The input was not accepted."),
    ("message.not-found", "Not found."),
    ("message.unavailable", "The whiteboard could not do that."),
    ("message.unsaved", "There is text that is not saved yet."),
    ("sort.ascending", "ascending"),
    ("sort.descending", "descending"),
    ("finding.slot", "Value {n}"),
    ("list.empty", "Nothing here yet."),
    ("list.separator", ", "),
    ("list.stale", "Old: the whiteboard could not be read again."),
    ("detail.not-loaded", "This record is not loaded."),
    ("confirm.sure", "Are you sure?"),
    ("value.none", "none"),
    ("command.soon", "coming soon"),
    ("about.version", "Version"),
    ("about.origin", "Origin"),
    ("about.licence", "Licence"),
    ("about.repository", "Repository"),
    ("about.unoffered", "Not offered"),
    ("problem.missing", "This is needed."),
    ("problem.not-multiple", "Only one value is allowed."),
    ("problem.unknown-field", "This field does not exist."),
    ("problem.empty", "Nothing typed."),
    ("problem.too-long", "This is too long."),
    ("problem.character", "This is not a number."),
    ("problem.uri.bad", "This is not a valid reference."),
    (
        "problem.uri.not-absolute",
        "A reference needs a scheme such as https:.",
    ),
    (
        "problem.uri.unsafe-scheme",
        "This kind of address is not allowed.",
    ),
    (
        "problem.choice.not-in-list",
        "Pick one of the listed entries.",
    ),
];

/// Names of types and commands, and labels that the last part of the key would not say well.
const NAMES: &[(&str, &str)] = &[
    ("nav.board", "Boards"),
    ("nav.page", "Pages"),
    ("nav.element", "Elements"),
    ("board", "Board"),
    ("page", "Page"),
    ("element", "Element"),
    ("page.add.board", "Board"),
    ("element.move.dx", "Move right by"),
    ("element.move.dy", "Move down by"),
    ("element.copy.dx", "Move the copy right by"),
    ("element.copy.dy", "Move the copy down by"),
    ("element.set-end.x", "Free end, x"),
    ("element.set-end.y", "Free end, y"),
    ("element.set-end.element", "Attach to element"),
    ("element.x", "X"),
    ("element.y", "Y"),
    ("page.bounds", "Area shown"),
    ("board.export.page", "Only this page"),
    ("board.export.data", "The file"),
    ("board.import.data", "The file"),
];

/// The English words for the keys of this program.
#[derive(Clone, Debug)]
pub struct Words {
    table: BTreeMap<String, String>,
}

/// `line-width` becomes `Line width`.
fn spoken(part: &str) -> String {
    let text = part.replace('-', " ");
    let mut chars = text.chars();
    chars
        .next()
        .map_or_else(String::new, |f| f.to_uppercase().chain(chars).collect())
}

impl Words {
    /// The table for the descriptions of this program.
    ///
    /// # Panics
    /// Never for this crate's own descriptions (a test registers them).
    pub fn new() -> Self {
        let mut table: BTreeMap<String, String> = BTreeMap::new();
        let reg = registry().expect("the descriptions of this crate are valid");
        for t in reg.iter() {
            let mut keys: Vec<&fs_types::FieldDescription> = t.fields.iter().collect();
            for c in &t.commands {
                table
                    .entry(c.label.clone())
                    .or_insert_with(|| spoken(c.label.rsplit('.').next().unwrap_or("")));
                keys.extend(c.input.iter().chain(&c.result));
            }
            for f in keys {
                // the placeholder of an input: a hint for the types that need one, empty (not the key) for the rest
                let hint = match f.basic {
                    fs_types::BasicType::Reference => {
                        "A reference, such as https://example.org/whiteboard/..."
                    }
                    fs_types::BasicType::Link => {
                        "An address, such as https://example.org/picture.png"
                    }
                    _ => "",
                };
                table
                    .entry(format!("hint.{}", f.label))
                    .or_insert_with(|| hint.to_owned());
                table
                    .entry(f.label.clone())
                    .or_insert_with(|| spoken(f.label.rsplit('.').next().unwrap_or("")));
                for choice in &f.choices {
                    table
                        .entry(format!("{}.{choice}", f.label))
                        .or_insert_with(|| spoken(choice));
                }
            }
        }
        for (k, v) in FRONTEND.iter().chain(NAMES) {
            table.insert((*k).to_owned(), (*v).to_owned());
        }
        Self { table }
    }

    /// Every key the table answers.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.table.keys().map(String::as_str)
    }
}

impl Default for Words {
    fn default() -> Self {
        Self::new()
    }
}

impl Say for Words {
    fn say(&self, key: &str) -> String {
        self.table
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_owned())
    }
}

/// The translation keys the window itself asks for (the part of the table the frontend needs).
pub fn keys() -> Vec<&'static str> {
    FRONTEND.iter().map(|(k, _)| *k).collect()
}
