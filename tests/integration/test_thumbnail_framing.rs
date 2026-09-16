//! Integration test for User Story 2 (002-ascii-terminal-theme): real thumbnail images
//! (and their placeholder) are framed with terminal-styled decoration, per FR-005.

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn thumbnails_and_placeholder_are_framed_with_the_terminal_palette() {
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
                    "Id": "has-image",
                    "Name": "Has Image",
                    "ImageTags": { "Primary": "tag-1" }
                },
                { "Id": "no-image", "Name": "No Image" }
            ],
            "TotalRecordCount": 2
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items/has-image/Images/Primary"))
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
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    assert!(out_path.join("images/has-image.jpg").exists());

    let html = fs::read_to_string(out_path.join("index.html")).unwrap();

    // Both .movie-thumb (real image) and .movie-thumb-placeholder (no image) must be
    // framed with the terminal palette, not the old gray values.
    assert!(html.contains(".movie-thumb"));
    assert!(html.contains("var(--fg-dim)"));
    assert!(!html.contains("#ddd"));
    assert!(!html.contains("#333"));
    assert!(!html.contains("#888"));
}
