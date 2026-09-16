//! A blocking client for the small slice of the Jellyfin REST API this tool needs,
//! per `specs/001-remote-movie-catalog/research.md` §2. Deliberately uses
//! `reqwest::blocking` rather than an async runtime — see research.md §1.

use std::fmt;
use std::time::Duration;

use reqwest::Url;
use reqwest::blocking::Client;

use crate::jellyfin::models::{ItemsResponse, JellyfinItem, VirtualFolder};

const API_TOKEN_HEADER: &str = "X-Emby-Token";
const PAGE_SIZE: usize = 200;

/// Errors talking to a Jellyfin server.
#[derive(Debug)]
pub enum JellyfinError {
    /// The HTTP request itself failed (connection, TLS, timeout, etc.).
    Request(reqwest::Error),
    /// Jellyfin responded with a non-2xx HTTP status.
    Api {
        /// The HTTP status code Jellyfin returned.
        status: u16,
        /// The response body, if any, for diagnostic purposes.
        body: String,
    },
    /// `--library-name` was given but no movie library with that name exists.
    LibraryNotFound(String),
    /// No movie library exists on the server at all.
    NoMovieLibrary,
}

impl fmt::Display for JellyfinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JellyfinError::Request(e) => write!(f, "request failed: {e}"),
            JellyfinError::Api { status, body } => {
                write!(f, "Jellyfin returned HTTP {status}: {body}")
            }
            JellyfinError::LibraryNotFound(name) => {
                write!(f, "no movie library named '{name}' was found")
            }
            JellyfinError::NoMovieLibrary => {
                write!(f, "no movie library was found on the server")
            }
        }
    }
}

impl std::error::Error for JellyfinError {}

impl From<reqwest::Error> for JellyfinError {
    fn from(e: reqwest::Error) -> Self {
        JellyfinError::Request(e)
    }
}

/// A blocking Jellyfin REST API client, authenticated with an API key.
pub struct JellyfinClient {
    base_url: Url,
    api_key: String,
    http: Client,
}

impl JellyfinClient {
    /// Builds a client for `base_url` (a trailing slash is recommended so relative
    /// request paths resolve correctly) authenticated with `api_key`.
    pub fn new(base_url: Url, api_key: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build the Jellyfin HTTP client");
        Self {
            base_url,
            api_key,
            http,
        }
    }

    fn get(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<reqwest::blocking::Response, JellyfinError> {
        let url = self.base_url.join(path).map_err(|e| JellyfinError::Api {
            status: 0,
            body: format!("invalid request path '{path}': {e}"),
        })?;
        let response = self
            .http
            .get(url)
            .header(API_TOKEN_HEADER, &self.api_key)
            .query(query)
            .send()?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            return Err(JellyfinError::Api { status, body });
        }
        Ok(response)
    }

    /// Finds the movie library to export: the one named `library_name` if given,
    /// otherwise the first movie library on the server.
    pub fn find_movie_library(
        &self,
        library_name: Option<&str>,
    ) -> Result<VirtualFolder, JellyfinError> {
        let response = self.get("Library/VirtualFolders", &[])?;
        let folders: Vec<VirtualFolder> = response.json()?;
        let mut movie_folders = folders
            .into_iter()
            .filter(|f| f.collection_type.as_deref() == Some("movies"));

        match library_name {
            Some(name) => movie_folders
                .find(|f| f.name == name)
                .ok_or_else(|| JellyfinError::LibraryNotFound(name.to_string())),
            None => movie_folders.next().ok_or(JellyfinError::NoMovieLibrary),
        }
    }

    /// Fetches every movie item under the given library (parent item id), paginating
    /// as needed.
    pub fn fetch_all_movies(&self, parent_id: &str) -> Result<Vec<JellyfinItem>, JellyfinError> {
        let mut items = Vec::new();
        let mut start_index = 0usize;

        loop {
            let start_str = start_index.to_string();
            let limit_str = PAGE_SIZE.to_string();
            let response = self.get(
                "Items",
                &[
                    ("ParentId", parent_id),
                    ("IncludeItemTypes", "Movie"),
                    ("Recursive", "true"),
                    ("Fields", "Genres,ProductionYear,ImageTags"),
                    ("StartIndex", &start_str),
                    ("Limit", &limit_str),
                ],
            )?;
            let page: ItemsResponse = response.json()?;
            let fetched = page.items.len();
            items.extend(page.items);
            start_index += fetched;

            if fetched == 0 || (start_index as i64) >= page.total_record_count {
                break;
            }
        }

        Ok(items)
    }

    /// Downloads a movie's primary (poster/thumbnail) image, along with the
    /// `Content-Type` Jellyfin reported for it (used to pick a correct file
    /// extension — Jellyfin posters are not always JPEG). `tag` should be the item's
    /// `ImageTags.Primary` value, when known, passed as Jellyfin's `tag` query
    /// parameter for cache-busting/consistency (research.md §2).
    pub fn fetch_primary_image(
        &self,
        item_id: &str,
        tag: Option<&str>,
    ) -> Result<FetchedImage, JellyfinError> {
        let query: &[(&str, &str)] = match tag {
            Some(tag) => &[("tag", tag)],
            None => &[],
        };
        let response = self.get(&format!("Items/{item_id}/Images/Primary"), query)?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let bytes = response.bytes()?.to_vec();
        Ok(FetchedImage {
            bytes,
            content_type,
        })
    }
}

/// The bytes of a downloaded image plus the `Content-Type` the server reported for it.
#[derive(Debug, Clone)]
pub struct FetchedImage {
    /// The raw image bytes.
    pub bytes: Vec<u8>,
    /// The `Content-Type` response header, when the server sent one.
    pub content_type: Option<String>,
}
