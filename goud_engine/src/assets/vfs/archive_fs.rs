//! Stub archive filesystem — placeholder for future archive-based asset loading.
//!
//! All methods return [`AssetLoadError::NotFound`] until a real archive
//! format is integrated.

use super::VirtualFs;
use crate::assets::AssetLoadError;

/// A virtual filesystem backed by an archive file (e.g., ZIP, PAK).
///
/// This is currently a stub. All operations return `NotFound`. A real
/// implementation will be added by issue #220.
#[derive(Debug)]
pub struct ArchiveFs {
    /// Path to the archive file (stored for future use).
    _archive_path: String,
}

impl ArchiveFs {
    /// Creates a new `ArchiveFs` pointing at the given archive file.
    pub fn new(archive_path: impl Into<String>) -> Self {
        Self {
            _archive_path: archive_path.into(),
        }
    }
}

impl VirtualFs for ArchiveFs {
    fn read(&self, path: &str) -> Result<Vec<u8>, AssetLoadError> {
        Err(AssetLoadError::not_found(path))
    }

    fn exists(&self, _path: &str) -> bool {
        false
    }

    fn list(&self, directory: &str) -> Result<Vec<String>, AssetLoadError> {
        Err(AssetLoadError::not_found(directory))
    }
}
