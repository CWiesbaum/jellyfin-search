//! Regression test guarding plan.md's Constraint: "Jellyfin credentials (API key)
//! MUST NOT appear anywhere in the generated output directory."

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn api_key_never_appears_in_generated_output() {
    const SECRET_API_KEY: &str = "super-secret-do-not-leak-9f8e7d6c";

    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Library/VirtualFolders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "Name": "Movies", "ItemId": "lib-1", "CollectionType": "movies" }
        ])))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Items": [
                {
                    "Id": "1",
                    "Name": "Some Movie",
                    "ProductionYear": 2020,
                    "Genres": ["Drama"],
                    "ImageTags": { "Primary": "tag-1" }
                }
            ],
            "TotalRecordCount": 1
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items/1/Images/Primary"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "image/jpeg")
                .set_body_bytes(vec![0xFFu8, 0xD8, 0xFF]),
        )
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
            SECRET_API_KEY,
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let mut checked_files = 0usize;
    for entry in walk(&out_path) {
        let contents = fs::read(&entry).unwrap();
        assert!(
            !contains_subslice(&contents, SECRET_API_KEY.as_bytes()),
            "API key leaked into generated file: {}",
            entry.display()
        );
        checked_files += 1;
    }
    assert!(
        checked_files >= 2,
        "expected index.html and at least one image to be checked"
    );
}

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                result.push(p);
            }
        }
    }
    result
}
