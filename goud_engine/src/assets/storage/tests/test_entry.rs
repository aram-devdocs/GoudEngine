//! Tests for [`AssetEntry`].

use crate::assets::{AssetEntry, AssetPath, AssetState};

use super::fixtures::TestTexture;

#[test]
fn test_empty() {
    let entry: AssetEntry<TestTexture> = AssetEntry::empty();
    assert!(entry.asset().is_none());
    assert!(!entry.is_loaded());
    assert!(!entry.is_loading());
    assert!(!entry.is_failed());
    assert!(entry.path().is_none());
}

#[test]
fn test_loading() {
    let entry: AssetEntry<TestTexture> = AssetEntry::loading(0.5);
    assert!(entry.is_loading());
    assert!(!entry.is_loaded());
    assert_eq!(entry.state().progress(), Some(0.5));
}

#[test]
fn test_loaded() {
    let entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    assert!(entry.is_loaded());
    assert!(!entry.is_loading());
    assert_eq!(entry.asset().unwrap().width, 256);
}

#[test]
fn test_with_path() {
    let entry = AssetEntry::with_path(
        TestTexture {
            width: 512,
            height: 512,
        },
        AssetPath::new("textures/player.png"),
    );
    assert!(entry.is_loaded());
    assert_eq!(
        entry.path().map(|p| p.as_str()),
        Some("textures/player.png")
    );
}

#[test]
fn test_failed() {
    let entry: AssetEntry<TestTexture> = AssetEntry::failed("File not found");
    assert!(entry.is_failed());
    assert!(!entry.is_loaded());
    assert_eq!(entry.state().error(), Some("File not found"));
}

#[test]
fn test_asset_mut() {
    let mut entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    if let Some(asset) = entry.asset_mut() {
        asset.width = 512;
    }
    assert_eq!(entry.asset().unwrap().width, 512);
}

#[test]
fn test_take_asset() {
    let mut entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    let asset = entry.take_asset();
    assert!(asset.is_some());
    assert!(entry.asset().is_none());
    assert_eq!(
        entry.state().discriminant(),
        AssetState::Unloaded.discriminant()
    );
}

#[test]
fn test_set_path() {
    let mut entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    entry.set_path(AssetPath::from_string("new/path.png".to_string()));
    assert_eq!(entry.path().map(|p| p.as_str()), Some("new/path.png"));
}

#[test]
fn test_clear_path() {
    let mut entry = AssetEntry::with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );
    entry.clear_path();
    assert!(entry.path().is_none());
}

#[test]
fn test_set_loaded() {
    let mut entry: AssetEntry<TestTexture> = AssetEntry::loading(0.5);
    entry.set_loaded(TestTexture {
        width: 256,
        height: 256,
    });
    assert!(entry.is_loaded());
    assert_eq!(entry.asset().unwrap().width, 256);
}

#[test]
fn test_set_progress() {
    let mut entry: AssetEntry<TestTexture> = AssetEntry::loading(0.0);
    entry.set_progress(0.75);
    assert_eq!(entry.state().progress(), Some(0.75));
}

#[test]
fn test_set_failed() {
    let mut entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    entry.set_failed("Error occurred");
    assert!(entry.is_failed());
    assert!(entry.asset().is_none());
}

#[test]
fn test_set_unloaded() {
    let mut entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    entry.set_unloaded();
    assert!(entry.asset().is_none());
    assert_eq!(
        entry.state().discriminant(),
        AssetState::Unloaded.discriminant()
    );
}

#[test]
fn test_default() {
    let entry: AssetEntry<TestTexture> = AssetEntry::default();
    assert!(entry.asset().is_none());
    assert!(!entry.is_loaded());
}

#[test]
fn test_clone() {
    let entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    let cloned = entry.clone();
    assert_eq!(entry.asset(), cloned.asset());
}

#[test]
fn test_debug() {
    let entry = AssetEntry::loaded(TestTexture {
        width: 256,
        height: 256,
    });
    let debug_str = format!("{:?}", entry);
    assert!(debug_str.contains("AssetEntry"));
}
