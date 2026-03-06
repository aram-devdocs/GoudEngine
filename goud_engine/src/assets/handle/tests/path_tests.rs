//! Tests for AssetPath.

use std::path::Path;

use super::super::path::AssetPath;

#[test]
fn test_new() {
    let path = AssetPath::new("textures/player.png");
    assert_eq!(path.as_str(), "textures/player.png");
}

#[test]
fn test_from_string() {
    let path = AssetPath::from_string("textures/player.png".to_string());
    assert_eq!(path.as_str(), "textures/player.png");
}

#[test]
fn test_is_empty() {
    let empty = AssetPath::new("");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);

    let non_empty = AssetPath::new("file.txt");
    assert!(!non_empty.is_empty());
}

#[test]
fn test_file_name() {
    assert_eq!(
        AssetPath::new("textures/player.png").file_name(),
        Some("player.png")
    );
    assert_eq!(AssetPath::new("player.png").file_name(), Some("player.png"));
    assert_eq!(
        AssetPath::new("a/b/c/file.txt").file_name(),
        Some("file.txt")
    );
    assert_eq!(AssetPath::new("textures/").file_name(), None);
    assert_eq!(AssetPath::new("").file_name(), None);
}

#[test]
fn test_extension() {
    assert_eq!(AssetPath::new("player.png").extension(), Some("png"));
    assert_eq!(
        AssetPath::new("textures/player.png").extension(),
        Some("png")
    );
    assert_eq!(AssetPath::new("archive.tar.gz").extension(), Some("gz"));
    assert_eq!(AssetPath::new("Makefile").extension(), None);
    assert_eq!(AssetPath::new(".gitignore").extension(), None);
    assert_eq!(AssetPath::new("").extension(), None);
}

#[test]
fn test_directory() {
    assert_eq!(
        AssetPath::new("textures/player.png").directory(),
        Some("textures")
    );
    assert_eq!(AssetPath::new("a/b/c/file.txt").directory(), Some("a/b/c"));
    assert_eq!(AssetPath::new("file.txt").directory(), None);
    assert_eq!(AssetPath::new("").directory(), None);
}

#[test]
fn test_stem() {
    assert_eq!(AssetPath::new("player.png").stem(), Some("player"));
    assert_eq!(AssetPath::new("textures/player.png").stem(), Some("player"));
    assert_eq!(AssetPath::new("archive.tar.gz").stem(), Some("archive.tar"));
    assert_eq!(AssetPath::new(".gitignore").stem(), Some(".gitignore"));
    assert_eq!(AssetPath::new("Makefile").stem(), Some("Makefile"));
}

#[test]
fn test_into_owned() {
    let borrowed = AssetPath::new("textures/player.png");
    let owned = borrowed.into_owned();
    assert_eq!(owned.as_str(), "textures/player.png");
}

#[test]
fn test_from_path() {
    let path = AssetPath::from_path(Path::new("textures/player.png"));
    assert_eq!(path.as_str(), "textures/player.png");
}

#[test]
fn test_join() {
    let base = AssetPath::new("textures");
    let full = base.join("player.png");
    assert_eq!(full.as_str(), "textures/player.png");

    // With trailing slash
    let base = AssetPath::new("textures/");
    let full = base.join("player.png");
    assert_eq!(full.as_str(), "textures/player.png");

    // With leading slash in other
    let base = AssetPath::new("textures");
    let full = base.join("/player.png");
    assert_eq!(full.as_str(), "textures/player.png");

    // Empty base
    let base = AssetPath::new("");
    let full = base.join("player.png");
    assert_eq!(full.as_str(), "player.png");

    // Empty other
    let base = AssetPath::new("textures");
    let full = base.join("");
    assert_eq!(full.as_str(), "textures");
}

#[test]
fn test_with_extension() {
    let path = AssetPath::new("textures/player.png");
    let new_path = path.with_extension("jpg");
    assert_eq!(new_path.as_str(), "textures/player.jpg");

    // No extension originally
    let path = AssetPath::new("Makefile");
    let new_path = path.with_extension("bak");
    assert_eq!(new_path.as_str(), "Makefile.bak");

    // No directory
    let path = AssetPath::new("player.png");
    let new_path = path.with_extension("jpg");
    assert_eq!(new_path.as_str(), "player.jpg");
}

#[test]
fn test_equality() {
    let p1 = AssetPath::new("textures/player.png");
    let p2 = AssetPath::new("textures/player.png");
    let p3 = AssetPath::new("textures/enemy.png");

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
}

#[test]
fn test_equality_with_str() {
    let path = AssetPath::new("textures/player.png");
    assert!(path == "textures/player.png");
    let str_ref: &str = "textures/player.png";
    assert!(path == str_ref);
}

#[test]
fn test_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(AssetPath::new("a.txt").into_owned());
    set.insert(AssetPath::new("b.txt").into_owned());

    assert_eq!(set.len(), 2);

    set.insert(AssetPath::new("a.txt").into_owned());
    assert_eq!(set.len(), 2);
}

#[test]
fn test_debug() {
    let path = AssetPath::new("textures/player.png");
    let debug_str = format!("{:?}", path);
    assert!(debug_str.contains("AssetPath"));
    assert!(debug_str.contains("textures/player.png"));
}

#[test]
fn test_display() {
    let path = AssetPath::new("textures/player.png");
    assert_eq!(format!("{}", path), "textures/player.png");
}

#[test]
fn test_as_ref() {
    let path = AssetPath::new("textures/player.png");
    let s: &str = path.as_ref();
    assert_eq!(s, "textures/player.png");
}

#[test]
fn test_from_str() {
    let path: AssetPath = "textures/player.png".into();
    assert_eq!(path.as_str(), "textures/player.png");
}

#[test]
fn test_from_string_into() {
    let path: AssetPath<'static> = "textures/player.png".to_string().into();
    assert_eq!(path.as_str(), "textures/player.png");
}
