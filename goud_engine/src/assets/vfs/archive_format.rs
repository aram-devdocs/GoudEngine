//! Binary archive format for bundling assets into a single file.
//!
//! # Layout
//!
//! ```text
//! [MAGIC: 4 bytes "GOUD"]
//! [TOC_SIZE: u32 LE]
//! [TOC bytes (entry_count + entries)]
//! [file data concatenated]
//! ```
//!
//! TOC entry format:
//! ```text
//! [path_len: u32 LE][path bytes][offset: u64 LE][size: u64 LE][crc32: u32 LE]
//! ```

use crate::assets::AssetLoadError;
use std::collections::HashMap;
use std::io::Write;

/// Magic bytes identifying a GOUD archive.
pub const ARCHIVE_MAGIC: [u8; 4] = *b"GOUD";

/// Current archive format version.
pub const ARCHIVE_VERSION: u32 = 1;

/// A single entry in the archive table of contents.
#[derive(Debug, Clone)]
pub struct ArchiveEntry {
    /// Relative path of the asset (forward slashes).
    pub path: String,
    /// Byte offset from the start of the data section.
    pub offset: u64,
    /// Size of the asset data in bytes.
    pub size: u64,
    /// CRC32 checksum (reserved, currently 0).
    pub crc32: u32,
}

/// Table of contents for an archive.
#[derive(Debug, Clone)]
pub struct ArchiveToc {
    /// Archive format version.
    pub version: u32,
    /// Entries in the archive.
    pub entries: Vec<ArchiveEntry>,
}

// ============================================================================
// ArchiveWriter
// ============================================================================

/// Builds an archive from in-memory file data.
///
/// Files are added with [`add_file`](Self::add_file) and the archive is
/// written to any [`Write`] target via [`write_to`](Self::write_to).
///
/// Entries are sorted by path for reproducible output.
#[derive(Debug)]
pub struct ArchiveWriter {
    files: Vec<(String, Vec<u8>)>,
}

impl ArchiveWriter {
    /// Creates an empty archive writer.
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Adds a file to the archive.
    ///
    /// The `path` should use forward slashes and be relative to the asset root.
    pub fn add_file(&mut self, path: &str, data: &[u8]) {
        self.files.push((path.to_string(), data.to_vec()));
    }

    /// Writes the archive to the given writer.
    ///
    /// Entries are sorted by path for reproducibility.
    pub fn write_to(&self, writer: &mut impl Write) -> Result<(), AssetLoadError> {
        let mut sorted: Vec<_> = self.files.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        // Compute offsets: each file's offset is relative to the data section start.
        let mut entries = Vec::with_capacity(sorted.len());
        let mut current_offset: u64 = 0;
        for (path, data) in &sorted {
            entries.push(ArchiveEntry {
                path: path.clone(),
                offset: current_offset,
                size: data.len() as u64,
                crc32: 0,
            });
            current_offset += data.len() as u64;
        }

        // Serialize the TOC to a buffer.
        let toc_bytes = serialize_toc(&ArchiveToc {
            version: ARCHIVE_VERSION,
            entries,
        })?;

        // Write header: MAGIC + TOC_SIZE + TOC + data.
        write_all(writer, &ARCHIVE_MAGIC)?;
        write_all(writer, &(toc_bytes.len() as u32).to_le_bytes())?;
        write_all(writer, &toc_bytes)?;

        for (_, data) in &sorted {
            write_all(writer, data)?;
        }

        Ok(())
    }
}

impl Default for ArchiveWriter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ArchiveReader
// ============================================================================

/// Reads assets from an in-memory archive.
///
/// The reader borrows the archive bytes and provides zero-copy access to
/// individual entries via [`read_entry`](Self::read_entry).
#[derive(Debug)]
pub struct ArchiveReader {
    toc: ArchiveToc,
    /// Byte offset where the data section starts within the source buffer.
    data_start: usize,
    /// Lookup table for fast path-based access.
    index: HashMap<String, usize>,
}

impl ArchiveReader {
    /// Parses an archive from raw bytes.
    ///
    /// Validates the magic bytes and version, then deserializes the TOC.
    pub fn from_bytes(data: &[u8]) -> Result<Self, AssetLoadError> {
        if data.len() < 8 {
            return Err(AssetLoadError::decode_failed(
                "Archive too small: missing header",
            ));
        }

        // Validate magic.
        if data[..4] != ARCHIVE_MAGIC {
            return Err(AssetLoadError::decode_failed(
                "Invalid archive: bad magic bytes",
            ));
        }

        // Read TOC size.
        let toc_size = u32::from_le_bytes(
            data[4..8]
                .try_into()
                .map_err(|_| AssetLoadError::decode_failed("Invalid TOC size bytes"))?,
        ) as usize;

        let toc_end = 8 + toc_size;
        if data.len() < toc_end {
            return Err(AssetLoadError::decode_failed(
                "Archive truncated: TOC extends past end of data",
            ));
        }

        let toc = deserialize_toc(&data[8..toc_end])?;

        // Build index for O(1) lookups.
        let mut index = HashMap::with_capacity(toc.entries.len());
        for (i, entry) in toc.entries.iter().enumerate() {
            index.insert(entry.path.clone(), i);
        }

        Ok(Self {
            toc,
            data_start: toc_end,
            index,
        })
    }

    /// Reads the data for the entry at `path` from the source buffer.
    pub fn read_entry<'a>(&self, path: &str, source: &'a [u8]) -> Result<&'a [u8], AssetLoadError> {
        let idx = self
            .index
            .get(path)
            .ok_or_else(|| AssetLoadError::not_found(path))?;
        let entry = &self.toc.entries[*idx];
        let start = self.data_start + entry.offset as usize;
        let end = start + entry.size as usize;
        if end > source.len() {
            return Err(AssetLoadError::decode_failed(format!(
                "Archive entry '{}' extends past end of data",
                path
            )));
        }
        Ok(&source[start..end])
    }

    /// Returns all entries in the TOC.
    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.toc.entries
    }

    /// Returns `true` if the archive contains an entry at the given path.
    pub fn entry_exists(&self, path: &str) -> bool {
        self.index.contains_key(path)
    }
}

// ============================================================================
// Binary serialization helpers (no external deps)
// ============================================================================

fn serialize_toc(toc: &ArchiveToc) -> Result<Vec<u8>, AssetLoadError> {
    let mut buf = Vec::new();
    write_all(&mut buf, &toc.version.to_le_bytes())?;
    write_all(&mut buf, &(toc.entries.len() as u32).to_le_bytes())?;
    for entry in &toc.entries {
        let path_bytes = entry.path.as_bytes();
        write_all(&mut buf, &(path_bytes.len() as u32).to_le_bytes())?;
        write_all(&mut buf, path_bytes)?;
        write_all(&mut buf, &entry.offset.to_le_bytes())?;
        write_all(&mut buf, &entry.size.to_le_bytes())?;
        write_all(&mut buf, &entry.crc32.to_le_bytes())?;
    }
    Ok(buf)
}

fn deserialize_toc(data: &[u8]) -> Result<ArchiveToc, AssetLoadError> {
    let mut cursor = 0;

    let version = read_u32(data, &mut cursor)?;
    let entry_count = read_u32(data, &mut cursor)?;

    let mut entries = Vec::with_capacity(entry_count as usize);
    for _ in 0..entry_count {
        let path_len = read_u32(data, &mut cursor)? as usize;
        if cursor + path_len > data.len() {
            return Err(AssetLoadError::decode_failed("TOC truncated: path data"));
        }
        let path = std::str::from_utf8(&data[cursor..cursor + path_len])
            .map_err(|e| AssetLoadError::decode_failed(format!("Invalid UTF-8 in path: {e}")))?
            .to_string();
        cursor += path_len;

        let offset = read_u64(data, &mut cursor)?;
        let size = read_u64(data, &mut cursor)?;
        let crc32 = read_u32(data, &mut cursor)?;

        entries.push(ArchiveEntry {
            path,
            offset,
            size,
            crc32,
        });
    }

    Ok(ArchiveToc { version, entries })
}

fn read_u32(data: &[u8], cursor: &mut usize) -> Result<u32, AssetLoadError> {
    if *cursor + 4 > data.len() {
        return Err(AssetLoadError::decode_failed(
            "Unexpected end of data reading u32",
        ));
    }
    let val = u32::from_le_bytes(
        data[*cursor..*cursor + 4]
            .try_into()
            .map_err(|_| AssetLoadError::decode_failed("Invalid u32 slice"))?,
    );
    *cursor += 4;
    Ok(val)
}

fn read_u64(data: &[u8], cursor: &mut usize) -> Result<u64, AssetLoadError> {
    if *cursor + 8 > data.len() {
        return Err(AssetLoadError::decode_failed(
            "Unexpected end of data reading u64",
        ));
    }
    let val = u64::from_le_bytes(
        data[*cursor..*cursor + 8]
            .try_into()
            .map_err(|_| AssetLoadError::decode_failed("Invalid u64 slice"))?,
    );
    *cursor += 8;
    Ok(val)
}

fn write_all(writer: &mut impl Write, data: &[u8]) -> Result<(), AssetLoadError> {
    writer
        .write_all(data)
        .map_err(|e| AssetLoadError::custom(format!("Write failed: {e}")))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_single_file() {
        let mut writer = ArchiveWriter::new();
        writer.add_file("hello.txt", b"Hello, world!");

        let mut buf = Vec::new();
        writer.write_to(&mut buf).unwrap();

        let reader = ArchiveReader::from_bytes(&buf).unwrap();
        assert!(reader.entry_exists("hello.txt"));
        assert!(!reader.entry_exists("missing.txt"));

        let data = reader.read_entry("hello.txt", &buf).unwrap();
        assert_eq!(data, b"Hello, world!");
    }

    #[test]
    fn round_trip_multiple_files() {
        let mut writer = ArchiveWriter::new();
        writer.add_file("b.txt", b"BBB");
        writer.add_file("a.txt", b"AAA");
        writer.add_file("c/nested.bin", &[1, 2, 3, 4]);

        let mut buf = Vec::new();
        writer.write_to(&mut buf).unwrap();

        let reader = ArchiveReader::from_bytes(&buf).unwrap();
        assert_eq!(reader.entries().len(), 3);

        // Verify sorted order.
        assert_eq!(reader.entries()[0].path, "a.txt");
        assert_eq!(reader.entries()[1].path, "b.txt");
        assert_eq!(reader.entries()[2].path, "c/nested.bin");

        assert_eq!(reader.read_entry("a.txt", &buf).unwrap(), b"AAA");
        assert_eq!(reader.read_entry("b.txt", &buf).unwrap(), b"BBB");
        assert_eq!(
            reader.read_entry("c/nested.bin", &buf).unwrap(),
            &[1, 2, 3, 4]
        );
    }

    #[test]
    fn round_trip_empty_archive() {
        let writer = ArchiveWriter::new();
        let mut buf = Vec::new();
        writer.write_to(&mut buf).unwrap();

        let reader = ArchiveReader::from_bytes(&buf).unwrap();
        assert_eq!(reader.entries().len(), 0);
        assert!(!reader.entry_exists("anything"));
    }

    #[test]
    fn invalid_magic_returns_error() {
        let data = b"BAAD\x00\x00\x00\x00";
        let err = ArchiveReader::from_bytes(data).unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn truncated_header_returns_error() {
        let err = ArchiveReader::from_bytes(b"GOU").unwrap_err();
        assert!(err.is_decode_failed());
    }

    #[test]
    fn missing_entry_returns_not_found() {
        let writer = ArchiveWriter::new();
        let mut buf = Vec::new();
        writer.write_to(&mut buf).unwrap();

        let reader = ArchiveReader::from_bytes(&buf).unwrap();
        let err = reader.read_entry("nonexistent", &buf).unwrap_err();
        assert!(err.is_not_found());
    }

    #[test]
    fn reproducible_output() {
        let mut writer1 = ArchiveWriter::new();
        writer1.add_file("z.txt", b"Z");
        writer1.add_file("a.txt", b"A");

        let mut writer2 = ArchiveWriter::new();
        writer2.add_file("a.txt", b"A");
        writer2.add_file("z.txt", b"Z");

        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();
        writer1.write_to(&mut buf1).unwrap();
        writer2.write_to(&mut buf2).unwrap();

        assert_eq!(
            buf1, buf2,
            "Archives with same content must be byte-identical"
        );
    }

    #[test]
    fn empty_file_entry() {
        let mut writer = ArchiveWriter::new();
        writer.add_file("empty.dat", b"");

        let mut buf = Vec::new();
        writer.write_to(&mut buf).unwrap();

        let reader = ArchiveReader::from_bytes(&buf).unwrap();
        let data = reader.read_entry("empty.dat", &buf).unwrap();
        assert!(data.is_empty());
    }
}
