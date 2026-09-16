//! Integration test for User Story 3: a failed run leaves previously published
//! output completely untouched, per specs/001-remote-movie-catalog/spec.md
//! (edge case) and FR-009.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path as wpath};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

fn snapshot_dir(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut map = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                let contents = fs::read(&p).unwrap();
                map.insert(p, contents);
            }
        }
    }
    map
}

#[tokio::test]
async fn failed_run_leaves_previous_output_untouched() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(wpath("/Library/VirtualFolders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "Name": "Movies", "ItemId": "lib-1", "CollectionType": "movies" }
        ])))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(wpath("/Items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Items": [{ "Id": "1", "Name": "Good Movie" }],
            "TotalRecordCount": 1
        })))
        .mount(&server)
        .await;

    let output_dir = tempfile::tempdir().unwrap();
    let out_path = output_dir.path().join("site");

    let status = binary()
        .args([
            "--server-url",
            &server.uri(),
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let before = snapshot_dir(&out_path);

    let output = binary()
        .args([
            "--server-url",
            "http://127.0.0.1:1", // nothing listens here: connection refused
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));

    let after = snapshot_dir(&out_path);
    assert_eq!(before, after);
}
