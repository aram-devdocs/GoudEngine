//! Tests for the virtual filesystem implementations.

use super::{ArchiveFs, OsFs, VirtualFs};
use super::archive_format::ArchiveWriter;

// ---------------------------------------------------------------------------
// OsFs tests
// ---------------------------------------------------------------------------

#[test]
fn os_fs_read_existing_file() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let file_path = dir.path().join("hello.txt");
    std::fs::write(&file_path, b"hello world").unwrap();

    let fs = OsFs::new(dir.path());
    let bytes = fs.read("hello.txt").expect("read should succeed");
    assert_eq!(bytes, b"hello world");
}

#[test]
fn os_fs_read_missing_file_returns_not_found() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let fs = OsFs::new(dir.path());
    let err = fs.read("nonexistent.txt").unwrap_err();
    assert!(err.is_not_found() || err.is_io_error());
}

#[test]
fn os_fs_read_nested_path() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::create_dir_all(dir.path().join("sub/dir")).unwrap();
    std::fs::write(dir.path().join("sub/dir/data.bin"), b"\x01\x02").unwrap();

    let fs = OsFs::new(dir.path());
    let bytes = fs.read("sub/dir/data.bin").unwrap();
    assert_eq!(bytes, &[0x01, 0x02]);
}

#[test]
fn os_fs_exists_returns_true_for_existing_file() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(dir.path().join("present.txt"), b"data").unwrap();

    let fs = OsFs::new(dir.path());
    assert!(fs.exists("present.txt"));
}

#[test]
fn os_fs_exists_returns_false_for_missing_file() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let fs = OsFs::new(dir.path());
    assert!(!fs.exists("ghost.txt"));
}

#[test]
fn os_fs_list_directory() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::create_dir_all(dir.path().join("textures")).unwrap();
    std::fs::write(dir.path().join("textures/a.png"), b"").unwrap();
    std::fs::write(dir.path().join("textures/b.png"), b"").unwrap();

    let fs = OsFs::new(dir.path());
    let mut files = fs.list("textures").expect("list should succeed");
    files.sort();
    assert_eq!(files, vec!["textures/a.png", "textures/b.png"]);
}

#[test]
fn os_fs_list_missing_directory_returns_error() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let fs = OsFs::new(dir.path());
    let err = fs.list("no_such_dir").unwrap_err();
    assert!(err.is_io_error());
}

#[test]
fn os_fs_root_accessor() {
    let fs = OsFs::new("/some/path");
    assert_eq!(fs.root().to_str().unwrap(), "/some/path");
}

// ---------------------------------------------------------------------------
// ArchiveFs tests
// ---------------------------------------------------------------------------

#[test]
fn archive_fs_read_returns_not_found() {
    let writer = ArchiveWriter::new();
    let mut buf = Vec::new();
    writer.write_to(&mut buf).unwrap();

    let fs = ArchiveFs::from_archive(buf).unwrap();
    let err = fs.read("any/file.txt").unwrap_err();
    assert!(err.is_not_found());
}

#[test]
fn archive_fs_exists_returns_false() {
    let writer = ArchiveWriter::new();
    let mut buf = Vec::new();
    writer.write_to(&mut buf).unwrap();

    let fs = ArchiveFs::from_archive(buf).unwrap();
    assert!(!fs.exists("any/file.txt"));
}

#[test]
fn archive_fs_list_returns_empty() {
    let writer = ArchiveWriter::new();
    let mut buf = Vec::new();
    writer.write_to(&mut buf).unwrap();

    let fs = ArchiveFs::from_archive(buf).unwrap();
    let result = fs.list("any").unwrap();
    assert!(result.is_empty());
}

#[test]
fn archive_fs_read_file_succeeds() {
    let mut writer = ArchiveWriter::new();
    writer.add_file("test/file.txt", b"archive data");
    let mut buf = Vec::new();
    writer.write_to(&mut buf).unwrap();

    let fs = ArchiveFs::from_archive(buf).unwrap();
    let data = fs.read("test/file.txt").unwrap();
    assert_eq!(data, b"archive data");
}

// ---------------------------------------------------------------------------
// Trait object tests — ensure VirtualFs works as dyn
// ---------------------------------------------------------------------------

#[test]
fn virtual_fs_is_object_safe() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(dir.path().join("test.txt"), b"dyn works").unwrap();

    let fs: Box<dyn VirtualFs> = Box::new(OsFs::new(dir.path()));
    let bytes = fs.read("test.txt").unwrap();
    assert_eq!(bytes, b"dyn works");
}

#[test]
fn virtual_fs_can_swap_implementations() {
    // Start with OsFs, swap to ArchiveFs — demonstrates the abstraction works
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(dir.path().join("file.txt"), b"os data").unwrap();

    let os: Box<dyn VirtualFs> = Box::new(OsFs::new(dir.path()));
    assert!(os.exists("file.txt"));

    let mut writer = ArchiveWriter::new();
    writer.add_file("archive_file.txt", b"archive data");
    let mut buf = Vec::new();
    writer.write_to(&mut buf).unwrap();

    let archive: Box<dyn VirtualFs> = Box::new(ArchiveFs::from_archive(buf).unwrap());
    assert!(archive.exists("archive_file.txt"));
    assert!(!archive.exists("file.txt"));
}
