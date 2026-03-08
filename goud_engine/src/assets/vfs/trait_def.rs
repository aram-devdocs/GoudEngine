//! Virtual filesystem trait definition.
//!
//! The [`VirtualFs`] trait abstracts file I/O so the asset server can read from
//! the OS filesystem, archives, embedded resources, or custom I/O layers.

use crate::assets::AssetLoadError;

/// Abstraction over a read-only filesystem for asset loading.
///
/// Implementations provide byte-level access to assets without exposing
/// the underlying storage mechanism (disk, archive, memory, network).
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` so they can be shared with
/// background loading threads.
///
/// # Example
///
/// ```
/// use goud_engine::assets::vfs::VirtualFs;
/// use goud_engine::assets::AssetLoadError;
///
/// struct MemoryFs {
///     files: std::collections::HashMap<String, Vec<u8>>,
/// }
///
/// impl VirtualFs for MemoryFs {
///     fn read(&self, path: &str) -> Result<Vec<u8>, AssetLoadError> {
///         self.files.get(path).cloned().ok_or_else(|| AssetLoadError::not_found(path))
///     }
///     fn exists(&self, path: &str) -> bool {
///         self.files.contains_key(path)
///     }
///     fn list(&self, directory: &str) -> Result<Vec<String>, AssetLoadError> {
///         let prefix = if directory.ends_with('/') {
///             directory.to_string()
///         } else {
///             format!("{}/", directory)
///         };
///         Ok(self.files.keys().filter(|k| k.starts_with(&prefix)).cloned().collect())
///     }
/// }
/// ```
pub trait VirtualFs: Send + Sync {
    /// Reads the entire contents of a file at `path`.
    ///
    /// Paths are relative to the filesystem root (no leading slash).
    ///
    /// # Errors
    ///
    /// Returns [`AssetLoadError::NotFound`] if the path does not exist,
    /// or [`AssetLoadError::IoError`] on read failures.
    fn read(&self, path: &str) -> Result<Vec<u8>, AssetLoadError>;

    /// Returns `true` if a file exists at the given path.
    fn exists(&self, path: &str) -> bool;

    /// Lists files in the given directory.
    ///
    /// Returns relative paths from the filesystem root.
    ///
    /// # Errors
    ///
    /// Returns [`AssetLoadError::IoError`] if the directory cannot be read.
    fn list(&self, directory: &str) -> Result<Vec<String>, AssetLoadError>;
}
