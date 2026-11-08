//! Imports of the own format that must be refused.

mod common;

use common::{busy, exported, fixture};
use fs_objects::ObjectId;
use fs_whiteboard_core::{Boards, Error};
use serde_json::{Value, json};
const OWN: &str = "application/vnd.freesynergy.whiteboard+json";

#[tokio::test]
async fn imports_that_point_at_nothing_or_loop_or_break_numbers_are_refused() {
    let fx = fixture().await;
    busy(&fx).await;
    let good = exported(&fx.ed.export(None, &[], OWN).await.unwrap().bytes);
    let ghost = json!(ObjectId::new());
    let refuse = |label: &'static str, val: Value| {
        let repo = fx.repo.clone();
        async move {
            let r = repo.import(OWN, val.to_string().as_bytes()).await;
            assert!(
                matches!(
                    r,
                    Err(Error::Invalid(_) | Error::Json(_) | Error::TooLarge(_))
                ),
                "{label}: {r:?}"
            );
        }
    };
    let mut val = good.clone();
    val["elements"][0]["parent"] = ghost.clone();
    refuse("dangling parent", val).await;
    let mut val = good.clone();
    val["elements"][0]["page"] = ghost.clone();
    refuse("dangling page", val).await;
    let mut val = good.clone();
    let pos = val["elements"]
        .as_array()
        .unwrap()
        .iter()
        .position(|e| e["kind"]["type"] == "connector")
        .unwrap();
    val["elements"][pos]["kind"]["source"]["element"] = ghost;
    refuse("dangling connector end", val).await;
    let mut val = good.clone();
    let groups: Vec<usize> = (0..val["elements"].as_array().unwrap().len())
        .filter(|pos| val["elements"][*pos]["kind"]["type"] == "group")
        .collect();
    let (grp, other) = (groups[0], 0);
    let (gid, oid) = (
        val["elements"][grp]["id"].clone(),
        val["elements"][other]["id"].clone(),
    );
    val["elements"][grp]["parent"] = oid;
    val["elements"][other]["parent"] = gid;
    refuse("cycle", val).await;
    let mut val = good.clone();
    val["elements"][0]["frame"]["width"] = json!(-3.0);
    refuse("negative size", val).await;
    let mut val = good.clone();
    val["version"] = json!(2);
    refuse("version", val).await;
    let mut val = good.clone();
    val["elements"][1]["id"] = val["elements"][0]["id"].clone();
    refuse("duplicate id", val).await;
    let mut val = good.clone();
    val["elements"][0]["order"] = json!("a0");
    refuse("bad order key", val).await;

    // NaN and infinity cannot even be written in JSON; the usual spellings are refused as well
    let text = good.to_string();
    for bad in ["NaN", "Infinity", "-Infinity", "1e999"] {
        let broken = text.replacen("\"angle\":0.0", &format!("\"angle\":{bad}"), 1);
        assert_ne!(broken, text);
        let r = fx.repo.import(OWN, broken.as_bytes()).await;
        assert!(
            matches!(r, Err(Error::Json(_) | Error::Invalid(_))),
            "{bad}: {r:?}"
        );
    }
    assert!(fx.repo.import(OWN, b"not json").await.is_err());
    assert_eq!(
        fx.repo.list(None).await.unwrap().items.len(),
        1,
        "nothing was imported"
    );
}
