//! Thread safety and `AnyAssetStorage` trait tests.

use crate::assets::{
    AnyAssetStorage, AssetEntry, AssetId, AssetPath, AssetType, TypedAssetStorage,
    UntypedAssetHandle,
};

use super::fixtures::{TestAudio, TestTexture};

// =========================================================================
// Thread Safety Tests
// =========================================================================

#[test]
fn test_typed_storage_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<TypedAssetStorage<TestTexture>>();
}

#[test]
fn test_typed_storage_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<TypedAssetStorage<TestTexture>>();
}

#[test]
fn test_asset_storage_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<crate::assets::AssetStorage>();
}

#[test]
fn test_asset_storage_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<crate::assets::AssetStorage>();
}

#[test]
fn test_asset_entry_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<AssetEntry<TestTexture>>();
}

#[test]
fn test_asset_entry_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<AssetEntry<TestTexture>>();
}

// =========================================================================
// AnyAssetStorage Tests
// =========================================================================

#[test]
fn test_asset_id() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let any_storage: &dyn AnyAssetStorage = &storage;

    assert_eq!(any_storage.asset_id(), AssetId::of::<TestTexture>());
}

#[test]
fn test_asset_info() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let any_storage: &dyn AnyAssetStorage = &storage;

    let info = any_storage.asset_info();
    assert_eq!(info.name, "TestTexture");
    assert_eq!(info.asset_type, AssetType::Texture);
}

#[test]
fn test_len_and_is_empty() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let any_storage: &dyn AnyAssetStorage = &storage;

    assert!(any_storage.is_empty());
    assert_eq!(any_storage.len(), 0);

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let any_storage: &dyn AnyAssetStorage = &storage;

    assert!(!any_storage.is_empty());
    assert_eq!(any_storage.len(), 1);
}

#[test]
fn test_is_alive_raw() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    let any_storage: &dyn AnyAssetStorage = &storage;
    assert!(any_storage.is_alive_raw(handle.index(), handle.generation()));
    assert!(!any_storage.is_alive_raw(handle.index(), handle.generation() + 1));
}

#[test]
fn test_remove_untyped() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let untyped = handle.untyped();

    let any_storage: &mut dyn AnyAssetStorage = &mut storage;
    assert!(any_storage.remove_untyped(&untyped));
    assert!(!any_storage.is_alive_raw(handle.index(), handle.generation()));
}

#[test]
fn test_remove_untyped_wrong_type() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    // Create untyped handle with wrong type
    let wrong_untyped = UntypedAssetHandle::new(0, 1, AssetId::of::<TestAudio>());

    let any_storage: &mut dyn AnyAssetStorage = &mut storage;
    assert!(!any_storage.remove_untyped(&wrong_untyped));
}

#[test]
fn test_get_state_untyped() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let untyped = handle.untyped();

    let any_storage: &dyn AnyAssetStorage = &storage;
    let state = any_storage.get_state_untyped(&untyped);
    assert!(state.is_some());
    assert!(state.unwrap().is_ready());
}

#[test]
fn test_get_path_untyped() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let handle = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );
    let untyped = handle.untyped();

    let any_storage: &dyn AnyAssetStorage = &storage;
    let path = any_storage.get_path_untyped(&untyped);
    assert!(path.is_some());
    assert_eq!(path.unwrap().as_str(), "textures/player.png");
}

#[test]
fn test_as_any_downcast() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    let any_storage: &dyn AnyAssetStorage = &storage;
    let downcasted = any_storage
        .as_any()
        .downcast_ref::<TypedAssetStorage<TestTexture>>();

    assert!(downcasted.is_some());
    assert_eq!(downcasted.unwrap().len(), 1);
}

#[test]
fn test_clear() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    let any_storage: &mut dyn AnyAssetStorage = &mut storage;
    any_storage.clear();

    assert!(any_storage.is_empty());
}
