//! Integration test for User Story 1: search/filter by title, per
//! specs/001-remote-movie-catalog/spec.md (User Story 1) and
//! contracts/movie-data-schema.md. Includes a movie with missing metadata (T028) to
//! confirm search-supporting rendering doesn't break on incomplete entries.

use std::fs;
use std::process::Command;

use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jellyfin-catalog-export"))
}

#[tokio::test]
async fn generated_page_embeds_all_movie_titles_and_a_search_input() {
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
                { "Id": "1", "Name": "Alpha Movie", "ProductionYear": 2001, "Genres": ["Drama"] },
                { "Id": "2", "Name": "Beta Movie", "ProductionYear": 2010, "Genres": ["Comedy"] },
                { "Id": "3", "Name": "Gamma Movie With No Extra Metadata" }
            ],
            "TotalRecordCount": 3
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
    assert!(status.success());

    let html = fs::read_to_string(out_path.join("index.html")).unwrap();
    assert!(html.contains("Alpha Movie"));
    assert!(html.contains("Beta Movie"));
    assert!(html.contains("Gamma Movie With No Extra Metadata"));
    assert!(html.contains(r#"id="search""#));
    assert!(html.contains(r#"id="catalog-data""#));
}
