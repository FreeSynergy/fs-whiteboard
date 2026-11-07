//! Shared helpers of the integration tests.

#![allow(dead_code)] // each test file uses a different subset

pub mod sample;
pub mod xml;

#[allow(unused_imports)]
pub use sample::{busy, el, exported};

use std::sync::Arc;

use fs_objects::{ObjectId, Store, TursoStore};
use fs_whiteboard_core::{BoardEditor, Boards, Element, Kind, Rect, ShapeKind, WhiteboardRepo};

/// Proptest settings: modest case counts, no failure files in the source tree.
pub fn cfg(cases: u32) -> proptest::test_runner::Config {
    proptest::test_runner::Config {
        cases,
        failure_persistence: None,
        ..Default::default()
    }
}

pub const ME: &str = "https://example.org/users/me";

/// A repository over a fresh in-memory store, and the store.
pub async fn setup() -> (WhiteboardRepo, Arc<dyn Store>) {
    let store: Arc<dyn Store> = Arc::new(TursoStore::open_in_memory().await.unwrap());
    (
        WhiteboardRepo::new(store.clone(), ME).with_instance("example.org"),
        store,
    )
}

/// A board with one page, an editor on it, and the ids.
pub struct Fixture {
    pub repo: WhiteboardRepo,
    pub store: Arc<dyn Store>,
    pub board: ObjectId,
    pub page: ObjectId,
    pub ed: Box<dyn BoardEditor>,
}

pub async fn fixture() -> Fixture {
    let (repo, store) = setup().await;
    let board = repo.create("Test board").await.unwrap();
    let page = board.pages[0].id;
    let ed = repo.open(board.id).await.unwrap().unwrap();
    Fixture {
        repo,
        store,
        board: board.id,
        page,
        ed,
    }
}

impl Fixture {
    /// A second person's editor on the same board.
    pub async fn editor_of(&self, actor: &str) -> Box<dyn BoardEditor> {
        self.repo
            .as_actor(actor)
            .open(self.board)
            .await
            .unwrap()
            .unwrap()
    }

    /// A rectangle at `(x, y)` of size 10 x 10, added through the editor.
    pub async fn rect(&self, x: f64, y: f64) -> ObjectId {
        add_rect(&*self.ed, self.page, x, y).await
    }
}

pub fn rect_element(page: ObjectId, x: f64, y: f64) -> Element {
    Element::new(
        page,
        Kind::Shape {
            shape: ShapeKind::Rectangle,
        },
        Rect::new(x, y, 10.0, 10.0),
    )
}

pub async fn add_rect(ed: &dyn BoardEditor, page: ObjectId, x: f64, y: f64) -> ObjectId {
    ed.add(rect_element(page, x, y)).await.unwrap().id
}

/// All elements of a page, front to back.
pub async fn all(ed: &dyn BoardEditor, page: ObjectId) -> Vec<Element> {
    ed.elements(page, None, None).await.unwrap().items
}

pub async fn frame_of(ed: &dyn BoardEditor, id: ObjectId) -> Rect {
    ed.get_element(id).await.unwrap().unwrap().frame
}

/// Feeds a board and all its elements from one store into another repository, as a sync adapter would.
pub async fn replicate(from: &Arc<dyn Store>, to: &WhiteboardRepo, board: ObjectId) {
    to.apply_remote(from.get(board).await.unwrap().unwrap())
        .await
        .unwrap();
    let q = fs_objects::Query::new()
        .of_type(fs_whiteboard_core::ELEMENT_TYPE)
        .with_index("board", board.to_string());
    for o in from.query(&q).await.unwrap() {
        to.apply_remote(o).await.unwrap();
    }
}

/// Two replicas of one board, each with its own store and actor.
pub struct Pair {
    pub a: Fixture,
    pub b: Fixture,
}

pub async fn pair() -> Pair {
    let a = fixture().await;
    let (repo_b, store_b) = setup().await;
    let repo_b = repo_b.as_actor("replica-b");
    replicate(&a.store, &repo_b, a.board).await;
    let ed = repo_b.open(a.board).await.unwrap().unwrap();
    let b = Fixture {
        repo: repo_b,
        store: store_b,
        board: a.board,
        page: a.page,
        ed,
    };
    Pair { a, b }
}

impl Pair {
    /// Both directions, so both replicas hold everything.
    pub async fn sync(&self) {
        replicate(&self.a.store, &self.b.repo, self.a.board).await;
        replicate(&self.b.store, &self.a.repo, self.a.board).await;
    }
}

/// The content of an element as stored, for byte comparison.
pub async fn raw(store: &Arc<dyn Store>, id: ObjectId) -> String {
    store.get(id).await.unwrap().unwrap().content.to_string()
}

/// The ids of a page, back to front.
pub async fn order_of(ed: &dyn BoardEditor, page: ObjectId) -> Vec<ObjectId> {
    all(ed, page).await.iter().map(|e| e.id).collect()
}

/// The ids of the elements of a page that touch `area`.
pub async fn ids_in(ed: &dyn BoardEditor, page: ObjectId, area: Rect) -> Vec<ObjectId> {
    ed.elements(page, Some(area), None)
        .await
        .unwrap()
        .items
        .iter()
        .map(|e| e.id)
        .collect()
}

/// The stored object of an element.
pub async fn element_object(f: &Fixture, id: ObjectId) -> fs_objects::StoredObject {
    f.store.get(id).await.unwrap().unwrap()
}
