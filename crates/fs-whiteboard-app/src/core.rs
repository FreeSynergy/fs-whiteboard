//! The core as the app holds it: the repository, a runtime handle, and one editor per board.
//!
//! Pattern: Facade + Registry — the three backends share one `Core`. It opens each board's editor once and keeps it, so
//! `undo` and `redo` work on the history of this window's session (one editor is one person's session, core DESIGN).
//! The admin frontend's `Backend` is synchronous, so a call blocks on the runtime (named in `DESIGN.md`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use fs_admin_frontend::BackendError;
use fs_objects::ObjectId;
use fs_whiteboard_core::{
    Board, BoardEditor, Boards, Cursor, Element, Error, Page, WhiteboardRepo,
};
use tokio::runtime::Handle;

use crate::ids::iri;

/// The repository with what a backend needs around it.
pub struct Core {
    pub(crate) repo: WhiteboardRepo,
    handle: Handle,
    instance: String,
    editors: Mutex<HashMap<ObjectId, Arc<dyn BoardEditor>>>,
}

/// A core refusal as the frontend's error: a missing thing is `NotFound`, everything else is told with its English text.
pub fn failure(e: &Error) -> BackendError {
    match e {
        Error::NotFound { .. } => BackendError::NotFound,
        other => BackendError::Unavailable(other.to_string()),
    }
}

impl Core {
    /// Over `repo`, calling on `handle`. `instance` must be the one the repository was made with.
    pub fn new(repo: WhiteboardRepo, handle: Handle, instance: &str) -> Arc<Self> {
        Arc::new(Self {
            repo,
            handle,
            instance: instance.to_owned(),
            editors: Mutex::new(HashMap::new()),
        })
    }

    /// The IRI of a record.
    pub fn iri(&self, id: ObjectId) -> String {
        iri(&self.instance, id)
    }

    /// Run a core call to its end.
    pub fn run<T>(
        &self,
        fut: impl std::future::Future<Output = fs_whiteboard_core::Result<T>>,
    ) -> Result<T, BackendError> {
        self.handle.block_on(fut).map_err(|e| failure(&e))
    }

    /// The editor of a board (opened once, then kept).
    pub fn editor(&self, board: ObjectId) -> Result<Arc<dyn BoardEditor>, BackendError> {
        if let Some(e) = self
            .editors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&board)
        {
            return Ok(Arc::clone(e));
        }
        let opened = self
            .run(self.repo.open(board))?
            .ok_or(BackendError::NotFound)?;
        let editor: Arc<dyn BoardEditor> = Arc::from(opened);
        self.editors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(board, Arc::clone(&editor));
        Ok(editor)
    }

    /// Forget the editor of a deleted board.
    pub fn forget(&self, board: ObjectId) {
        self.editors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&board);
    }

    /// Every board, page by page.
    pub fn boards(&self) -> Result<Vec<Board>, BackendError> {
        let mut out = Vec::new();
        let mut cursor: Option<Cursor> = None;
        loop {
            let page = self.run(self.repo.list(cursor.as_ref()))?;
            out.extend(page.items);
            match page.next {
                Some(next) => cursor = Some(next),
                None => return Ok(out),
            }
        }
    }

    /// The board that holds `page`.
    pub fn board_of_page(&self, page: ObjectId) -> Result<Board, BackendError> {
        self.boards()?
            .into_iter()
            .find(|b| b.pages.iter().any(|p| p.id == page))
            .ok_or(BackendError::NotFound)
    }

    /// The board that holds the element `id`, and the element.
    pub fn find_element(&self, id: ObjectId) -> Result<(Board, Element), BackendError> {
        for board in self.boards()? {
            let editor = self.editor(board.id)?;
            if let Some(el) = self.run(editor.get_element(id))? {
                return Ok((board, el));
            }
        }
        Err(BackendError::NotFound)
    }

    /// The elements shown on a page, page by page.
    pub fn elements(&self, board: ObjectId, page: ObjectId) -> Result<Vec<Element>, BackendError> {
        let editor = self.editor(board)?;
        let mut out = Vec::new();
        let mut cursor: Option<Cursor> = None;
        loop {
            let got = self.run(editor.elements(page, None, cursor.as_ref()))?;
            out.extend(got.items);
            match got.next {
                Some(next) => cursor = Some(next),
                None => return Ok(out),
            }
        }
    }

    /// The pages of all boards that are not removed, with the board of each.
    pub fn pages(&self) -> Result<Vec<(Board, Page)>, BackendError> {
        Ok(self
            .boards()?
            .into_iter()
            .flat_map(|b| {
                b.pages
                    .clone()
                    .into_iter()
                    .filter(|p| !p.deleted)
                    .map(move |p| (b.clone(), p))
            })
            .collect())
    }
}
