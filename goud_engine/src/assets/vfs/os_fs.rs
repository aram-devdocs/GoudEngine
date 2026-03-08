//! OS filesystem implementation of [`VirtualFs`].
//!
//! [`OsFs`] delegates to `std::fs` and resolves all paths relative to a
//! configured root directory, preserving the existing `AssetServer` behavior.

use super::VirtualFs;
use crate::assets::AssetLoadError;
use std::path::PathBuf;

/// Virtual filesystem backed by the operating system's native file I/O.
///
/// All paths passed to [`VirtualFs`] methods are joined with the root
/// directory before accessing the OS filesystem.
///
/// # Example
///
/// ```
/// use goud_engine::assets::vfs::OsFs;
///
/// let fs = OsFs::new("assets");
/// assert_eq!(fs.root().to_str().unwrap(), "assets");
/// ```
#[derive(Debug, Clone)]
pub struct OsFs {
    root: PathBuf,
}

impl OsFs {
    /// Creates a new `OsFs` rooted at the given directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the root directory.
    #[inline]
    pub fn root(&self) -> &std::path::Path {
        &self.root
    }
}

impl VirtualFs for OsFs {
    fn read(&self, path: &str) -> Result<Vec<u8>, AssetLoadError> {
        let full_path = self.root.join(path);
        std::fs::read(&full_path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AssetLoadError::not_found(&full_path)
            } else {
                AssetLoadError::io_error(&full_path, e)
            }
        })
    }

    fn exists(&self, path: &str) -> bool {
        self.root.join(path).exists()
    }

    fn list(&self, directory: &str) -> Result<Vec<String>, AssetLoadError> {
        let full_path = self.root.join(directory);
        let entries = std::fs::read_dir(&full_path)
            .map_err(|e| AssetLoadError::io_error(&full_path, e))?;

        let mut result = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| AssetLoadError::io_error(&full_path, e))?;
            if let Some(name) = entry.file_name().to_str() {
                if directory.is_empty() {
                    result.push(name.to_string());
                } else {
                    result.push(format!("{}/{}", directory, name));
                }
            }
        }
        result.sort();
        Ok(result)
    }
}
