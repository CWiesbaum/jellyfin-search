//! Raw Jellyfin REST API response types this tool consumes. Internal only — see
//! `specs/001-remote-movie-catalog/data-model.md`'s `JellyfinMovieItem` note: none of
//! this is published as-is in the generated site.

use serde::Deserialize;

/// A Jellyfin `/Items` list response.
#[derive(Debug, Deserialize)]
pub struct ItemsResponse {
    /// The page of items returned by this response.
    #[serde(rename = "Items")]
    pub items: Vec<JellyfinItem>,
    /// The total number of items in the library, used to detect when pagination
    /// is complete.
    #[serde(rename = "TotalRecordCount")]
    pub total_record_count: i64,
}

/// A single raw Jellyfin item, as returned by `/Items` for a movie.
#[derive(Debug, Deserialize, Clone)]
pub struct JellyfinItem {
    /// Jellyfin's internal item id, used only to fetch this item's image.
    #[serde(rename = "Id")]
    pub id: String,
    /// The movie's title, if Jellyfin reports one.
    #[serde(rename = "Name")]
    pub name: Option<String>,
    /// The movie's release year, if Jellyfin reports one.
    #[serde(rename = "ProductionYear")]
    pub production_year: Option<i32>,
    /// The movie's genres, if Jellyfin reports any.
    #[serde(rename = "Genres")]
    pub genres: Option<Vec<String>>,
    /// Presence/tag information for this item's images.
    #[serde(rename = "ImageTags")]
    pub image_tags: Option<ImageTags>,
}

/// The subset of a Jellyfin item's `ImageTags` this tool cares about.
#[derive(Debug, Deserialize, Clone)]
pub struct ImageTags {
    /// The primary (poster/thumbnail) image's tag, if one is set.
    #[serde(rename = "Primary")]
    pub primary: Option<String>,
}

/// A Jellyfin library ("virtual folder"), as returned by `/Library/VirtualFolders`.
#[derive(Debug, Deserialize, Clone)]
pub struct VirtualFolder {
    /// The library's display name, matched against `--library-name`.
    #[serde(rename = "Name")]
    pub name: String,
    /// The library's item id, used as `ParentId` when listing its movies.
    #[serde(rename = "ItemId")]
    pub item_id: String,
    /// Jellyfin's collection type for this library (e.g. `"movies"`).
    #[serde(rename = "CollectionType")]
    pub collection_type: Option<String>,
}
