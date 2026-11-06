//! [`BoardDoc`]: the name and pages of a board with the clock of each unit.
//!
//! Pattern: Value object — merge units of a board are its `name` and, per page, `name`, `order` and
//! `deleted`. Pages are never dropped (a removal is `deleted`), so the set of pages only grows.

use std::collections::BTreeMap;

use fs_store::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::to_value;

use crate::clock::Clock;
use crate::model::Page;

/// A board and the clocks of its units. Keys: `name`, `<page id>/name`, `/order`, `/deleted`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardDoc {
    /// Display name of the board.
    pub name: String,
    /// All pages ever added, by id.
    pub pages: BTreeMap<ObjectId, Page>,
    /// Clock of each unit.
    #[serde(rename = "fs:clocks")]
    pub clocks: BTreeMap<String, Clock>,
}

impl BoardDoc {
    /// A new board written by `actor`, with one first page.
    pub fn fresh(name: &str, first: Page, actor: &str) -> Self {
        let mut doc = Self {
            name: name.to_string(),
            pages: BTreeMap::new(),
            clocks: BTreeMap::new(),
        };
        doc.clocks.insert("name".into(), Clock::after(0, actor));
        doc.add_page(first, actor);
        doc
    }

    /// The highest counter seen on this object.
    pub fn highest(&self) -> u64 {
        self.clocks.values().map(Clock::counter).max().unwrap_or(0)
    }

    /// The clock of a unit; `(0, "")` if never written.
    pub fn clock(&self, key: &str) -> Clock {
        self.clocks.get(key).cloned().unwrap_or_default()
    }

    /// Renames the board.
    pub fn set_name(&mut self, name: &str, actor: &str) {
        self.name = name.to_string();
        let c = Clock::after(self.highest(), actor);
        self.clocks.insert("name".into(), c);
    }

    /// Adds a page.
    pub fn add_page(&mut self, page: Page, actor: &str) {
        let c = Clock::after(self.highest(), actor);
        for part in ["name", "order", "deleted"] {
            self.clocks.insert(format!("{}/{part}", page.id), c.clone());
        }
        self.pages.insert(page.id, page);
    }

    /// Changes one unit of a page (`name`, `order` or `deleted`) to the value `change` sets.
    pub fn edit_page(
        &mut self,
        id: ObjectId,
        part: &str,
        actor: &str,
        change: impl FnOnce(&mut Page),
    ) {
        if let Some(p) = self.pages.get_mut(&id) {
            change(p);
            let c = Clock::after(self.highest(), actor);
            self.clocks.insert(format!("{id}/{part}"), c);
        }
    }

    /// Text of a page's unit value, for comparing candidates in a merge.
    pub(crate) fn page_value(p: &Page, part: &str) -> String {
        let v = match part {
            "name" => to_value(&p.name),
            "order" => to_value(&p.order),
            _ => to_value(p.deleted),
        };
        v.map(|v| v.to_string()).unwrap_or_default()
    }
}
