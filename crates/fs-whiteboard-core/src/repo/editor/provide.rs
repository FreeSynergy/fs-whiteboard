//! `Editor` as a provider of `BoardEditor`.
//!
//! Pattern: Adapter — each trait method calls the inherent method of the same concern, which lives
//! in the file of its feature.

use async_trait::async_trait;
use fs_objects::{Blob, ObjectId};

use super::handle::Editor;
use crate::error::Result;
use crate::model::{Align, Board, ConnectorEnd, Cursor, Element, Paged, Point, Rect, StyleField};
use crate::traits::BoardEditor;

#[async_trait]
impl BoardEditor for Editor {
    async fn get(&self) -> Result<Board> {
        Editor::get(self).await
    }
    async fn get_element(&self, id: ObjectId) -> Result<Option<Element>> {
        Editor::get_element(self, id).await
    }
    async fn elements(
        &self,
        page: ObjectId,
        area: Option<Rect>,
        cursor: Option<&Cursor>,
    ) -> Result<Paged<Element>> {
        Editor::elements(self, page, area, cursor).await
    }
    async fn bounds(&self, page: ObjectId) -> Result<Option<Rect>> {
        Editor::bounds(self, page).await
    }
    async fn add_page(&self, name: &str) -> Result<(Board, ObjectId)> {
        Editor::add_page(self, name).await
    }
    async fn remove_page(&self, page: ObjectId) -> Result<Board> {
        Editor::remove_page(self, page).await
    }
    async fn move_page(&self, page: ObjectId, before: Option<ObjectId>) -> Result<Board> {
        Editor::move_page(self, page, before).await
    }
    async fn add(&self, element: Element) -> Result<Element> {
        Editor::add(self, element).await
    }
    async fn move_by(&self, ids: &[ObjectId], dx: f64, dy: f64) -> Result<()> {
        Editor::move_by(self, ids, dx, dy).await
    }
    async fn set_frame(&self, id: ObjectId, frame: Rect) -> Result<()> {
        Editor::set_frame(self, id, frame).await
    }
    async fn rotate(&self, id: ObjectId, angle: f64) -> Result<()> {
        Editor::rotate(self, id, angle).await
    }
    async fn set_style(&self, ids: &[ObjectId], fields: &[StyleField]) -> Result<()> {
        Editor::set_style(self, ids, fields).await
    }
    async fn set_text(&self, id: ObjectId, text: &str) -> Result<()> {
        Editor::set_text(self, id, text).await
    }
    async fn set_end(
        &self,
        id: ObjectId,
        end: ConnectorEnd,
        element: Option<ObjectId>,
        point: Point,
    ) -> Result<()> {
        Editor::set_end(self, id, end, element, point).await
    }
    async fn set_locked(&self, ids: &[ObjectId], locked: bool) -> Result<()> {
        Editor::set_locked(self, ids, locked).await
    }
    async fn delete(&self, ids: &[ObjectId]) -> Result<()> {
        Editor::delete(self, ids).await
    }
    async fn copy(&self, ids: &[ObjectId], dx: f64, dy: f64) -> Result<Vec<ObjectId>> {
        Editor::copy(self, ids, dx, dy).await
    }
    async fn group(&self, ids: &[ObjectId]) -> Result<ObjectId> {
        Editor::group(self, ids).await
    }
    async fn ungroup(&self, id: ObjectId) -> Result<Vec<ObjectId>> {
        Editor::ungroup(self, id).await
    }
    async fn reorder(&self, ids: &[ObjectId], before: Option<ObjectId>) -> Result<()> {
        Editor::reorder(self, ids, before).await
    }
    async fn align(&self, ids: &[ObjectId], how: Align) -> Result<()> {
        Editor::align(self, ids, how).await
    }
    async fn undo(&self) -> Result<bool> {
        Editor::undo(self).await
    }
    async fn redo(&self) -> Result<bool> {
        Editor::redo(self).await
    }
    async fn export(
        &self,
        page: Option<ObjectId>,
        ids: &[ObjectId],
        media_type: &str,
    ) -> Result<Blob> {
        Editor::export(self, page, ids, media_type).await
    }
    async fn set_parent(
        &self,
        ids: &[ObjectId],
        page: Option<ObjectId>,
        parent: Option<ObjectId>,
    ) -> Result<()> {
        Editor::set_parent(self, ids, page, parent).await
    }
}
