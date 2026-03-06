//! Tests for [`TypedAssetStorage`].

use crate::assets::{AssetHandle, AssetPath, TypedAssetStorage};

use super::fixtures::TestTexture;

#[test]
fn test_new() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    assert!(storage.is_empty());
    assert_eq!(storage.len(), 0);
}

#[test]
fn test_with_capacity() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::with_capacity(100);
    assert!(storage.is_empty());
}

#[test]
fn test_insert() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    assert!(handle.is_valid());
    assert!(storage.is_alive(&handle));
    assert_eq!(storage.len(), 1);
}

#[test]
fn test_insert_multiple() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let h1 = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let h2 = storage.insert(TestTexture {
        width: 512,
        height: 512,
    });
    let h3 = storage.insert(TestTexture {
        width: 1024,
        height: 1024,
    });

    assert_ne!(h1, h2);
    assert_ne!(h2, h3);
    assert_eq!(storage.len(), 3);
}

#[test]
fn test_insert_with_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let h1 = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    assert!(storage.is_alive(&h1));
    assert!(storage.has_path("textures/player.png"));
}

#[test]
fn test_insert_with_path_deduplication() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let h1 = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    let h2 = storage.insert_with_path(
        TestTexture {
            width: 512,
            height: 512,
        }, // Different asset
        AssetPath::new("textures/player.png"), // Same path
    );

    // Should return existing handle
    assert_eq!(h1, h2);
    assert_eq!(storage.len(), 1);
    // Original asset preserved
    assert_eq!(storage.get(&h1).unwrap().width, 256);
}

#[test]
fn test_reserve() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.reserve();
    assert!(storage.is_alive(&handle));
    assert!(storage.get(&handle).is_none()); // Not loaded yet
    assert_eq!(storage.len(), 1);
}

#[test]
fn test_reserve_with_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.reserve_with_path(AssetPath::new("textures/player.png"));
    assert!(storage.is_alive(&handle));
    assert!(storage.has_path("textures/player.png"));
    assert!(storage.get(&handle).is_none());
}

#[test]
fn test_set_loaded() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.reserve();
    assert!(storage.get(&handle).is_none());

    let result = storage.set_loaded(
        &handle,
        TestTexture {
            width: 256,
            height: 256,
        },
    );
    assert!(result);
    assert_eq!(storage.get(&handle).unwrap().width, 256);
}

#[test]
fn test_set_loaded_invalid_handle() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let result = storage.set_loaded(
        &AssetHandle::INVALID,
        TestTexture {
            width: 256,
            height: 256,
        },
    );
    assert!(!result);
}

#[test]
fn test_remove() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let removed = storage.remove(&handle);

    assert!(removed.is_some());
    assert_eq!(removed.unwrap().width, 256);
    assert!(!storage.is_alive(&handle));
    assert_eq!(storage.len(), 0);
}

#[test]
fn test_remove_with_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    storage.remove(&handle);
    assert!(!storage.has_path("textures/player.png"));
}

#[test]
fn test_remove_invalid() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let removed = storage.remove(&AssetHandle::INVALID);
    assert!(removed.is_none());
}

#[test]
fn test_get() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let asset = storage.get(&handle);

    assert!(asset.is_some());
    assert_eq!(asset.unwrap().width, 256);
}

#[test]
fn test_get_invalid() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    assert!(storage.get(&AssetHandle::INVALID).is_none());
}

#[test]
fn test_get_stale() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.remove(&handle);

    assert!(storage.get(&handle).is_none());
}

#[test]
fn test_get_mut() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    if let Some(asset) = storage.get_mut(&handle) {
        asset.width = 512;
    }

    assert_eq!(storage.get(&handle).unwrap().width, 512);
}

#[test]
fn test_get_entry() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let entry = storage.get_entry(&handle);

    assert!(entry.is_some());
    assert!(entry.unwrap().is_loaded());
}

#[test]
fn test_get_state() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let state = storage.get_state(&handle);

    assert!(state.is_some());
    assert!(state.unwrap().is_ready());
}

#[test]
fn test_get_handle_by_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    let found = storage.get_handle_by_path("textures/player.png");
    assert_eq!(found, Some(handle));

    assert!(storage.get_handle_by_path("nonexistent.png").is_none());
}

#[test]
fn test_get_by_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    let asset = storage.get_by_path("textures/player.png");
    assert!(asset.is_some());
    assert_eq!(asset.unwrap().width, 256);
}

#[test]
fn test_set_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let result = storage.set_path(&handle, AssetPath::new("textures/player.png"));

    assert!(result);
    assert!(storage.has_path("textures/player.png"));
    assert_eq!(
        storage.get_handle_by_path("textures/player.png"),
        Some(handle)
    );
}

#[test]
fn test_set_path_replaces_old() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("old/path.png"),
    );

    storage.set_path(&handle, AssetPath::new("new/path.png"));

    assert!(!storage.has_path("old/path.png"));
    assert!(storage.has_path("new/path.png"));
}

#[test]
fn test_clear_path() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let handle = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    storage.clear_path(&handle);
    assert!(!storage.has_path("textures/player.png"));
}

#[test]
fn test_clear() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let h1 = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let h2 = storage.insert(TestTexture {
        width: 512,
        height: 512,
    });

    storage.clear();

    assert!(!storage.is_alive(&h1));
    assert!(!storage.is_alive(&h2));
    assert_eq!(storage.len(), 0);
}

#[test]
fn test_iter() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.insert(TestTexture {
        width: 512,
        height: 512,
    });

    let pairs: Vec<_> = storage.iter().collect();
    assert_eq!(pairs.len(), 2);
}

#[test]
fn test_handles() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let h1 = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let h2 = storage.insert(TestTexture {
        width: 512,
        height: 512,
    });

    let handles: Vec<_> = storage.handles().collect();
    assert!(handles.contains(&h1));
    assert!(handles.contains(&h2));
}

#[test]
fn test_paths() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/a.png"),
    );
    storage.insert_with_path(
        TestTexture {
            width: 512,
            height: 512,
        },
        AssetPath::new("textures/b.png"),
    );

    let paths: Vec<_> = storage.paths().collect();
    assert!(paths.contains(&"textures/a.png"));
    assert!(paths.contains(&"textures/b.png"));
    assert_eq!(storage.path_count(), 2);
}

#[test]
fn test_default() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::default();
    assert!(storage.is_empty());
}

#[test]
fn test_debug() {
    let storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();
    let debug_str = format!("{:?}", storage);
    assert!(debug_str.contains("TypedAssetStorage"));
    assert!(debug_str.contains("TestTexture"));
}

#[test]
fn test_slot_reuse() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    // Insert and remove to create free slot
    let h1 = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.remove(&h1);

    // New handle should have same index but different generation
    let h2 = storage.insert(TestTexture {
        width: 512,
        height: 512,
    });
    assert_eq!(h1.index(), h2.index());
    assert_ne!(h1.generation(), h2.generation());
}

#[test]
fn test_stale_path_index() {
    let mut storage: TypedAssetStorage<TestTexture> = TypedAssetStorage::new();

    let h1 = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    storage.remove(&h1);

    // Path should not return stale handle
    assert!(storage.get_handle_by_path("textures/player.png").is_none());

    // Can insert with same path again
    let h2 = storage.insert_with_path(
        TestTexture {
            width: 512,
            height: 512,
        },
        AssetPath::new("textures/player.png"),
    );

    assert!(storage.get_handle_by_path("textures/player.png").is_some());
    assert_ne!(h1, h2);
}
