//! Imports of the own format that must be refused.

mod common;

use common::{busy, exported, fixture};
use fs_objects::ObjectId;
use fs_whiteboard_core::{Boards, Error};
use serde_json::{Value, json};
const OWN: &str = "application/vnd.freesynergy.whiteboard+json";

#[tokio::test]
async fn imports_that_point_at_nothing_or_loop_or_break_numbers_are_refused() {
    let f = fixture().await;
    busy(&f).await;
    let good = exported(&f.ed.export(None, &[], OWN).await.unwrap().bytes);
    let ghost = json!(ObjectId::new());
    let refuse = |label: &'static str, v: Value| {
        let repo = f.repo.clone();
        async move {
            let r = repo.import(OWN, v.to_string().as_bytes()).await;
            assert!(
                matches!(
                    r,
                    Err(Error::Invalid(_) | Error::Json(_) | Error::TooLarge(_))
                ),
                "{label}: {r:?}"
            );
        }
    };
    let mut v = good.clone();
    v["elements"][0]["parent"] = ghost.clone();
    refuse("dangling parent", v).await;
    let mut v = good.clone();
    v["elements"][0]["page"] = ghost.clone();
    refuse("dangling page", v).await;
    let mut v = good.clone();
    let i = v["elements"]
        .as_array()
        .unwrap()
        .iter()
        .position(|e| e["kind"]["type"] == "connector")
        .unwrap();
    v["elements"][i]["kind"]["source"]["element"] = ghost;
    refuse("dangling connector end", v).await;
    let mut v = good.clone();
    let groups: Vec<usize> = (0..v["elements"].as_array().unwrap().len())
        .filter(|i| v["elements"][*i]["kind"]["type"] == "group")
        .collect();
    let (g, other) = (groups[0], 0);
    let (gid, oid) = (
        v["elements"][g]["id"].clone(),
        v["elements"][other]["id"].clone(),
    );
    v["elements"][g]["parent"] = oid;
    v["elements"][other]["parent"] = gid;
    refuse("cycle", v).await;
    let mut v = good.clone();
    v["elements"][0]["frame"]["width"] = json!(-3.0);
    refuse("negative size", v).await;
    let mut v = good.clone();
    v["version"] = json!(2);
    refuse("version", v).await;
    let mut v = good.clone();
    v["elements"][1]["id"] = v["elements"][0]["id"].clone();
    refuse("duplicate id", v).await;
    let mut v = good.clone();
    v["elements"][0]["order"] = json!("a0");
    refuse("bad order key", v).await;

    // NaN and infinity cannot even be written in JSON; the usual spellings are refused as well
    let text = good.to_string();
    for bad in ["NaN", "Infinity", "-Infinity", "1e999"] {
        let broken = text.replacen("\"angle\":0.0", &format!("\"angle\":{bad}"), 1);
        assert_ne!(broken, text);
        let r = f.repo.import(OWN, broken.as_bytes()).await;
        assert!(
            matches!(r, Err(Error::Json(_) | Error::Invalid(_))),
            "{bad}: {r:?}"
        );
    }
    assert!(f.repo.import(OWN, b"not json").await.is_err());
    assert_eq!(
        f.repo.list(None).await.unwrap().items.len(),
        1,
        "nothing was imported"
    );
}
