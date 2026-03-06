//! Tests for AssetState, AssetInfo, thread safety, and integration scenarios.

use super::fixtures::{SimpleAsset, TestAudio, TestTexture};
use crate::assets::asset::{
    asset_id::AssetId, asset_info::AssetInfo, asset_state::AssetState, asset_type::AssetType,
};

// =========================================================================
// AssetState Tests
// =========================================================================

#[test]
fn test_asset_state_is_ready() {
    assert!(AssetState::Loaded.is_ready());
    assert!(!AssetState::NotLoaded.is_ready());
    assert!(!AssetState::Loading { progress: 0.5 }.is_ready());
    assert!(!AssetState::Failed {
        error: "test".to_string()
    }
    .is_ready());
    assert!(!AssetState::Unloaded.is_ready());
}

#[test]
fn test_asset_state_is_loading() {
    assert!(AssetState::Loading { progress: 0.0 }.is_loading());
    assert!(AssetState::Loading { progress: 0.5 }.is_loading());
    assert!(AssetState::Loading { progress: 1.0 }.is_loading());
    assert!(!AssetState::NotLoaded.is_loading());
    assert!(!AssetState::Loaded.is_loading());
}

#[test]
fn test_asset_state_is_failed() {
    assert!(AssetState::Failed {
        error: "error".to_string()
    }
    .is_failed());
    assert!(!AssetState::Loaded.is_failed());
    assert!(!AssetState::Loading { progress: 0.5 }.is_failed());
}

#[test]
fn test_asset_state_progress() {
    assert_eq!(AssetState::Loading { progress: 0.0 }.progress(), Some(0.0));
    assert_eq!(AssetState::Loading { progress: 0.5 }.progress(), Some(0.5));
    assert_eq!(AssetState::Loading { progress: 1.0 }.progress(), Some(1.0));
    assert_eq!(AssetState::NotLoaded.progress(), None);
    assert_eq!(AssetState::Loaded.progress(), None);
}

#[test]
fn test_asset_state_error() {
    let state = AssetState::Failed {
        error: "File not found".to_string(),
    };
    assert_eq!(state.error(), Some("File not found"));
    assert_eq!(AssetState::Loaded.error(), None);
    assert_eq!(AssetState::Loading { progress: 0.5 }.error(), None);
}

#[test]
fn test_asset_state_discriminant() {
    assert_eq!(AssetState::NotLoaded.discriminant(), 0);
    assert_eq!(AssetState::Loading { progress: 0.0 }.discriminant(), 1);
    assert_eq!(AssetState::Loaded.discriminant(), 2);
    assert_eq!(
        AssetState::Failed {
            error: "".to_string()
        }
        .discriminant(),
        3
    );
    assert_eq!(AssetState::Unloaded.discriminant(), 4);
}

#[test]
fn test_asset_state_default() {
    assert_eq!(AssetState::default(), AssetState::NotLoaded);
}

#[test]
fn test_asset_state_display() {
    assert_eq!(format!("{}", AssetState::NotLoaded), "NotLoaded");
    assert_eq!(
        format!("{}", AssetState::Loading { progress: 0.5 }),
        "Loading(50%)"
    );
    assert_eq!(format!("{}", AssetState::Loaded), "Loaded");
    assert_eq!(
        format!(
            "{}",
            AssetState::Failed {
                error: "oops".to_string()
            }
        ),
        "Failed(oops)"
    );
    assert_eq!(format!("{}", AssetState::Unloaded), "Unloaded");
}

#[test]
fn test_asset_state_clone() {
    let state1 = AssetState::Loading { progress: 0.75 };
    let state2 = state1.clone();
    assert_eq!(state1, state2);
}

#[test]
fn test_asset_state_eq() {
    assert_eq!(AssetState::Loaded, AssetState::Loaded);
    assert_ne!(AssetState::Loaded, AssetState::NotLoaded);
    assert_eq!(
        AssetState::Loading { progress: 0.5 },
        AssetState::Loading { progress: 0.5 }
    );
    assert_ne!(
        AssetState::Loading { progress: 0.5 },
        AssetState::Loading { progress: 0.6 }
    );
}

#[test]
fn test_asset_state_debug() {
    let debug_str = format!("{:?}", AssetState::Loaded);
    assert!(debug_str.contains("Loaded"));
}

// =========================================================================
// AssetInfo Tests
// =========================================================================

#[test]
fn test_asset_info_of() {
    let info = AssetInfo::of::<TestTexture>();
    assert_eq!(info.name, "TestTexture");
    assert_eq!(info.asset_type, AssetType::Texture);
}

#[test]
fn test_asset_info_id() {
    let info = AssetInfo::of::<TestTexture>();
    assert_eq!(info.id, AssetId::of::<TestTexture>());
}

#[test]
fn test_asset_info_size() {
    let info = AssetInfo::of::<TestTexture>();
    assert_eq!(info.size, std::mem::size_of::<TestTexture>());
}

#[test]
fn test_asset_info_align() {
    let info = AssetInfo::of::<TestTexture>();
    assert_eq!(info.align, std::mem::align_of::<TestTexture>());
}

#[test]
fn test_asset_info_extensions() {
    let info = AssetInfo::of::<TestTexture>();
    assert!(info.extensions.contains(&"png"));
    assert!(info.extensions.contains(&"jpg"));
}

#[test]
fn test_asset_info_display() {
    let info = AssetInfo::of::<TestTexture>();
    let display_str = format!("{}", info);
    assert!(display_str.contains("TestTexture"));
    assert!(display_str.contains("Texture"));
}

#[test]
fn test_asset_info_debug() {
    let info = AssetInfo::of::<TestTexture>();
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("AssetInfo"));
    assert!(debug_str.contains("TestTexture"));
}

#[test]
fn test_asset_info_clone() {
    let info1 = AssetInfo::of::<TestTexture>();
    let info2 = info1.clone();
    assert_eq!(info1.name, info2.name);
    assert_eq!(info1.id, info2.id);
}

#[test]
fn test_asset_info_default_asset() {
    let info = AssetInfo::of::<SimpleAsset>();
    assert_eq!(info.asset_type, AssetType::Custom);
    assert!(info.extensions.is_empty());
}

// =========================================================================
// Thread Safety Tests
// =========================================================================

#[test]
fn test_asset_id_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<AssetId>();
}

#[test]
fn test_asset_id_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<AssetId>();
}

#[test]
fn test_asset_type_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<AssetType>();
}

#[test]
fn test_asset_type_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<AssetType>();
}

#[test]
fn test_asset_state_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<AssetState>();
}

#[test]
fn test_asset_state_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<AssetState>();
}

#[test]
fn test_asset_info_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<AssetInfo>();
}

#[test]
fn test_asset_info_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<AssetInfo>();
}

// =========================================================================
// Integration Tests
// =========================================================================

#[test]
fn test_asset_workflow() {
    use crate::assets::asset::trait_def::Asset;

    struct GameTexture {
        #[allow(dead_code)]
        id: u32,
        #[allow(dead_code)]
        width: u32,
        #[allow(dead_code)]
        height: u32,
    }

    impl Asset for GameTexture {
        fn asset_type_name() -> &'static str {
            "GameTexture"
        }

        fn asset_type() -> AssetType {
            AssetType::Texture
        }

        fn extensions() -> &'static [&'static str] {
            &["png", "dds"]
        }
    }

    let info = AssetInfo::of::<GameTexture>();
    assert_eq!(info.name, "GameTexture");
    assert!(info.asset_type.is_gpu_asset());

    let id = AssetId::of::<GameTexture>();
    assert_eq!(id, info.id);
}

#[test]
fn test_multiple_asset_types() {
    use std::collections::HashMap;

    let mut registry: HashMap<AssetId, AssetInfo> = HashMap::new();
    registry.insert(AssetId::of::<TestTexture>(), AssetInfo::of::<TestTexture>());
    registry.insert(AssetId::of::<TestAudio>(), AssetInfo::of::<TestAudio>());
    registry.insert(AssetId::of::<SimpleAsset>(), AssetInfo::of::<SimpleAsset>());

    assert_eq!(registry.len(), 3);

    let tex_info = registry.get(&AssetId::of::<TestTexture>()).unwrap();
    assert_eq!(tex_info.name, "TestTexture");

    let audio_info = registry.get(&AssetId::of::<TestAudio>()).unwrap();
    assert_eq!(audio_info.name, "TestAudio");
}

#[test]
fn test_asset_state_transitions() {
    let mut state = AssetState::NotLoaded;
    assert!(!state.is_ready());

    state = AssetState::Loading { progress: 0.0 };
    assert!(state.is_loading());
    assert_eq!(state.progress(), Some(0.0));

    state = AssetState::Loading { progress: 0.5 };
    assert_eq!(state.progress(), Some(0.5));

    state = AssetState::Loading { progress: 1.0 };
    assert_eq!(state.progress(), Some(1.0));

    state = AssetState::Loaded;
    assert!(state.is_ready());
    assert!(!state.is_loading());
}

#[test]
fn test_asset_state_failure_path() {
    let mut state = AssetState::NotLoaded;

    state = AssetState::Loading { progress: 0.3 };
    assert!(state.is_loading());

    state = AssetState::Failed {
        error: "File not found: player.png".to_string(),
    };
    assert!(state.is_failed());
    assert_eq!(state.error(), Some("File not found: player.png"));
    assert!(!state.is_ready());
}
