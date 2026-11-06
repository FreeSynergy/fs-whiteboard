//! The [`BoardEditor`] contract: everything one person does on one board.
//!
//! Pattern: Command (interface) + Null Object — each edit is undoable by its editor alone. The
//! reading methods are required; every other method answers `Unsupported` unless a provider
//! overrides it. The Rust form of `BoardEditor` in `whiteboard.capnp`; all ids must be on this board.

use async_trait::async_trait;
use fs_store::{Blob, ObjectId};

use crate::error::Result;
use crate::model::{Align, Board, ConnectorEnd, Cursor, Element, Paged, Point, Rect, StyleField};
use crate::reading::End;
use crate::unsupported::not_possible;

/// An edit of a locked element is refused, other than `set_locked`.
#[async_trait]
pub trait BoardEditor: Send + Sync {
    /// `board.open`: the board with its pages.
    async fn get(&self) -> Result<Board>;
    /// `board.open`: one element (also a deleted one); a group has the union of its members as frame.
    async fn get_element(&self, id: ObjectId) -> Result<Option<Element>>;
    /// `board.view`: not deleted, not hidden elements of a page, back to front, optionally only those
    /// touching `area`.
    async fn elements(
        &self,
        page: ObjectId,
        area: Option<Rect>,
        cursor: Option<&Cursor>,
    ) -> Result<Paged<Element>>;
    /// `canvas.navigate`: the area all shown elements of a page cover.
    async fn bounds(&self, page: ObjectId) -> Result<Option<Rect>>;
    /// Where the ends of a connector are (not a contract method: a reading rule made available).
    async fn connector_ends(&self, id: ObjectId) -> Result<(End, End)>;

    /// `page.add`: a new last page.
    async fn add_page(&self, _name: &str) -> Result<(Board, ObjectId)> {
        not_possible("page.add")
    }
    /// `page.remove`: not the last one.
    async fn remove_page(&self, _page: ObjectId) -> Result<Board> {
        not_possible("page.remove")
    }
    /// `page.add`: place a page before another; `None` = last.
    async fn move_page(&self, _page: ObjectId, _before: Option<ObjectId>) -> Result<Board> {
        not_possible("page.add")
    }
    /// `shape.add` and the other add capabilities: id and `order` are ignored; a new element goes to
    /// the front of its page.
    async fn add(&self, _element: Element) -> Result<Element> {
        not_possible("shape.add")
    }
    /// `element.move`: by `(dx, dy)`; a group moves with its members, each once.
    async fn move_by(&self, _ids: &[ObjectId], _dx: f64, _dy: f64) -> Result<()> {
        not_possible("element.move")
    }
    /// `element.move`: position and size.
    async fn set_frame(&self, _id: ObjectId, _frame: Rect) -> Result<()> {
        not_possible("element.move")
    }
    /// `element.move`: radians, around the middle of the frame.
    async fn rotate(&self, _id: ObjectId, _angle: f64) -> Result<()> {
        not_possible("element.move")
    }
    /// `element.style`: changes only the properties named.
    async fn set_style(&self, _ids: &[ObjectId], _fields: &[StyleField]) -> Result<()> {
        not_possible("element.style")
    }
    /// `text.edit`: text, note, area, connector label, or the label of a shape.
    async fn set_text(&self, _id: ObjectId, _text: &str) -> Result<()> {
        not_possible("text.edit")
    }
    /// `connector.edit`: one end; `None` element = free at `point`.
    async fn set_end(
        &self,
        _id: ObjectId,
        _end: ConnectorEnd,
        _element: Option<ObjectId>,
        _point: Point,
    ) -> Result<()> {
        not_possible("connector.edit")
    }
    /// `element.lock`.
    async fn set_locked(&self, _ids: &[ObjectId], _locked: bool) -> Result<()> {
        not_possible("element.lock")
    }
    /// `element.delete`, `stroke.erase`: marks as deleted; a group hides its members.
    async fn delete(&self, _ids: &[ObjectId]) -> Result<()> {
        not_possible("element.delete")
    }
    /// `element.move`: copies (members included) moved by `(dx, dy)`; connectors between copied
    /// elements point at the copies. Returns the copies of the elements named.
    async fn copy(&self, _ids: &[ObjectId], _dx: f64, _dy: f64) -> Result<Vec<ObjectId>> {
        not_possible("element.move")
    }
    /// `element.group`: refused if it would form a cycle or a member is locked.
    async fn group(&self, _ids: &[ObjectId]) -> Result<ObjectId> {
        not_possible("element.group")
    }
    /// `element.ungroup`: the members go to the group's parent; returns them.
    async fn ungroup(&self, _id: ObjectId) -> Result<Vec<ObjectId>> {
        not_possible("element.ungroup")
    }
    /// `element.reorder`: directly behind `before`, keeping their order; `None` = at the front.
    async fn reorder(&self, _ids: &[ObjectId], _before: Option<ObjectId>) -> Result<()> {
        not_possible("element.reorder")
    }
    /// `element.align`.
    async fn align(&self, _ids: &[ObjectId], _how: Align) -> Result<()> {
        not_possible("element.align")
    }
    /// `history.undo`: takes back this editor's last edit, except properties someone else changed
    /// since. `false` when nothing was left to do.
    async fn undo(&self) -> Result<bool> {
        not_possible("history.undo")
    }
    /// `history.redo`.
    async fn redo(&self) -> Result<bool> {
        not_possible("history.redo")
    }
    /// `board.export`, `board.save`: the own format or `image/svg+xml`. Empty `ids` = the whole page;
    /// `page` `None` = the whole board, own format only.
    async fn export(
        &self,
        _page: Option<ObjectId>,
        _ids: &[ObjectId],
        _media_type: &str,
    ) -> Result<Blob> {
        not_possible("board.export")
    }
    /// `element.group`: move to a page and a parent; `None` parent = directly on the page. Refused if
    /// it would form a cycle.
    async fn set_parent(
        &self,
        _ids: &[ObjectId],
        _page: Option<ObjectId>,
        _parent: Option<ObjectId>,
    ) -> Result<()> {
        not_possible("element.group")
    }
}
