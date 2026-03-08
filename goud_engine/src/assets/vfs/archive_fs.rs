//! Archive-backed virtual filesystem.
//!
//! [`ArchiveFs`] loads assets from a GOUD archive created by
//! [`ArchiveWriter`](super::archive_format::ArchiveWriter). It owns the raw
//! bytes and delegates lookups to [`ArchiveReader`](super::archive_format::ArchiveReader).

use super::archive_format::ArchiveReader;
use super::VirtualFs;
use crate::assets::AssetLoadError;

/// A virtual filesystem backed by an in-memory GOUD archive.
///
/// Construct via [`from_archive`](Self::from_archive), passing the raw archive
/// bytes (typically loaded from disk or embedded into the binary).
#[derive(Debug)]
pub struct ArchiveFs {
    /// Raw archive bytes (header + TOC + data).
    data: Vec<u8>,
    /// Parsed reader for TOC lookups.
    reader: ArchiveReader,
}

impl ArchiveFs {
    /// Creates an `ArchiveFs` from raw archive bytes.
    ///
    /// Parses the header and table of contents. Returns an error if the bytes
    /// are not a valid GOUD archive.
    pub fn from_archive(data: Vec<u8>) -> Result<Self, AssetLoadError> {
        let reader = ArchiveReader::from_bytes(&data)?;
        Ok(Self { data, reader })
    }
}

impl VirtualFs for ArchiveFs {
    fn read(&self, path: &str) -> Result<Vec<u8>, AssetLoadError> {
        let slice = self.reader.read_entry(path, &self.data)?;
        Ok(slice.to_vec())
    }

    fn exists(&self, path: &str) -> bool {
        self.reader.entry_exists(path)
    }

    fn list(&self, directory: &str) -> Result<Vec<String>, AssetLoadError> {
        let prefix = if directory.is_empty() {
            String::new()
        } else if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{directory}/")
        };

        let mut result: Vec<String> = self
            .reader
            .entries()
            .iter()
            .filter(|e| {
                if prefix.is_empty() {
                    true
                } else {
                    e.path.starts_with(&prefix)
                }
            })
            .map(|e| e.path.clone())
            .collect();

        result.sort();
        Ok(result)
    }
}
