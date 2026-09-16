//! Integration test for User Story 3: newly added movies appear after the next run,
//! with a later `generatedAt` timestamp, per specs/001-remote-movie-catalog/spec.md
//! (User Story 3, Acceptance Scenario 1).

use std::fs;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use serde_json::{Value, json};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

async fn mount_library(server: &MockServer, items: Value, total: i64) {
    Mock::given(method("GET"))
        .and(path("/Library/VirtualFolders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            { "Name": "Movies", "ItemId": "lib-1", "CollectionType": "movies" }
        ])))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Items": items,
            "TotalRecordCount": total
        })))
        .mount(server)
        .await;
}

fn extract_generated_at(html: &str) -> String {
    let marker = "\"generatedAt\":\"";
    let start = html.find(marker).expect("generatedAt not found") + marker.len();
    let end = html[start..].find('"').expect("unterminated generatedAt") + start;
    html[start..end].to_string()
}

#[tokio::test]
async fn newly_added_movie_appears_after_the_next_run_with_a_later_timestamp() {
    let output_dir = tempfile::tempdir().unwrap();
    let out_path = output_dir.path().join("site");

    let server_1 = MockServer::start().await;
    mount_library(&server_1, json!([{ "Id": "1", "Name": "First Movie" }]), 1).await;

    let status = binary()
        .args([
            "--server-url",
            &server_1.uri(),
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let first_html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(first_html.contains("First Movie"));
    assert!(!first_html.contains("Second Movie"));
    let first_generated_at = extract_generated_at(&first_html);

    sleep(Duration::from_millis(20));

    let server_2 = MockServer::start().await;
    mount_library(
        &server_2,
        json!([
            { "Id": "1", "Name": "First Movie" },
            { "Id": "2", "Name": "Second Movie" }
        ]),
        2,
    )
    .await;

    let status = binary()
        .args([
            "--server-url",
            &server_2.uri(),
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let second_html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(second_html.contains("First Movie"));
    assert!(second_html.contains("Second Movie"));
    let second_generated_at = extract_generated_at(&second_html);

    assert!(second_generated_at > first_generated_at);
}

/// Per spec.md's edge case: "What happens when a movie is removed from the library at
/// home? It MUST no longer appear in the overview after the next successful refresh."
#[tokio::test]
async fn removed_movie_is_absent_after_the_next_run() {
    let output_dir = tempfile::tempdir().unwrap();
    let out_path = output_dir.path().join("site");

    let server_1 = MockServer::start().await;
    mount_library(
        &server_1,
        json!([
            { "Id": "1", "Name": "Keeper Movie" },
            { "Id": "2", "Name": "Soon Removed Movie" }
        ]),
        2,
    )
    .await;

    let status = binary()
        .args([
            "--server-url",
            &server_1.uri(),
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let first_html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(first_html.contains("Keeper Movie"));
    assert!(first_html.contains("Soon Removed Movie"));

    let server_2 = MockServer::start().await;
    mount_library(&server_2, json!([{ "Id": "1", "Name": "Keeper Movie" }]), 1).await;

    let status = binary()
        .args([
            "--server-url",
            &server_2.uri(),
            "--output-dir",
            out_path.to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(status.success());
    let second_html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(second_html.contains("Keeper Movie"));
    assert!(!second_html.contains("Soon Removed Movie"));
}
