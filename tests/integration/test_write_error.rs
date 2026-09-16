//! Integration test for the `EXIT_WRITE_ERROR` (3) path, per
//! specs/001-remote-movie-catalog/contracts/cli-interface.md.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn exits_3_when_output_directory_cannot_be_written() {
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

    let base = tempfile::tempdir().unwrap();
    let readonly_parent = base.path().join("readonly_parent");
    fs::create_dir_all(&readonly_parent).unwrap();
    // No write permission on the parent: the atomic writer's staging directory (a
    // sibling of --output-dir, created inside this same parent) cannot be created.
    fs::set_permissions(&readonly_parent, fs::Permissions::from_mode(0o555)).unwrap();
    let out_path = readonly_parent.join("site");

    let output = binary()
        .args([
            "--server-url",
            &server.uri(),
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .output()
        .unwrap();

    // Restore write permission before the tempdir is cleaned up on drop.
    fs::set_permissions(&readonly_parent, fs::Permissions::from_mode(0o755)).unwrap();

    assert_eq!(output.status.code(), Some(3));
    assert!(!out_path.exists());
}
