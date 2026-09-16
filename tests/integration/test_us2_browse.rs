//! Integration test for User Story 2: rich browsing with graceful metadata
//! fallbacks, per specs/001-remote-movie-catalog/spec.md (User Story 2) and
//! data-model.md's thumbnail-download validation rule.

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn thumbnails_are_downloaded_and_missing_metadata_is_handled_gracefully() {
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
                    "ProductionYear": 2015,
                    "Genres": ["Action"],
                    "ImageTags": { "Primary": "tag-1" }
                },
                {
                    "Id": "has-png-image",
                    "Name": "Has PNG Image",
                    "ImageTags": { "Primary": "tag-3" }
                },
                {
                    "Id": "broken-image",
                    "Name": "Broken Image",
                    "ImageTags": { "Primary": "tag-2" }
                },
                { "Id": "no-metadata", "Name": "No Metadata" }
            ],
            "TotalRecordCount": 4
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

    Mock::given(method("GET"))
        .and(path("/Items/has-png-image/Images/Primary"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "image/png")
                .set_body_bytes(vec![0x89u8, 0x50, 0x4E, 0x47]),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items/broken-image/Images/Primary"))
        .respond_with(ResponseTemplate::new(500))
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
    assert!(out_path.join("images/has-png-image.png").exists());
    assert!(!out_path.join("images/broken-image.jpg").exists());

    let html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(html.contains("images/has-image.jpg"));
    assert!(html.contains("images/has-png-image.png"));
    assert!(!html.contains("images/broken-image.jpg"));
    assert!(html.contains("No Metadata"));
    assert!(html.contains("Broken Image"));
}
