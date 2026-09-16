//! Library crate exposing the export pipeline for both the CLI binary and the test
//! suite. See `specs/001-remote-movie-catalog/plan.md` and `contracts/` for the design
//! this implements.
#![warn(missing_docs)]

pub mod catalog;
pub mod cli;
pub mod jellyfin;
pub mod site;

use cli::Cli;
use jellyfin::client::JellyfinClient;

/// Successful run: a complete new snapshot was written to `--output-dir`.
pub const EXIT_SUCCESS: i32 = 0;
/// Configuration/argument error (missing required flag, no API key, invalid `--server-url`).
pub const EXIT_CONFIG_ERROR: i32 = 1;
/// Jellyfin connection or authentication error.
pub const EXIT_CONNECTION_ERROR: i32 = 2;
/// Output write error.
pub const EXIT_WRITE_ERROR: i32 = 3;

/// Runs the full export -> map -> render -> write pipeline for the given CLI
/// arguments, returning the process exit code to use, per `contracts/cli-interface.md`.
pub fn run(cli: &Cli) -> i32 {
    let api_key = match cli.resolve_api_key() {
        Ok(key) => key,
        Err(_) => {
            log::error!(
                "no Jellyfin API key available: pass --api-key or set {}",
                cli::API_KEY_ENV_VAR
            );
            return EXIT_CONFIG_ERROR;
        }
    };

    let base_url = match normalize_base_url(&cli.server_url) {
        Ok(url) => url,
        Err(e) => {
            log::error!("invalid --server-url '{}': {e}", cli.server_url);
            return EXIT_CONFIG_ERROR;
        }
    };

    let client = JellyfinClient::new(base_url, api_key);

    // Nothing under --output-dir is touched until after the Jellyfin fetch succeeds,
    // so a connection/auth failure here never disturbs a previously published site.
    let library = match client.find_movie_library(cli.library_name.as_deref()) {
        Ok(library) => library,
        Err(e) => {
            log::error!("could not find a movie library on the Jellyfin server: {e}");
            return EXIT_CONNECTION_ERROR;
        }
    };
    log::info!("using movie library '{}'", library.name);

    let items = match client.fetch_all_movies(&library.item_id) {
        Ok(items) => items,
        Err(e) => {
            log::error!("could not fetch movies from Jellyfin: {e}");
            return EXIT_CONNECTION_ERROR;
        }
    };
    log::info!("fetched {} movie(s) from Jellyfin", items.len());

    let staging_dir = site::generator::staging_dir_for(&cli.output_dir);
    if let Err(e) = std::fs::create_dir_all(&staging_dir) {
        log::error!(
            "could not create staging directory {}: {e}",
            staging_dir.display()
        );
        return EXIT_WRITE_ERROR;
    }

    let movies = catalog::model::build_snapshot(&items, &client, &staging_dir);
    log::info!("mapped {} movie(s) into the catalog snapshot", movies.len());
    let snapshot = catalog::model::CatalogSnapshot {
        generated_at: catalog::model::now_rfc3339(),
        movies,
    };

    let result = site::generator::write_site(&snapshot, &staging_dir)
        .and_then(|_| site::generator::publish(&staging_dir, &cli.output_dir));

    match result {
        Ok(()) => {
            println!(
                "exported {} movie(s) to {}",
                snapshot.movies.len(),
                cli.output_dir.display()
            );
            EXIT_SUCCESS
        }
        Err(e) => {
            log::error!(
                "could not write output to {}: {e}",
                cli.output_dir.display()
            );
            let _ = std::fs::remove_dir_all(&staging_dir);
            EXIT_WRITE_ERROR
        }
    }
}

/// Parses `raw` as a URL and ensures its path ends with `/`, so relative Jellyfin API
/// paths resolve correctly when joined against it.
fn normalize_base_url(raw: &str) -> Result<reqwest::Url, String> {
    let mut url = reqwest::Url::parse(raw).map_err(|e| e.to_string())?;
    if !url.path().ends_with('/') {
        let new_path = format!("{}/", url.path());
        url.set_path(&new_path);
    }
    Ok(url)
}
