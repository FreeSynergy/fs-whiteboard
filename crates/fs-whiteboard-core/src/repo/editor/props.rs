//! Properties: style, text, connector ends, lock, delete.
//!
//! Pattern: Command — each is a list of unit changes under one new clock per element, so it merges
//! with other people's changes to other properties of the same element.

use fs_store::ObjectId;
use serde_json::{Value, to_value};

use super::handle::Editor;
use super::selection::unique;
use crate::error::{Error, Result};
use crate::model::{Anchor, ConnectorEnd, Kind, Point, StyleField};
use crate::repo::tx::Tx;
use crate::unit::Unit;
use crate::validate;

fn field(f: &StyleField) -> (Unit, Value) {
    let v = |r: serde_json::Result<Value>| r.unwrap_or_default();
    match f {
        StyleField::Stroke(s) => (Unit::StyleStroke, v(to_value(s))),
        StyleField::Fill(s) => (Unit::StyleFill, v(to_value(s))),
        StyleField::LineWidth(w) => (Unit::StyleLineWidth, v(to_value(w))),
        StyleField::Dashed(d) => (Unit::StyleDashed, v(to_value(d))),
        StyleField::Opacity(o) => (Unit::StyleOpacity, v(to_value(o))),
        StyleField::FontFamily(s) => (Unit::StyleFontFamily, v(to_value(s))),
        StyleField::FontSize(s) => (Unit::StyleFontSize, v(to_value(s))),
    }
}

impl Editor {
    /// One change of the same units on every element named; they must be editable.
    async fn each(
        &self,
        ids: &[ObjectId],
        editable: bool,
        changes: Vec<(Unit, Value)>,
    ) -> Result<()> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        for id in unique(ids) {
            if editable {
                tx.editable(st, id)?;
            } else if tx.el(st, id)?.deleted {
                return Err(Error::NotFound {
                    what: "element",
                    id,
                });
            }
            tx.set(st, id, changes.clone())?;
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }

    pub(super) async fn set_style(&self, ids: &[ObjectId], fields: &[StyleField]) -> Result<()> {
        fields.iter().try_for_each(validate::style_field)?;
        self.each(ids, true, fields.iter().map(field).collect())
            .await
    }

    pub(super) async fn set_locked(&self, ids: &[ObjectId], locked: bool) -> Result<()> {
        self.each(ids, false, vec![(Unit::Locked, Value::Bool(locked))])
            .await
    }

    pub(super) async fn delete(&self, ids: &[ObjectId]) -> Result<()> {
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        for id in unique(ids) {
            if tx.el(st, id)?.deleted {
                continue;
            }
            tx.editable(st, id)?;
            tx.set(st, id, vec![(Unit::Deleted, Value::Bool(true))])?;
        }
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }

    pub(super) async fn set_text(&self, id: ObjectId, text: &str) -> Result<()> {
        validate::text(text)?;
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        let unit = match &tx.editable(st, id)?.kind {
            Kind::Text { .. } | Kind::Note { .. } | Kind::Area { .. } | Kind::Connector(_) => {
                Unit::Text
            }
            Kind::Shape { .. } => Unit::Label,
            k => return Err(Error::invalid(format!("a {} has no text", k.name()))),
        };
        tx.set(st, id, vec![(unit, to_value(text)?)])?;
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }

    pub(super) async fn set_end(
        &self,
        id: ObjectId,
        end: ConnectorEnd,
        element: Option<ObjectId>,
        point: Point,
    ) -> Result<()> {
        validate::point(&point)?;
        let mut g = self.repo.boards.lock().await;
        let st = self.repo.state(&mut g, self.board).await?;
        let mut tx = Tx::new(&self.repo.actor);
        if !matches!(tx.editable(st, id)?.kind, Kind::Connector(_)) {
            return Err(Error::invalid("only a connector has ends"));
        }
        if let Some(target) = element {
            if target == id {
                return Err(Error::invalid("a connector cannot follow itself"));
            }
            if tx.el(st, target)?.deleted {
                return Err(Error::NotFound {
                    what: "element",
                    id: target,
                });
            }
        }
        let unit = if end == ConnectorEnd::Source {
            Unit::Source
        } else {
            Unit::Target
        };
        tx.set(st, id, vec![(unit, to_value(Anchor { element, point })?)])?;
        let edit = self.repo.commit(st, tx).await?;
        self.record(edit);
        Ok(())
    }
}
