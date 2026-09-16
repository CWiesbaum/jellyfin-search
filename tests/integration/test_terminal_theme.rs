//! Integration test for User Story 1 (002-ascii-terminal-theme): the whole generated
//! page presents the fixed black-background/green-text terminal palette, per FR-001.

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn generated_page_uses_the_fixed_terminal_palette() {
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
            "Items": [{ "Id": "1", "Name": "Some Movie" }],
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

    let html = fs::read_to_string(out_path.join("index.html")).unwrap();

    assert!(html.contains("--bg: #000000"));
    assert!(html.contains("--fg: #33ff33"));
    assert!(html.contains("--fg-dim: #1a8c1a"));

    // None of the 9 retired light-theme values may remain anywhere on the page.
    for retired_color in [
        "#fafafa", "#eee", "#ccc", "#666", "#aaa", "#111", "#ddd", "#333", "#888",
    ] {
        assert!(
            !html.contains(retired_color),
            "retired light-theme color {retired_color} should not appear in the generated page"
        );
    }
}
