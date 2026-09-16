//! Contract test for CLI argument parsing, per
//! specs/001-remote-movie-catalog/contracts/cli-interface.md.

use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[test]
fn exits_1_when_server_url_missing() {
    let dir = tempfile::tempdir().unwrap();
    let status = binary()
        .args(["--output-dir", dir.path().to_str().unwrap()])
        .env("JELLYFIN_API_KEY", "test-key")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn exits_1_when_output_dir_missing() {
    let status = binary()
        .args(["--server-url", "http://example.invalid"])
        .env("JELLYFIN_API_KEY", "test-key")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn exits_1_when_no_api_key_available() {
    let dir = tempfile::tempdir().unwrap();
    let status = binary()
        .args([
            "--server-url",
            "http://example.invalid",
            "--output-dir",
            dir.path().to_str().unwrap(),
        ])
        .env_remove("JELLYFIN_API_KEY")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}

#[test]
fn exits_1_on_invalid_server_url() {
    let dir = tempfile::tempdir().unwrap();
    let status = binary()
        .args([
            "--server-url",
            "not a url",
            "--output-dir",
            dir.path().to_str().unwrap(),
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(1));
}
