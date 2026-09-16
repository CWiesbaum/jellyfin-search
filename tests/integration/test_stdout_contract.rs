//! Integration test for the documented stdout contract, per
//! specs/001-remote-movie-catalog/contracts/cli-interface.md: "on success, a single
//! final summary line (movie count and output path). Nothing is written to stdout on
//! failure."

use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn success_writes_exactly_one_summary_line_to_stdout() {
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

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "expected exactly one stdout line, got: {stdout:?}"
    );
    assert!(
        lines[0].contains('1'),
        "summary line should mention the movie count"
    );
    assert!(
        lines[0].contains(out_path.to_str().unwrap()),
        "summary line should mention the output path"
    );
}

#[tokio::test]
async fn failure_writes_nothing_to_stdout() {
    let output_dir = tempfile::tempdir().unwrap();
    let out_path = output_dir.path().join("site");

    // Missing --api-key and no JELLYFIN_API_KEY env var: a config error (exit 1),
    // the earliest failure point in the pipeline.
    let output = binary()
        .args([
            "--server-url",
            "http://example.invalid",
            "--output-dir",
            out_path.to_str().unwrap(),
        ])
        .env_remove("JELLYFIN_API_KEY")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stdout.is_empty(),
        "expected no stdout output on failure, got: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}
