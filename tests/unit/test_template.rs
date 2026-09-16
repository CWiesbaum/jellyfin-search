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

/// Per spec.md FR-006 (002-ascii-terminal-theme): the terminal theme MUST be fixed,
/// not conditional on the visitor's OS/browser light/dark preference, so the old
/// `prefers-color-scheme` media query must be gone entirely, not just supplemented.
#[test]
fn no_prefers_color_scheme_media_query_remains() {
    let snapshot = empty_snapshot();
    let html = render(&snapshot).unwrap();

    assert!(!html.contains("prefers-color-scheme"));
}

/// Per data-model.md's Presentation Tokens table (002-ascii-terminal-theme).
#[test]
fn declares_the_fixed_terminal_palette_custom_properties() {
    let snapshot = empty_snapshot();
    let html = render(&snapshot).unwrap();

    assert!(html.contains("--bg: #000000") || html.contains("--bg:#000000"));
    assert!(html.contains("--fg: #33ff33") || html.contains("--fg:#33ff33"));
    assert!(html.contains("--fg-dim: #1a8c1a") || html.contains("--fg-dim:#1a8c1a"));
}

/// Per FR-002 (002-ascii-terminal-theme): the page uses a monospace font throughout.
#[test]
fn declares_a_monospace_font_family() {
    let snapshot = empty_snapshot();
    let html = render(&snapshot).unwrap();

    assert!(html.contains("monospace"));
}

/// Per FR-004 / research.md §1 (002-ascii-terminal-theme): a hand-authored ASCII-art
/// banner appears near the top of the page, rendered as preformatted monospace text.
#[test]
fn renders_an_ascii_art_banner_near_the_top_of_the_page() {
    let snapshot = empty_snapshot();
    let html = render(&snapshot).unwrap();

    let banner_start = html.find("<pre").expect("expected a <pre> banner element");
    let body_start = html.find("<body").unwrap_or(0);
    assert!(banner_start > body_start, "banner should be inside <body>");
    assert!(html.contains("MOVIE LIBRARY"));

    // The banner must appear before the header/search UI, not after it.
    let header_start = html.find("<header").expect("expected a <header> element");
    assert!(
        banner_start < header_start,
        "banner should appear near the top of the page"
    );
}

/// Per research.md §1: the fixed-width ASCII banner cannot reflow like the responsive
/// grid, so it must not be able to force the whole page to scroll horizontally on
/// narrow (~320px) phone viewports.
#[test]
fn banner_has_overflow_handling_for_narrow_viewports() {
    let snapshot = empty_snapshot();
    let html = render(&snapshot).unwrap();

    let style_start = html.find("<style>").unwrap();
    let style_end = html.find("</style>").unwrap();
    let style_block = &html[style_start..style_end];
    let banner_rule_start = style_block
        .find("#banner")
        .expect("expected a #banner rule");
    let banner_rule_end = banner_rule_start
        + style_block[banner_rule_start..]
            .find('}')
            .expect("expected #banner rule to close");
    let banner_rule = &style_block[banner_rule_start..banner_rule_end];

    assert!(
        banner_rule.contains("overflow-x"),
        "#banner rule should declare overflow-x handling, got: {banner_rule}"
    );
}

fn empty_snapshot() -> CatalogSnapshot {
    CatalogSnapshot {
        generated_at: "2026-09-16T00:00:00.000Z".to_string(),
        movies: vec![],
    }
}
