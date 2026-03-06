//! Tests for UntypedAssetHandle.

use crate::assets::AssetId;

use super::super::typed::AssetHandle;
use super::super::untyped::UntypedAssetHandle;
use super::common::{TestAudio, TestTexture};

#[test]
fn test_new() {
    let handle = UntypedAssetHandle::new(42, 7, AssetId::of::<TestTexture>());
    assert_eq!(handle.index(), 42);
    assert_eq!(handle.generation(), 7);
    assert_eq!(handle.asset_id(), AssetId::of::<TestTexture>());
}

#[test]
fn test_invalid() {
    let handle = UntypedAssetHandle::invalid();
    assert!(!handle.is_valid());
}

#[test]
fn test_from_typed() {
    let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
    let untyped = UntypedAssetHandle::from_typed(typed);

    assert_eq!(untyped.index(), 10);
    assert_eq!(untyped.generation(), 5);
    assert_eq!(untyped.asset_id(), AssetId::of::<TestTexture>());
}

#[test]
fn test_typed() {
    let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
    let untyped = typed.untyped();

    // Correct type succeeds
    let recovered: Option<AssetHandle<TestTexture>> = untyped.typed();
    assert!(recovered.is_some());
    assert_eq!(recovered.unwrap(), typed);

    // Wrong type fails
    let wrong: Option<AssetHandle<TestAudio>> = untyped.typed();
    assert!(wrong.is_none());
}

#[test]
fn test_typed_unchecked() {
    let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
    let untyped = typed.untyped();

    // SAFETY: We know this was created from TestTexture
    let recovered: AssetHandle<TestTexture> = unsafe { untyped.typed_unchecked() };
    assert_eq!(typed, recovered);
}

#[test]
fn test_is_type() {
    let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
    let untyped = typed.untyped();

    assert!(untyped.is_type::<TestTexture>());
    assert!(!untyped.is_type::<TestAudio>());
}

#[test]
fn test_equality() {
    let h1 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>());
    let h2 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>());
    let h3 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestAudio>()); // Different type
    let h4 = UntypedAssetHandle::new(1, 2, AssetId::of::<TestTexture>()); // Different gen

    assert_eq!(h1, h2);
    assert_ne!(h1, h3); // Different asset type
    assert_ne!(h1, h4); // Different generation
}

#[test]
fn test_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>()));
    set.insert(UntypedAssetHandle::new(1, 1, AssetId::of::<TestAudio>()));

    // Different types are different entries
    assert_eq!(set.len(), 2);
}

#[test]
fn test_debug() {
    let handle = UntypedAssetHandle::new(42, 7, AssetId::of::<TestTexture>());
    let debug_str = format!("{:?}", handle);
    assert!(debug_str.contains("UntypedAssetHandle"));
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("7"));

    let invalid = UntypedAssetHandle::invalid();
    let debug_str = format!("{:?}", invalid);
    assert!(debug_str.contains("INVALID"));
}

#[test]
fn test_display() {
    let handle = UntypedAssetHandle::new(42, 7, AssetId::of::<TestTexture>());
    assert_eq!(format!("{}", handle), "42:7");

    let invalid = UntypedAssetHandle::invalid();
    assert_eq!(format!("{}", invalid), "INVALID");
}

#[test]
fn test_clone_copy() {
    let h1 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>());
    let h2 = h1; // Copy
    let h3 = h1.clone();

    assert_eq!(h1, h2);
    assert_eq!(h1, h3);
}

#[test]
fn test_default() {
    let handle: UntypedAssetHandle = Default::default();
    assert!(!handle.is_valid());
}

#[test]
fn test_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<UntypedAssetHandle>();
}

#[test]
fn test_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<UntypedAssetHandle>();
}
