//! Command-line argument parsing. See
//! `specs/001-remote-movie-catalog/contracts/cli-interface.md` for the full contract.

use std::path::PathBuf;

use clap::Parser;

/// Environment variable checked for the Jellyfin API key when `--api-key` is omitted.
pub const API_KEY_ENV_VAR: &str = "JELLYFIN_API_KEY";

/// Command-line arguments for the Jellyfin catalog export tool.
#[derive(Debug, Parser)]
#[command(
    name = "jellyfin-catalog-export",
    about = "Export a Jellyfin movie library to a self-contained static site"
)]
pub struct Cli {
    /// Base URL of the Jellyfin server to export from.
    #[arg(long)]
    pub server_url: String,

    /// Jellyfin API key. Falls back to the JELLYFIN_API_KEY environment variable.
    #[arg(long)]
    pub api_key: Option<String>,

    /// Local directory to write the generated static site into.
    #[arg(long)]
    pub output_dir: PathBuf,

    /// Selects a specific Jellyfin movie library by name (default: the first one found).
    #[arg(long)]
    pub library_name: Option<String>,
}

/// Returned when neither `--api-key` nor `JELLYFIN_API_KEY` resolves to a value.
#[derive(Debug)]
pub struct MissingApiKey;

impl Cli {
    /// Resolves the API key from `--api-key`, falling back to `JELLYFIN_API_KEY`.
    pub fn resolve_api_key(&self) -> Result<String, MissingApiKey> {
        if let Some(key) = &self.api_key {
            return Ok(key.clone());
        }
        std::env::var(API_KEY_ENV_VAR).map_err(|_| MissingApiKey)
    }
}
