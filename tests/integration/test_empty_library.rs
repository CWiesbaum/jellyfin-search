//! Integration test for an empty library, per data-model.md's `CatalogSnapshot.movies`
//! note ("an empty library is valid — the page should render a clear 'no movies yet'
//! state rather than erroring").

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn empty_library_exports_successfully_with_a_distinct_empty_state() {
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
            "Items": [],
            "TotalRecordCount": 0
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
    assert!(
        status.success(),
        "an empty library must not be treated as an error"
    );

    let html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(html.contains(r#"id="empty-library""#));
    assert!(html.contains("\"movies\":[]"));
    // The empty-library message must be distinct from the search "no match" message.
    assert!(html.contains("Your library is empty"));
}
