//! Unit tests for MovieEntry mapping/validation rules, per data-model.md.

use jellyfin_catalog_export::catalog::model::{
    MIN_PLAUSIBLE_YEAR, build_snapshot, extension_for_content_type, map_movie,
};
use jellyfin_catalog_export::jellyfin::client::JellyfinClient;
use jellyfin_catalog_export::jellyfin::models::JellyfinItem;

/// Every item here has no `image_tags`, so `build_snapshot` never dereferences this
/// client to fetch an image — a placeholder is safe to construct.
fn unused_client() -> JellyfinClient {
    JellyfinClient::new(
        reqwest::Url::parse("http://127.0.0.1:1/").unwrap(),
        "unused".to_string(),
    )
}

#[test]
fn build_snapshot_sorts_movies_alphabetically_by_title_case_insensitively() {
    let items = vec![
        item(Some("charlie"), None, None),
        item(Some("Alpha"), None, None),
        item(Some("bravo"), None, None),
    ];
    let staging = tempfile::tempdir().unwrap();

    let movies = build_snapshot(&items, &unused_client(), staging.path());

    let titles: Vec<&str> = movies.iter().map(|m| m.title.as_str()).collect();
    assert_eq!(titles, vec!["Alpha", "bravo", "charlie"]);
}

#[test]
fn picks_extension_from_content_type_and_falls_back_to_jpg() {
    assert_eq!(extension_for_content_type(Some("image/jpeg")), "jpg");
    assert_eq!(extension_for_content_type(Some("image/png")), "png");
    assert_eq!(extension_for_content_type(Some("image/webp")), "webp");
    assert_eq!(extension_for_content_type(Some("image/gif")), "gif");
    // Content-Type may carry parameters (e.g. a charset); the media type still matches.
    assert_eq!(
        extension_for_content_type(Some("image/png; charset=binary")),
        "png"
    );
    assert_eq!(
        extension_for_content_type(Some("application/octet-stream")),
        "jpg"
    );
    assert_eq!(extension_for_content_type(None), "jpg");
}

fn item(name: Option<&str>, year: Option<i32>, genres: Option<Vec<&str>>) -> JellyfinItem {
    JellyfinItem {
        id: "abc123".to_string(),
        name: name.map(|s| s.to_string()),
        production_year: year,
        genres: genres.map(|gs| gs.into_iter().map(String::from).collect()),
        image_tags: None,
    }
}

#[test]
fn drops_entries_with_empty_title() {
    let whitespace_only = item(Some("   "), Some(2020), None);
    assert!(map_movie(&whitespace_only, 2026).is_none());

    let missing = item(None, Some(2020), None);
    assert!(map_movie(&missing, 2026).is_none());
}

#[test]
fn trims_title_whitespace() {
    let m = item(Some("  Trimmed  "), None, None);
    let entry = map_movie(&m, 2026).unwrap();
    assert_eq!(entry.title, "Trimmed");
}

#[test]
fn keeps_plausible_release_year() {
    let m = item(Some("Example"), Some(2019), None);
    let entry = map_movie(&m, 2026).unwrap();
    assert_eq!(entry.release_year, Some(2019));
}

#[test]
fn drops_out_of_range_release_year_but_keeps_the_entry() {
    let too_early = item(Some("Ancient"), Some(MIN_PLAUSIBLE_YEAR - 1), None);
    let entry = map_movie(&too_early, 2026).unwrap();
    assert_eq!(entry.release_year, None);

    let too_late = item(Some("Future"), Some(2028), None);
    let entry = map_movie(&too_late, 2026).unwrap();
    assert_eq!(entry.release_year, None);
}

#[test]
fn filters_empty_genre_strings_and_omits_an_all_empty_list() {
    let with_blank_genres = item(Some("Example"), None, Some(vec!["Drama", "", "  "]));
    let entry = map_movie(&with_blank_genres, 2026).unwrap();
    assert_eq!(entry.genres, Some(vec!["Drama".to_string()]));

    let all_blank_genres = item(Some("Example"), None, Some(vec!["", "  "]));
    let entry = map_movie(&all_blank_genres, 2026).unwrap();
    assert_eq!(entry.genres, None);
}
