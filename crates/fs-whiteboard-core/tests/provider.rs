//! The provider rule (ADR 0045): a provider that overrides nothing answers `Unsupported`, and the
//! report lists every capability id the contract names.

mod common;

use async_trait::async_trait;
use common::setup;
use fs_interfaces::Capabilities;
use fs_objects::ObjectId;
use fs_whiteboard_core::{
    Board, BoardEditor, Boards, Cursor, Element, Error, Paged, Rect, Result, WhiteboardFeed,
};

/// Every capability id in `whiteboard.capnp` v0.26.0 (comments of the methods).
const CONTRACT: [&str; 31] = [
    "board.create",
    "board.open",
    "board.delete",
    "board.import",
    "board.export",
    "board.save",
    "board.view",
    "board.subscribe",
    "canvas.navigate",
    "page.add",
    "page.remove",
    "shape.add",
    "text.add",
    "note.add",
    "stroke.draw",
    "connector.add",
    "image.add",
    "frame.add",
    "element.move",
    "element.style",
    "element.lock",
    "element.delete",
    "element.group",
    "element.ungroup",
    "element.reorder",
    "element.align",
    "text.edit",
    "connector.edit",
    "stroke.erase",
    "history.undo",
    "history.redo",
];
/// Named in the header of the contract as not in it yet.
const NOT_YET: [&str; 7] = [
    "sync",
    "board.sync",
    "board.share",
    "presence.*",
    "library.*",
    "template.*",
    "comment.add",
];

#[tokio::test]
async fn the_report_names_every_capability_of_the_contract() {
    let (repo, _) = setup().await;
    let caps = Boards::capabilities(&repo);
    for id in CONTRACT {
        assert!(caps.is_supported(id), "{id} should be supported");
    }
    for id in NOT_YET {
        let why = caps
            .why_not(id)
            .unwrap_or_else(|| panic!("{id} should say why not"));
        assert_eq!(why.capability(), id);
        assert!(
            caps.entries()
                .iter()
                .find(|e| e.capability == id)
                .is_some_and(|e| !e.reason.is_empty())
        );
    }
    for id in ["board.export.png", "board.export.pdf"] {
        assert!(caps.why_not(id).is_some());
    }
    let mut ids: Vec<&str> = caps
        .entries()
        .iter()
        .map(|e| e.capability.as_str())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), caps.entries().len(), "no id twice");
}

struct ReadOnly;

#[async_trait]
impl Boards for ReadOnly {
    fn capabilities(&self) -> Capabilities {
        Capabilities::new("read-only")
    }
    async fn list(&self, _: Option<&Cursor>) -> Result<Paged<Board>> {
        Ok(Paged {
            items: vec![],
            next: None,
        })
    }
    async fn open(&self, _: ObjectId) -> Result<Option<Box<dyn BoardEditor>>> {
        Ok(None)
    }
}

#[async_trait]
impl WhiteboardFeed for ReadOnly {}

#[async_trait]
impl BoardEditor for ReadOnly {
    async fn get(&self) -> Result<Board> {
        Err(Error::invalid("empty"))
    }
    async fn get_element(&self, _: ObjectId) -> Result<Option<Element>> {
        Ok(None)
    }
    async fn elements(
        &self,
        _: ObjectId,
        _: Option<Rect>,
        _: Option<&Cursor>,
    ) -> Result<Paged<Element>> {
        Ok(Paged {
            items: vec![],
            next: None,
        })
    }
    async fn bounds(&self, _: ObjectId) -> Result<Option<Rect>> {
        Ok(None)
    }
}

fn unsupported<T: std::fmt::Debug>(r: Result<T>, id: &str) {
    match r {
        Err(Error::Unsupported(u)) => assert_eq!(u.capability(), id),
        other => panic!("{id}: {other:?}"),
    }
}

#[tokio::test]
async fn a_provider_that_overrides_nothing_says_unsupported() {
    let p = ReadOnly;
    let id = ObjectId::new();
    unsupported(p.create("x").await, "board.create");
    unsupported(p.rename(id, "x").await, "board.create");
    unsupported(Boards::delete(&p, id).await, "board.delete");
    unsupported(Boards::import(&p, "x", b"").await, "board.import");
    unsupported(p.subscribe(id).await.map(|_| ()), "board.subscribe");
    unsupported(p.add_page("x").await, "page.add");
    unsupported(p.remove_page(id).await, "page.remove");
    unsupported(
        p.add(Element::new(
            id,
            fs_whiteboard_core::Kind::Group,
            Rect::default(),
        ))
        .await,
        "shape.add",
    );
    unsupported(p.move_by(&[id], 0.0, 0.0).await, "element.move");
    unsupported(p.set_style(&[id], &[]).await, "element.style");
    unsupported(p.set_text(id, "").await, "text.edit");
    unsupported(p.set_locked(&[id], true).await, "element.lock");
    unsupported(BoardEditor::delete(&p, &[id]).await, "element.delete");
    unsupported(p.group(&[id]).await, "element.group");
    unsupported(p.undo().await, "history.undo");
    unsupported(p.redo().await, "history.redo");
    unsupported(p.export(None, &[], "x").await, "board.export");
}
