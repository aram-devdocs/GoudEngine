//! Tests for the fallback asset registry.

use super::FallbackRegistry;
use crate::assets::loaders::audio::AudioAsset;
use crate::assets::loaders::mesh::MeshAsset;
use crate::assets::loaders::{TextureAsset, TextureFormat};
use crate::assets::Asset;

// ---- Test asset types -------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct DummyAsset {
    value: i32,
}

impl Asset for DummyAsset {}

#[derive(Debug)]
struct NonCloneAsset;

impl Asset for NonCloneAsset {}

// ---- FallbackRegistry::new --------------------------------------------------

#[test]
fn test_new_registry_is_empty() {
    let registry = FallbackRegistry::new();

    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

// ---- FallbackRegistry::with_defaults ----------------------------------------

#[test]
fn test_with_defaults_registers_texture() {
    let registry = FallbackRegistry::with_defaults();

    assert!(registry.has::<TextureAsset>());
    assert_eq!(registry.len(), 3); // texture + audio + mesh
}

#[test]
fn test_default_texture_is_magenta_1x1() {
    let registry = FallbackRegistry::with_defaults();
    let tex = registry.get_cloned::<TextureAsset>().unwrap();

    assert_eq!(tex.width, 1);
    assert_eq!(tex.height, 1);
    assert_eq!(tex.data, vec![255, 0, 255, 255]);
}

#[test]
fn test_with_defaults_registers_audio() {
    let registry = FallbackRegistry::with_defaults();

    assert!(registry.has::<AudioAsset>());
    let audio = registry.get_cloned::<AudioAsset>().unwrap();
    assert!(audio.is_empty());
    assert_eq!(audio.sample_rate(), 44100);
    assert_eq!(audio.channel_count(), 2);
}

#[test]
fn test_with_defaults_registers_mesh() {
    let registry = FallbackRegistry::with_defaults();

    assert!(registry.has::<MeshAsset>());
    let mesh = registry.get_cloned::<MeshAsset>().unwrap();
    assert_eq!(mesh.vertex_count(), 3);
    assert_eq!(mesh.triangle_count(), 1);
}

// ---- register / get_cloned --------------------------------------------------

#[test]
fn test_register_and_retrieve_custom_fallback() {
    let mut registry = FallbackRegistry::new();
    registry.register(DummyAsset { value: 42 });

    let fallback = registry.get_cloned::<DummyAsset>();

    assert!(fallback.is_some());
    assert_eq!(fallback.unwrap(), DummyAsset { value: 42 });
}

#[test]
fn test_get_cloned_returns_independent_clones() {
    let mut registry = FallbackRegistry::new();
    registry.register(DummyAsset { value: 7 });

    let a = registry.get_cloned::<DummyAsset>().unwrap();
    let b = registry.get_cloned::<DummyAsset>().unwrap();

    assert_eq!(a, b);
}

#[test]
fn test_get_cloned_returns_none_for_unregistered() {
    let registry = FallbackRegistry::new();

    assert!(registry.get_cloned::<DummyAsset>().is_none());
}

#[test]
fn test_has_returns_false_for_unregistered() {
    let registry = FallbackRegistry::new();

    assert!(!registry.has::<DummyAsset>());
}

#[test]
fn test_register_overwrites_previous() {
    let mut registry = FallbackRegistry::new();
    registry.register(DummyAsset { value: 1 });
    registry.register(DummyAsset { value: 2 });

    let fallback = registry.get_cloned::<DummyAsset>().unwrap();
    assert_eq!(fallback.value, 2);
    assert_eq!(registry.len(), 1);
}

// ---- Default trait ----------------------------------------------------------

#[test]
fn test_default_trait_includes_defaults() {
    let registry = FallbackRegistry::default();

    assert!(registry.has::<TextureAsset>());
    assert!(registry.has::<AudioAsset>());
    assert!(registry.has::<MeshAsset>());
}

// ---- len / is_empty ---------------------------------------------------------

#[test]
fn test_len_reflects_registered_count() {
    let mut registry = FallbackRegistry::new();
    assert_eq!(registry.len(), 0);

    registry.register(DummyAsset { value: 0 });
    assert_eq!(registry.len(), 1);

    registry.register(TextureAsset {
        data: vec![0, 0, 0, 255],
        width: 1,
        height: 1,
        format: TextureFormat::Png,
    });
    assert_eq!(registry.len(), 2);
}
