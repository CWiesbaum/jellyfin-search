//! Entry point Cargo discovers for unit tests; each submodule lives at the exact
//! path referenced from specs/001-remote-movie-catalog/tasks.md.

#[path = "unit/test_catalog_model.rs"]
mod test_catalog_model;

#[path = "unit/test_atomic_write.rs"]
mod test_atomic_write;

#[path = "unit/test_template.rs"]
mod test_template;
