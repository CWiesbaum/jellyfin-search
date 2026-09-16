//! Integration test for `JellyfinClient::fetch_all_movies`'s pagination loop across
//! more than one page, per research.md §2 (paginated `/Items` fetch via
//! `StartIndex`/`Limit`). The client's internal page size is 200 (`PAGE_SIZE` in
//! `src/jellyfin/client.rs`); this test uses a library just over that size so at
//! least two requests are required to fetch it all.

use std::fs;
use std::process::Command;

use serde_json::{Value, json};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

fn movies_page(start: usize, count: usize) -> Vec<Value> {
    (start..start + count)
        .map(|i| json!({ "Id": i.to_string(), "Name": format!("Movie {i}") }))
        .collect()
}

#[tokio::test]
async fn fetches_every_page_of_a_library_larger_than_the_page_size() {
    const PAGE_SIZE: usize = 200;
    const TOTAL: usize = PAGE_SIZE + 3;

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
        .and(query_param("StartIndex", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Items": movies_page(0, PAGE_SIZE),
            "TotalRecordCount": TOTAL
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Items"))
        .and(query_param("StartIndex", PAGE_SIZE.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "Items": movies_page(PAGE_SIZE, TOTAL - PAGE_SIZE),
            "TotalRecordCount": TOTAL
        })))
        .expect(1)
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
    // First page's first movie, and second page's last movie (index TOTAL - 1).
    assert!(html.contains("\"Movie 0\""));
    assert!(html.contains(&format!("\"Movie {}\"", TOTAL - 1)));

    server.verify().await;
}
