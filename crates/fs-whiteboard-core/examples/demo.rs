//! A board in one go: shapes, a group, a connector, undo, a concurrent edit on a second replica, SVG.
//!
//! Run with `cargo run -p fs-whiteboard-core --example demo`.

use std::sync::Arc;

use fs_objects::{Query, Store, TursoStore};
use fs_whiteboard_core::{
    Anchor, Boards, Connector, Element, Error, Kind, Rect, ShapeKind, StyleField, WhiteboardRepo,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let store: Arc<dyn Store> = Arc::new(TursoStore::open_in_memory().await?);
    let ada = WhiteboardRepo::new(store.clone(), "https://example.org/people/ada")
        .with_instance("example.org");
    let board = ada.create("Roof plan").await?;
    let page = board.pages[0].id;
    let ed = ada.open(board.id).await?.expect("just created");

    let shape = |x: f64| {
        Element::new(
            page,
            Kind::Shape {
                shape: ShapeKind::Rectangle,
            },
            Rect::new(x, 0.0, 80.0, 40.0),
        )
    };
    let a = ed.add(shape(0.0)).await?.id;
    let b = ed.add(shape(200.0)).await?.id;
    let link = Connector {
        source: Anchor {
            element: Some(a),
            ..Anchor::default()
        },
        target: Anchor {
            element: Some(b),
            ..Anchor::default()
        },
        label: "feeds".into(),
        arrow_end: true,
    };
    ed.add(Element::new(
        page,
        Kind::Connector(link),
        Rect::new(80.0, 20.0, 120.0, 0.0),
    ))
    .await?;
    ed.set_style(&[a, b], &[StyleField::Stroke("#222222".into())])
        .await?;
    let group = ed.group(&[a, b]).await?;
    ed.move_by(&[group], 0.0, 100.0).await?;
    println!(
        "moved the group; a is at y = {}",
        ed.get_element(a).await?.expect("there").frame.y
    );
    ed.undo().await?;
    println!(
        "undone;           a is at y = {}",
        ed.get_element(a).await?.expect("there").frame.y
    );

    // Bob works on his own replica, apart; his change reaches Ada through apply_remote
    let bob_store: Arc<dyn Store> = Arc::new(TursoStore::open_in_memory().await?);
    let bob = WhiteboardRepo::new(bob_store.clone(), "https://example.org/people/bob");
    for repo_from in [&store] {
        bob.apply_remote(repo_from.get(board.id).await?.expect("board"))
            .await?;
        let q = Query::new()
            .of_type(fs_whiteboard_core::ELEMENT_TYPE)
            .with_index("board", board.id.to_string());
        for o in repo_from.query(&q).await? {
            bob.apply_remote(o).await?;
        }
    }
    let bobs = bob.open(board.id).await?.expect("synced");
    bobs.set_style(&[a], &[StyleField::Fill("#ffcc00".into())])
        .await?;
    ed.move_by(&[a], 10.0, 0.0).await?;
    ada.apply_remote(bob_store.get(a).await?.expect("a"))
        .await?;
    let merged = ed.get_element(a).await?.expect("there");
    println!(
        "merged: x = {}, fill = {}",
        merged.frame.x, merged.style.fill
    );

    let svg = ed.export(Some(page), &[], "image/svg+xml").await?;
    println!(
        "svg: {} bytes\n{}",
        svg.bytes.len(),
        String::from_utf8_lossy(&svg.bytes)
    );
    for (id, why) in Boards::capabilities(&ada)
        .entries()
        .iter()
        .filter(|e| !e.supported)
        .map(|e| (&e.capability, &e.reason))
    {
        println!("not offered: {id} ({why})");
    }
    Ok(())
}
