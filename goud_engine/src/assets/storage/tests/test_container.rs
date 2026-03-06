//! Tests for [`AssetStorage`].

use crate::assets::{AssetPath, AssetStorage};

use super::fixtures::{TestAudio, TestTexture};

#[test]
fn test_new() {
    let storage = AssetStorage::new();
    assert!(storage.is_empty());
    assert_eq!(storage.type_count(), 0);
}

#[test]
fn test_insert() {
    let mut storage = AssetStorage::new();
    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    assert!(handle.is_valid());
    assert!(storage.is_alive(&handle));
    assert_eq!(storage.type_count(), 1);
}

#[test]
fn test_insert_multiple_types() {
    let mut storage = AssetStorage::new();

    let tex_handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let audio_handle = storage.insert(TestAudio { duration: 2.5 });

    assert!(storage.is_alive(&tex_handle));
    assert!(storage.is_alive(&audio_handle));
    assert_eq!(storage.type_count(), 2);
    assert_eq!(storage.total_len(), 2);
}

#[test]
fn test_insert_with_path() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    assert!(storage.has_path::<TestTexture>("textures/player.png"));
    assert_eq!(
        storage.get_handle_by_path::<TestTexture>("textures/player.png"),
        Some(handle)
    );
}

#[test]
fn test_reserve() {
    let mut storage = AssetStorage::new();

    let handle = storage.reserve::<TestTexture>();
    assert!(storage.is_alive(&handle));
    assert!(storage.get::<TestTexture>(&handle).is_none());
}

#[test]
fn test_set_loaded() {
    let mut storage = AssetStorage::new();

    let handle = storage.reserve::<TestTexture>();
    storage.set_loaded(
        &handle,
        TestTexture {
            width: 256,
            height: 256,
        },
    );

    assert_eq!(storage.get::<TestTexture>(&handle).unwrap().width, 256);
}

#[test]
fn test_remove() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let removed = storage.remove::<TestTexture>(&handle);

    assert!(removed.is_some());
    assert!(!storage.is_alive(&handle));
}

#[test]
fn test_remove_untyped() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let untyped = handle.untyped();

    let result = storage.remove_untyped(&untyped);
    assert!(result);
    assert!(!storage.is_alive(&handle));
}

#[test]
fn test_get() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let asset = storage.get::<TestTexture>(&handle);

    assert!(asset.is_some());
    assert_eq!(asset.unwrap().width, 256);
}

#[test]
fn test_get_mut() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    if let Some(asset) = storage.get_mut::<TestTexture>(&handle) {
        asset.width = 512;
    }

    assert_eq!(storage.get::<TestTexture>(&handle).unwrap().width, 512);
}

#[test]
fn test_get_entry() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let entry = storage.get_entry::<TestTexture>(&handle);

    assert!(entry.is_some());
    assert!(entry.unwrap().is_loaded());
}

#[test]
fn test_is_alive_untyped() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let untyped = handle.untyped();

    assert!(storage.is_alive_untyped(&untyped));

    storage.remove_untyped(&untyped);
    assert!(!storage.is_alive_untyped(&untyped));
}

#[test]
fn test_get_state() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let state = storage.get_state::<TestTexture>(&handle);

    assert!(state.is_some());
    assert!(state.unwrap().is_ready());
}

#[test]
fn test_get_state_untyped() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let untyped = handle.untyped();

    let state = storage.get_state_untyped(&untyped);
    assert!(state.is_some());
    assert!(state.unwrap().is_ready());
}

#[test]
fn test_get_by_path() {
    let mut storage = AssetStorage::new();

    storage.insert_with_path(
        TestTexture {
            width: 256,
            height: 256,
        },
        AssetPath::new("textures/player.png"),
    );

    let asset = storage.get_by_path::<TestTexture>("textures/player.png");
    assert!(asset.is_some());
    assert_eq!(asset.unwrap().width, 256);
}

#[test]
fn test_set_path() {
    let mut storage = AssetStorage::new();

    let handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.set_path(&handle, AssetPath::new("textures/player.png"));

    assert!(storage.has_path::<TestTexture>("textures/player.png"));
}

#[test]
fn test_len() {
    let mut storage = AssetStorage::new();

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.insert(TestTexture {
        width: 512,
        height: 512,
    });
    storage.insert(TestAudio { duration: 2.5 });

    assert_eq!(storage.len::<TestTexture>(), 2);
    assert_eq!(storage.len::<TestAudio>(), 1);
    assert_eq!(storage.total_len(), 3);
}

#[test]
fn test_is_empty_type() {
    let mut storage = AssetStorage::new();

    assert!(storage.is_empty_type::<TestTexture>());

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    assert!(!storage.is_empty_type::<TestTexture>());
    assert!(storage.is_empty_type::<TestAudio>());
}

#[test]
fn test_clear_type() {
    let mut storage = AssetStorage::new();

    let tex_handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.insert(TestAudio { duration: 2.5 });

    storage.clear_type::<TestTexture>();

    assert!(!storage.is_alive(&tex_handle));
    assert_eq!(storage.len::<TestTexture>(), 0);
    assert_eq!(storage.len::<TestAudio>(), 1);
}

#[test]
fn test_clear() {
    let mut storage = AssetStorage::new();

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.insert(TestAudio { duration: 2.5 });

    storage.clear();

    assert_eq!(storage.total_len(), 0);
}

#[test]
fn test_has_type() {
    let mut storage = AssetStorage::new();

    assert!(!storage.has_type::<TestTexture>());

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    assert!(storage.has_type::<TestTexture>());
}

#[test]
fn test_registered_types() {
    let mut storage = AssetStorage::new();

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.insert(TestAudio { duration: 2.5 });

    let types = storage.registered_types();
    assert_eq!(types.len(), 2);
}

#[test]
fn test_get_storage() {
    let mut storage = AssetStorage::new();

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });

    let typed_storage = storage.get_storage::<TestTexture>();
    assert!(typed_storage.is_some());
    assert_eq!(typed_storage.unwrap().len(), 1);
}

#[test]
fn test_get_or_create_storage() {
    let mut storage = AssetStorage::new();

    // Should create storage on first access
    let typed = storage.get_or_create_storage::<TestTexture>();
    typed.insert(TestTexture {
        width: 256,
        height: 256,
    });

    assert_eq!(storage.len::<TestTexture>(), 1);
}

#[test]
fn test_iter() {
    let mut storage = AssetStorage::new();

    storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    storage.insert(TestTexture {
        width: 512,
        height: 512,
    });

    let pairs: Vec<_> = storage.iter::<TestTexture>().collect();
    assert_eq!(pairs.len(), 2);
}

#[test]
fn test_handles() {
    let mut storage = AssetStorage::new();

    let h1 = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let h2 = storage.insert(TestTexture {
        width: 512,
        height: 512,
    });

    let handles: Vec<_> = storage.handles::<TestTexture>().collect();
    assert!(handles.contains(&h1));
    assert!(handles.contains(&h2));
}

#[test]
fn test_default() {
    let storage = AssetStorage::default();
    assert!(storage.is_empty());
}

#[test]
fn test_debug() {
    let storage = AssetStorage::new();
    let debug_str = format!("{:?}", storage);
    assert!(debug_str.contains("AssetStorage"));
}

#[test]
fn test_type_isolation() {
    let mut storage = AssetStorage::new();

    // Insert same index for different types
    let tex_handle = storage.insert(TestTexture {
        width: 256,
        height: 256,
    });
    let audio_handle = storage.insert(TestAudio { duration: 2.5 });

    // Should not interfere with each other
    storage.remove::<TestTexture>(&tex_handle);

    assert!(!storage.is_alive(&tex_handle));
    assert!(storage.is_alive(&audio_handle));
}

#[test]
fn test_stress_multiple_types() {
    let mut storage = AssetStorage::new();

    // Insert many assets of multiple types
    for i in 0..1000 {
        storage.insert(TestTexture {
            width: i,
            height: i,
        });
        storage.insert(TestAudio { duration: i as f32 });
    }

    assert_eq!(storage.len::<TestTexture>(), 1000);
    assert_eq!(storage.len::<TestAudio>(), 1000);
    assert_eq!(storage.total_len(), 2000);
}
