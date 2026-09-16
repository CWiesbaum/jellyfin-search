//! Unit test for template data-substitution, per research.md §6.

use jellyfin_catalog_export::catalog::model::{CatalogSnapshot, MovieEntry};
use jellyfin_catalog_export::site::template::{CATALOG_DATA_MARKER, render};

#[test]
fn substitutes_marker_with_serialized_snapshot() {
    let snapshot = CatalogSnapshot {
        generated_at: "2026-09-16T00:00:00.000Z".to_string(),
        movies: vec![MovieEntry {
            title: "Example".to_string(),
            release_year: Some(2020),
            genres: None,
            thumbnail: None,
        }],
    };

    let html = render(&snapshot).unwrap();

    assert!(!html.contains(CATALOG_DATA_MARKER));
    assert!(html.contains("\"Example\""));
    assert!(html.contains("\"generatedAt\":\"2026-09-16T00:00:00.000Z\""));
}

#[test]
fn escapes_script_close_sequences_in_embedded_data() {
    let snapshot = CatalogSnapshot {
        generated_at: "2026-09-16T00:00:00.000Z".to_string(),
        movies: vec![MovieEntry {
            title: "</script><script>alert(1)</script>".to_string(),
            release_year: None,
            genres: None,
            thumbnail: None,
        }],
    };

    let html = render(&snapshot).unwrap();

    assert!(!html.contains("</script><script>alert"));
}

/// Regression guard for FR-006 ("no further requests back to any server at runtime"):
/// the generated page must never issue a fetch/XHR call, since its data is embedded
/// inline (research.md §4) and everything else it loads (images, styles) is a plain
/// same-origin static asset the browser fetches implicitly, not a JS-driven API call.
#[test]
fn generated_page_contains_no_runtime_network_calls() {
    let snapshot = CatalogSnapshot {
        generated_at: "2026-09-16T00:00:00.000Z".to_string(),
        movies: vec![MovieEntry {
            title: "Example".to_string(),
            release_year: Some(2020),
            genres: Some(vec!["Drama".to_string()]),
            thumbnail: Some("images/1.jpg".to_string()),
        }],
    };

    let html = render(&snapshot).unwrap();

    assert!(!html.contains("fetch("));
    assert!(!html.contains("fetch ("));
    assert!(!html.contains("XMLHttpRequest"));
    assert!(!html.contains("WebSocket("));
    assert!(!html.contains("EventSource("));
}
