//! Entry point Cargo discovers for contract tests; each submodule lives at the exact
//! path referenced from specs/001-remote-movie-catalog/tasks.md.

#[path = "contract/test_cli_interface.rs"]
mod test_cli_interface;

#[path = "contract/test_jellyfin_models.rs"]
mod test_jellyfin_models;

#[path = "contract/test_movie_data_schema.rs"]
mod test_movie_data_schema;
