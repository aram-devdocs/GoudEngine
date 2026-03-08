//! Asset directory packager for creating distribution archives.

use crate::assets::vfs::archive_format::ArchiveWriter;
use crate::assets::AssetLoadError;
use std::path::Path;

/// Packages all files in a directory into a GOUD archive file.
///
/// Walks `input_dir` recursively, adding every file to the archive.
/// Paths are stored relative to `input_dir` with forward slashes.
///
/// # Arguments
///
/// * `input_dir` - Directory to recursively package
/// * `output_path` - Path where the archive file will be written
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use goud_engine::assets::packager;
///
/// packager::package_directory(Path::new("assets"), Path::new("game.goud")).unwrap();
/// ```
pub fn package_directory(input_dir: &Path, output_path: &Path) -> Result<(), AssetLoadError> {
    let mut writer = ArchiveWriter::new();
    collect_files(input_dir, input_dir, &mut writer)?;

    let mut file =
        std::fs::File::create(output_path).map_err(|e| AssetLoadError::io_error(output_path, e))?;
    writer.write_to(&mut file)
}

fn collect_files(
    base: &Path,
    dir: &Path,
    writer: &mut ArchiveWriter,
) -> Result<(), AssetLoadError> {
    let entries = std::fs::read_dir(dir).map_err(|e| AssetLoadError::io_error(dir, e))?;

    // Collect and sort for reproducibility
    let mut sorted_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    sorted_entries.sort_by_key(|e| e.path());

    for entry in sorted_entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files(base, &path, writer)?;
        } else if path.is_file() {
            let relative = path
                .strip_prefix(base)
                .map_err(|e| AssetLoadError::custom(format!("Path prefix error: {e}")))?;
            // Normalize to forward slashes
            let key = relative.to_string_lossy().replace('\\', "/");
            let data = std::fs::read(&path).map_err(|e| AssetLoadError::io_error(&path, e))?;
            writer.add_file(&key, &data);
        }
    }
    Ok(())
}
