//! Entry point Cargo discovers for integration tests; each submodule lives at the
//! exact path referenced from specs/001-remote-movie-catalog/tasks.md.
//! Submodules are added incrementally as each user-story phase reaches its tests.

#[path = "integration/test_us1_search.rs"]
mod test_us1_search;

#[path = "integration/test_us2_browse.rs"]
mod test_us2_browse;

#[path = "integration/test_us3_freshness.rs"]
mod test_us3_freshness;

#[path = "integration/test_us3_resilience.rs"]
mod test_us3_resilience;

#[path = "integration/test_library_selection.rs"]
mod test_library_selection;

#[path = "integration/test_credential_leakage.rs"]
mod test_credential_leakage;

#[path = "integration/test_pagination.rs"]
mod test_pagination;

#[path = "integration/test_write_error.rs"]
mod test_write_error;

#[path = "integration/test_empty_library.rs"]
mod test_empty_library;

#[path = "integration/test_stdout_contract.rs"]
mod test_stdout_contract;

#[path = "integration/test_image_tag_query_param.rs"]
mod test_image_tag_query_param;
