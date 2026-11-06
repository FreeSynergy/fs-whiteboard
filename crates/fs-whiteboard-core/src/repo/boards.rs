//! `Boards`: create, open, rename, delete, list and import.
//!
//! Pattern: Repository — a board is one `Collection` object holding its pages; every element of it
//! is an object of its own, removed with it.

use async_trait::async_trait;
use fs_interfaces::Capabilities;
use fs_store::{ObjectId, Query};

use super::editor::Editor;
use super::handle::WhiteboardRepo;
use crate::consts::{BOARD_CONTEXT, BOARD_TYPE};
use crate::doc::{BoardDoc, board_to_stored};
use crate::error::{Error, Result};
use crate::format;
use crate::model::{Board, Cursor, Page, Paged};
use crate::order;
use crate::traits::{BoardEditor, Boards};

#[async_trait]
impl Boards for WhiteboardRepo {
    fn capabilities(&self) -> Capabilities {
        Self::support()
    }

    async fn list(&self, cursor: Option<&Cursor>) -> Result<Paged<Board>> {
        let q = Query::new().of_type(BOARD_TYPE).in_context(BOARD_CONTEXT);
        let mut ids: Vec<ObjectId> = self
            .store
            .query(&q)
            .await?
            .iter()
            .map(|o| o.envelope.id)
            .collect();
        ids.sort();
        let after = cursor.and_then(|c| c.0.parse::<ObjectId>().ok());
        ids.retain(|id| after.is_none_or(|a| *id > a));
        let more = ids.len() > self.page_size;
        ids.truncate(self.page_size);
        let next = more
            .then(|| ids.last().map(|id| Cursor(id.to_string())))
            .flatten();
        let mut items = Vec::new();
        let mut guard = self.boards.lock().await;
        for id in ids {
            match self.state(&mut guard, id).await {
                Ok(st) => items.push(st.board(&self.iri(id))),
                Err(e) => self.report(id, &e.to_string()),
            }
        }
        Ok(Paged { items, next })
    }

    async fn open(&self, id: ObjectId) -> Result<Option<Box<dyn BoardEditor>>> {
        let mut guard = self.boards.lock().await;
        match self.state(&mut guard, id).await {
            Ok(_) => Ok(Some(Box::new(Editor::new(self.clone(), id)))),
            Err(Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn create(&self, name: &str) -> Result<Board> {
        let first = Page {
            id: ObjectId::new(),
            name: "Page 1".into(),
            order: order::between(None, None),
            deleted: false,
        };
        self.create_board(BoardDoc::fresh(non_blank(name)?, first, &self.actor))
            .await
    }

    async fn rename(&self, id: ObjectId, name: &str) -> Result<Board> {
        let name = non_blank(name)?;
        let mut guard = self.boards.lock().await;
        let st = self.state(&mut guard, id).await?;
        let mut doc = st.doc.clone();
        doc.set_name(name, &self.actor);
        self.save_board(id, &doc).await?;
        st.set_doc(doc);
        Ok(st.board(&self.iri(id)))
    }

    async fn delete(&self, id: ObjectId) -> Result<()> {
        let mut guard = self.boards.lock().await;
        self.state(&mut guard, id).await?;
        self.purge(&mut guard, id).await?;
        Ok(())
    }

    async fn import(&self, media_type: &str, data: &[u8]) -> Result<Board> {
        let imported = format::import(media_type, data, &self.actor)?;
        let board = self.create_board(imported.board).await?;
        let mut guard = self.boards.lock().await;
        let st = self.state(&mut guard, board.id).await?;
        for doc in imported.elements {
            let mut doc = doc;
            doc.element.board = board.id;
            self.save_element(&doc).await?;
            st.put(doc);
        }
        Ok(st.board(&board.iri))
    }
}

fn non_blank(name: &str) -> Result<&str> {
    if name.trim().is_empty() {
        return Err(Error::invalid("a board needs a name"));
    }
    crate::validate::text(name)?;
    Ok(name)
}

impl WhiteboardRepo {
    async fn create_board(&self, doc: BoardDoc) -> Result<Board> {
        let id = ObjectId::new();
        let o = board_to_stored(&doc, id, &self.instance, &self.actor, None)?;
        self.store.put(o, &self.actor).await?;
        let mut guard = self.boards.lock().await;
        let st = self.state(&mut guard, id).await?;
        Ok(st.board(&self.iri(id)))
    }
}
