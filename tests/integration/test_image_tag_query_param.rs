//! Integration test verifying the movie's `ImageTags.Primary` tag is passed as the
//! `tag` query parameter when fetching its thumbnail, per research.md §2 ("using the
//! returned image tag for cache-busting/consistency").

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn thumbnail_request_includes_the_image_tag() {
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
                    "Name": "Tagged Movie",
                    "ImageTags": { "Primary": "expected-tag-value" }
                }
            ],
            "TotalRecordCount": 1
        })))
        .mount(&server)
        .await;

    // Only responds when the request's `tag` query parameter matches the tag reported
    // in the /Items response above; a request without it (or with the wrong value)
    // gets wiremock's default 404, which would surface as a download warning and an
    // absent thumbnail in the output.
    Mock::given(method("GET"))
        .and(path("/Items/1/Images/Primary"))
        .and(query_param("tag", "expected-tag-value"))
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

    assert!(
        out_path.join("images/1.jpg").exists(),
        "thumbnail should have downloaded via the tag-matched mock"
    );
    let html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(html.contains("images/1.jpg"));
}
