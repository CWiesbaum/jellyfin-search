//! Writes the generated site to disk and atomically publishes it, per
//! `specs/001-remote-movie-catalog/research.md` §8 / spec FR-009.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::catalog::model::CatalogSnapshot;
use crate::site::template;

/// Renders `snapshot` and writes `index.html` into `staging_dir` (which must already
/// exist).
pub fn write_site(snapshot: &CatalogSnapshot, staging_dir: &Path) -> io::Result<()> {
    let html =
        template::render(snapshot).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(staging_dir.join("index.html"), html)
}

/// Returns a fresh, not-yet-existing staging directory path that is a sibling of
/// `output_dir`, so it can later be renamed onto it atomically on the same filesystem.
pub fn staging_dir_for(output_dir: &Path) -> PathBuf {
    let file_name = output_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let sibling_name = format!(".{file_name}.tmp-{pid}-{nanos}");
    match output_dir.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(sibling_name),
        _ => PathBuf::from(sibling_name),
    }
}

/// Atomically replaces `output_dir`'s contents with `staging_dir`'s contents.
/// On success, `output_dir` now holds what was in `staging_dir` and `staging_dir` no
/// longer exists. On any failure, `output_dir` is left completely untouched.
pub fn publish(staging_dir: &Path, output_dir: &Path) -> io::Result<()> {
    if let Some(parent) = output_dir.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let mut backup_name = staging_dir.file_name().unwrap_or_default().to_os_string();
    backup_name.push(".previous");
    let backup_dir = staging_dir.with_file_name(backup_name);

    let had_previous = output_dir.exists();
    if had_previous {
        fs::rename(output_dir, &backup_dir)?;
    }

    match fs::rename(staging_dir, output_dir) {
        Ok(()) => {
            if had_previous {
                let _ = fs::remove_dir_all(&backup_dir);
            }
            Ok(())
        }
        Err(e) => {
            if had_previous {
                let _ = fs::rename(&backup_dir, output_dir);
            }
            Err(e)
        }
    }
}
