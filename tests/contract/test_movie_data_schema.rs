//! Contract test for the embedded catalog JSON schema, per
//! specs/001-remote-movie-catalog/contracts/movie-data-schema.md.

use jellyfin_catalog_export::catalog::model::{CatalogSnapshot, MovieEntry};

#[test]
fn omits_absent_optional_fields_entirely() {
    let snapshot = CatalogSnapshot {
        generated_at: "2026-09-16T00:00:00.000Z".to_string(),
        movies: vec![MovieEntry {
            title: "Bare Movie".to_string(),
            release_year: None,
            genres: None,
            thumbnail: None,
        }],
    };

    let json = serde_json::to_value(&snapshot).unwrap();
    let movie = &json["movies"][0];

    assert!(movie.get("releaseYear").is_none());
    assert!(movie.get("genres").is_none());
    assert!(movie.get("thumbnail").is_none());
    assert_eq!(movie["title"], "Bare Movie");
}

#[test]
fn includes_present_optional_fields() {
    let snapshot = CatalogSnapshot {
        generated_at: "2026-09-16T00:00:00.000Z".to_string(),
        movies: vec![MovieEntry {
            title: "Full Movie".to_string(),
            release_year: Some(2021),
            genres: Some(vec!["Comedy".to_string()]),
            thumbnail: Some("images/1.jpg".to_string()),
        }],
    };

    let json = serde_json::to_value(&snapshot).unwrap();
    let movie = &json["movies"][0];

    assert_eq!(movie["releaseYear"], 2021);
    assert_eq!(movie["genres"], serde_json::json!(["Comedy"]));
    assert_eq!(movie["thumbnail"], "images/1.jpg");
}
