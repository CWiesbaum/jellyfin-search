//! Domain types and mapping/validation logic, per
//! `specs/001-remote-movie-catalog/data-model.md`.

use std::path::Path;

use chrono::Datelike;
use serde::Serialize;

use crate::jellyfin::client::JellyfinClient;
use crate::jellyfin::models::JellyfinItem;

/// Earliest release year treated as plausible.
pub const MIN_PLAUSIBLE_YEAR: i32 = 1870;

/// A single movie as shown in the overview. See data-model.md's `MovieEntry` and
/// `contracts/movie-data-schema.md` for the field contract.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MovieEntry {
    /// The movie's title. Never empty.
    pub title: String,
    /// The movie's release year, when known and plausible.
    #[serde(rename = "releaseYear", skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i32>,
    /// The movie's genres, when Jellyfin reports at least one non-empty genre.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genres: Option<Vec<String>>,
    /// Path to the downloaded thumbnail, relative to `index.html`, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

/// The full exported library as embedded in the generated page. See data-model.md's
/// `CatalogSnapshot` and `contracts/movie-data-schema.md`.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogSnapshot {
    /// When this export run completed, as an RFC 3339 UTC timestamp.
    #[serde(rename = "generatedAt")]
    pub generated_at: String,
    /// Every movie included in this snapshot.
    pub movies: Vec<MovieEntry>,
}

/// Maps a raw Jellyfin item into a `MovieEntry`, enforcing data-model.md's validation
/// rules. Returns `None` when the item has no usable title; the caller is responsible
/// for logging that a movie was skipped.
pub fn map_movie(item: &JellyfinItem, current_year: i32) -> Option<MovieEntry> {
    let title = item.name.as_deref().unwrap_or("").trim();
    if title.is_empty() {
        return None;
    }

    let release_year = item
        .production_year
        .filter(|&year| (MIN_PLAUSIBLE_YEAR..=current_year + 1).contains(&year));

    let genres = item.genres.as_ref().map(|genres| {
        genres
            .iter()
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .collect::<Vec<_>>()
    });
    let genres = genres.filter(|g| !g.is_empty());

    Some(MovieEntry {
        title: title.to_string(),
        release_year,
        genres,
        thumbnail: None,
    })
}

/// Maps every item and downloads each mapped movie's thumbnail (when Jellyfin reports
/// one) into `images/` under `staging_dir`. A download failure is logged and leaves
/// that entry's thumbnail absent rather than a dangling reference (data-model.md).
pub fn build_snapshot(
    items: &[JellyfinItem],
    client: &JellyfinClient,
    staging_dir: &Path,
) -> Vec<MovieEntry> {
    let current_year = chrono::Utc::now().year();
    let images_dir = staging_dir.join("images");

    let mut movies: Vec<MovieEntry> = items
        .iter()
        .filter_map(|item| {
            let mut entry = match map_movie(item, current_year) {
                Some(entry) => entry,
                None => {
                    log::warn!("skipping movie with empty title (Jellyfin id={})", item.id);
                    return None;
                }
            };

            let image_tag = item
                .image_tags
                .as_ref()
                .and_then(|tags| tags.primary.as_deref());

            if let Some(image_tag) = image_tag {
                match client.fetch_primary_image(&item.id, Some(image_tag)) {
                    Ok(image) => {
                        let extension = extension_for_content_type(image.content_type.as_deref());
                        match write_thumbnail(&images_dir, &item.id, extension, &image.bytes) {
                            Ok(relative_path) => entry.thumbnail = Some(relative_path),
                            Err(e) => {
                                log::warn!("could not save thumbnail for '{}': {e}", entry.title)
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("could not download thumbnail for '{}': {e}", entry.title)
                    }
                }
            }

            Some(entry)
        })
        .collect();

    // Export order from Jellyfin isn't meaningful to a browsing user; sort so the
    // generated page is predictably browsable "at a glance" (User Story 2). See
    // contracts/movie-data-schema.md's "Consumer expectations" note.
    movies.sort_by_key(|entry| entry.title.to_lowercase());

    movies
}

/// Maps a `Content-Type` header value to a file extension for a downloaded thumbnail,
/// falling back to `jpg` when the type is absent or not a recognized image format.
pub fn extension_for_content_type(content_type: Option<&str>) -> &'static str {
    let media_type = content_type
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase());

    match media_type.as_deref() {
        Some("image/png") => "png",
        Some("image/webp") => "webp",
        Some("image/gif") => "gif",
        Some("image/bmp") => "bmp",
        _ => "jpg",
    }
}

fn write_thumbnail(
    images_dir: &Path,
    item_id: &str,
    extension: &str,
    bytes: &[u8],
) -> std::io::Result<String> {
    std::fs::create_dir_all(images_dir)?;
    let file_name = format!("{item_id}.{extension}");
    std::fs::write(images_dir.join(&file_name), bytes)?;
    Ok(format!("images/{file_name}"))
}

/// The current UTC time as an RFC 3339 timestamp with millisecond precision, used as
/// `CatalogSnapshot.generated_at`.
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}
