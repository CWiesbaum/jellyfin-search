//! Smoke test for the published Docker image, per specs/003-docker-image-publish. Reuses
//! the same wiremock-backed mock Jellyfin server pattern as test_us1_search.rs, but invokes
//! `docker run <image>` instead of the native binary. Only runs when
//! JCE_DOCKER_SMOKE_IMAGE is set to an image tag to test — plain `cargo test` runs skip it,
//! since they don't have Docker or a built image available.

use std::env;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn docker_image_runs_full_export_against_mock_jellyfin_server() {
    let Ok(image) = env::var("JCE_DOCKER_SMOKE_IMAGE") else {
        eprintln!("skipping: JCE_DOCKER_SMOKE_IMAGE not set");
        return;
    };

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
                { "Id": "1", "Name": "Alpha Movie", "ProductionYear": 2001, "Genres": ["Drama"] }
            ],
            "TotalRecordCount": 1
        })))
        .mount(&server)
        .await;

    let output_dir = tempfile::tempdir().unwrap();

    // --network host lets the container reach the mock server on the host's loopback
    // interface at the ephemeral port wiremock bound (Linux-only, matches the CI runners
    // and devcontainer this feature targets). The volume is mounted at a parent directory
    // (/data) rather than directly at --output-dir: the CLI's atomic publish step
    // (src/site/generator.rs::publish) does `rename(staging_dir, output_dir)`, and you
    // cannot rename onto an active bind-mount point (EBUSY) — only onto a plain
    // subdirectory within one (contracts/docker-image-contract.md's Volumes section).
    // --user matches the mounted directory's own owning uid/gid, overriding the image's
    // default "nobody" user, so the process can actually write into a host-owned bind
    // mount (its uid otherwise has no relation to the mounted directory's owner).
    let meta = fs::metadata(output_dir.path()).unwrap();
    let status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "host",
            "--user",
            &format!("{}:{}", meta.uid(), meta.gid()),
            "-v",
            &format!("{}:/data", output_dir.path().display()),
            &image,
            "--server-url",
            &server.uri(),
            "--output-dir",
            "/data/site",
            "--api-key",
            "test-key",
        ])
        .status()
        .unwrap();
    assert!(
        status.success(),
        "docker run exited non-zero for image {image}"
    );

    let html = fs::read_to_string(output_dir.path().join("site").join("index.html")).unwrap();
    assert!(html.contains("Alpha Movie"));
}
