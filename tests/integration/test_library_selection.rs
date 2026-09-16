//! Integration tests for `--library-name` selection, per
//! specs/001-remote-movie-catalog/contracts/cli-interface.md and
//! src/jellyfin/client.rs's `JellyfinError::{LibraryNotFound, NoMovieLibrary}`.

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn selects_the_named_library_among_several() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Library/VirtualFolders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "Name": "Kids Movies", "ItemId": "lib-kids", "CollectionType": "movies" },
            { "Name": "Home Videos", "ItemId": "lib-videos", "CollectionType": "homevideos" },
            { "Name": "Main Movies", "ItemId": "lib-main", "CollectionType": "movies" }
        ])))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Items": [{ "Id": "1", "Name": "Main Library Movie" }],
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
            "--library-name",
            "Main Movies",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    let html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(html.contains("Main Library Movie"));
}

#[tokio::test]
async fn exits_2_when_named_library_does_not_exist() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Library/VirtualFolders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "Name": "Main Movies", "ItemId": "lib-main", "CollectionType": "movies" }
        ])))
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
            "--library-name",
            "Does Not Exist",
        ])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
    assert!(!out_path.exists());
}

#[tokio::test]
async fn exits_2_when_server_has_no_movie_library() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Library/VirtualFolders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "Name": "Home Videos", "ItemId": "lib-videos", "CollectionType": "homevideos" }
        ])))
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
    assert_eq!(status.code(), Some(2));
    assert!(!out_path.exists());
}
