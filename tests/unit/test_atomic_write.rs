//! Unit test for the atomic output-directory writer, per research.md §8 / spec FR-009.

use std::fs;

use jellyfin_catalog_export::site::generator::{publish, staging_dir_for};

#[test]
fn successful_publish_replaces_output_dir_contents() {
    let base = tempfile::tempdir().unwrap();
    let output_dir = base.path().join("out");
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("old.txt"), b"old").unwrap();

    let staging = staging_dir_for(&output_dir);
    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join("new.txt"), b"new").unwrap();

    publish(&staging, &output_dir).unwrap();

    assert!(output_dir.join("new.txt").exists());
    assert!(!output_dir.join("old.txt").exists());
    assert!(!staging.exists());
}

#[test]
fn failed_publish_leaves_output_dir_completely_untouched() {
    let base = tempfile::tempdir().unwrap();
    let output_dir = base.path().join("out");
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("old.txt"), b"old").unwrap();

    // A staging dir that does not exist makes the rename fail.
    let missing_staging = base.path().join("does-not-exist");

    let result = publish(&missing_staging, &output_dir);

    assert!(result.is_err());
    assert!(output_dir.join("old.txt").exists());
    assert_eq!(fs::read(output_dir.join("old.txt")).unwrap(), b"old");
}
