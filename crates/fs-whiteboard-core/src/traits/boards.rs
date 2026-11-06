//! The [`Boards`] contract: create, open, rename, delete, list, import.
//!
//! Pattern: Repository (interface) + Null Object — callers use `dyn Boards`. Like a parent class
//! (ADR 0045): every provider has every method; one it does not override answers an explicit
//! `Unsupported`. The Rust form of `Boards` in `whiteboard.capnp`.

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_store::ObjectId;

use super::BoardEditor;
use crate::error::Result;
use crate::model::{Board, Cursor, Paged};
use crate::unsupported::not_possible;

/// The boards of a store. Interface status: draft (ADR 0045).
#[async_trait]
pub trait Boards: Send + Sync {
    /// What this provider can do, per capability id.
    fn capabilities(&self) -> Capabilities;
    /// `board.open`: boards in creation order, one page.
    async fn list(&self, cursor: Option<&Cursor>) -> Result<Paged<Board>>;
    /// `board.open`: the editor of a board, with an empty undo history; `None` if there is no such
    /// board. One editor is one person's session.
    async fn open(&self, id: ObjectId) -> Result<Option<Box<dyn BoardEditor>>>;

    /// `board.create`: a new board with one page.
    async fn create(&self, _name: &str) -> Result<Board> {
        not_possible("board.create")
    }
    /// `board.create`: change the name.
    async fn rename(&self, _id: ObjectId, _name: &str) -> Result<Board> {
        not_possible("board.create")
    }
    /// `board.delete`: the board with every element of it.
    async fn delete(&self, _id: ObjectId) -> Result<()> {
        not_possible("board.delete")
    }
    /// `board.import`: the own format, JSON, at most 32 MiB; other media types are `Unsupported`.
    async fn import(&self, _media_type: &str, _data: &[u8]) -> Result<Board> {
        not_possible("board.import")
    }
}
