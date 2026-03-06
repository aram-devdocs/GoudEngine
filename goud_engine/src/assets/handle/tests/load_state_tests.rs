//! Tests for HandleLoadState.

use crate::assets::AssetState;

use super::super::load_state::HandleLoadState;
use super::super::typed::AssetHandle;
use super::common::TestTexture;

#[test]
fn test_new() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let state = HandleLoadState::new(handle, AssetState::Loaded);

    assert_eq!(*state.handle(), handle);
    assert!(state.is_ready());
}

#[test]
fn test_invalid() {
    let state: HandleLoadState<TestTexture> = HandleLoadState::invalid();
    assert!(!state.is_valid());
    assert!(!state.is_ready());
    assert_eq!(*state.state(), AssetState::NotLoaded);
}

#[test]
fn test_is_ready() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);

    let loaded = HandleLoadState::new(handle, AssetState::Loaded);
    assert!(loaded.is_ready());

    let loading = HandleLoadState::new(handle, AssetState::Loading { progress: 0.5 });
    assert!(!loading.is_ready());

    let failed = HandleLoadState::new(
        handle,
        AssetState::Failed {
            error: "test".to_string(),
        },
    );
    assert!(!failed.is_ready());
}

#[test]
fn test_is_loading() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.5 });

    assert!(state.is_loading());
    assert_eq!(state.progress(), Some(0.5));
}

#[test]
fn test_is_failed() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let state = HandleLoadState::new(
        handle,
        AssetState::Failed {
            error: "File not found".to_string(),
        },
    );

    assert!(state.is_failed());
    assert_eq!(state.error(), Some("File not found"));
}

#[test]
fn test_into_handle() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let state = HandleLoadState::new(handle, AssetState::Loaded);

    let recovered = state.into_handle();
    assert_eq!(recovered, handle);
}

#[test]
fn test_set_state() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let mut state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.0 });

    assert!(state.is_loading());

    state.set_state(AssetState::Loaded);
    assert!(state.is_ready());
}

#[test]
fn test_default() {
    let state: HandleLoadState<TestTexture> = Default::default();
    assert!(!state.is_valid());
}

#[test]
fn test_clone() {
    let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
    let state1 = HandleLoadState::new(handle, AssetState::Loaded);
    let state2 = state1.clone();

    assert_eq!(state1, state2);
}
