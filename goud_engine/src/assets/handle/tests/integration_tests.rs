//! Integration tests for handle types working together.

use crate::assets::{AssetState, AssetType};

use super::super::load_state::HandleLoadState;
use super::super::path::AssetPath;
use super::super::typed::{AssetHandle, WeakAssetHandle};
use super::super::untyped::UntypedAssetHandle;
use super::common::{TestAudio, TestTexture};

#[test]
fn test_handle_lifecycle() {
    // Simulate asset lifecycle with handles
    let handle: AssetHandle<TestTexture> = AssetHandle::new(0, 1);

    // Initially loading
    let mut state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.0 });
    assert!(state.is_loading());

    // Progress update
    state.set_state(AssetState::Loading { progress: 0.5 });
    assert_eq!(state.progress(), Some(0.5));

    // Loaded
    state.set_state(AssetState::Loaded);
    assert!(state.is_ready());
}

#[test]
fn test_mixed_handle_collection() {
    // Store different asset types in single collection
    let tex_handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let audio_handle: AssetHandle<TestAudio> = AssetHandle::new(2, 1);

    let handles: Vec<UntypedAssetHandle> = vec![tex_handle.untyped(), audio_handle.untyped()];

    // Can filter by type
    let textures: Vec<_> = handles
        .iter()
        .filter_map(|h| h.typed::<TestTexture>())
        .collect();
    assert_eq!(textures.len(), 1);
    assert_eq!(textures[0], tex_handle);
}

#[test]
fn test_path_to_handle_workflow() {
    // Simulate path -> handle workflow
    let path = AssetPath::new("textures/player.png");
    assert_eq!(path.extension(), Some("png"));

    // Asset system would create handle
    let _handle: AssetHandle<TestTexture> = AssetHandle::new(0, 1);

    // Check type matches extension
    assert_eq!(AssetHandle::<TestTexture>::asset_type(), AssetType::Texture);
}

#[test]
fn test_weak_handle_usage() {
    // Strong handle
    let strong: AssetHandle<TestTexture> = AssetHandle::new(1, 1);

    // Create weak reference (for cache)
    let weak = WeakAssetHandle::from_handle(&strong);

    // Upgrade when needed
    let upgraded = weak.upgrade();
    assert_eq!(upgraded, strong);
}
