//! Tests for the Asset trait, AssetId, and AssetType.

use super::fixtures::{SimpleAsset, TestAudio, TestTexture};
use crate::assets::asset::{asset_id::AssetId, asset_type::AssetType, trait_def::Asset};
use std::any::TypeId;

// =========================================================================
// Asset Trait Tests
// =========================================================================

#[test]
fn test_trait_asset_type_name() {
    assert_eq!(TestTexture::asset_type_name(), "TestTexture");
    assert_eq!(TestAudio::asset_type_name(), "TestAudio");
}

#[test]
fn test_asset_type_name_default() {
    let name = SimpleAsset::asset_type_name();
    assert!(name.contains("SimpleAsset"));
}

#[test]
fn test_asset_type() {
    assert_eq!(TestTexture::asset_type(), AssetType::Texture);
    assert_eq!(TestAudio::asset_type(), AssetType::Audio);
}

#[test]
fn test_trait_asset_type_default() {
    assert_eq!(SimpleAsset::asset_type(), AssetType::Custom);
}

#[test]
fn test_extensions() {
    assert!(TestTexture::extensions().contains(&"png"));
    assert!(TestTexture::extensions().contains(&"jpg"));
    assert!(TestAudio::extensions().contains(&"wav"));
}

#[test]
fn test_extensions_default() {
    assert!(SimpleAsset::extensions().is_empty());
}

#[test]
fn test_asset_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<TestTexture>();
    requires_send::<TestAudio>();
    requires_send::<SimpleAsset>();
}

#[test]
fn test_asset_is_sync() {
    fn requires_sync<T: Sync>() {}
    requires_sync::<TestTexture>();
    requires_sync::<TestAudio>();
    requires_sync::<SimpleAsset>();
}

#[test]
fn test_asset_is_static() {
    fn requires_static<T: 'static>() {}
    requires_static::<TestTexture>();
    requires_static::<TestAudio>();
    requires_static::<SimpleAsset>();
}

#[test]
fn test_asset_trait_bounds() {
    fn requires_asset<T: Asset>() {}
    requires_asset::<TestTexture>();
    requires_asset::<TestAudio>();
    requires_asset::<SimpleAsset>();
}

// =========================================================================
// AssetId Tests
// =========================================================================

#[test]
fn test_asset_id_of() {
    let id1 = AssetId::of::<TestTexture>();
    let id2 = AssetId::of::<TestTexture>();
    assert_eq!(id1, id2);
}

#[test]
fn test_asset_id_different_types() {
    let tex_id = AssetId::of::<TestTexture>();
    let audio_id = AssetId::of::<TestAudio>();
    assert_ne!(tex_id, audio_id);
}

#[test]
fn test_asset_id_of_raw() {
    let id1 = AssetId::of::<TestTexture>();
    let id2 = AssetId::of_raw::<TestTexture>();
    assert_eq!(id1, id2);
}

#[test]
fn test_asset_id_type_id() {
    let id = AssetId::of::<TestTexture>();
    assert_eq!(id.type_id(), TypeId::of::<TestTexture>());
}

#[test]
fn test_asset_id_debug() {
    let id = AssetId::of::<TestTexture>();
    let debug_str = format!("{:?}", id);
    assert!(debug_str.contains("AssetId"));
}

#[test]
fn test_asset_id_display() {
    let id = AssetId::of::<TestTexture>();
    let display_str = format!("{}", id);
    assert!(display_str.contains("AssetId"));
}

#[test]
fn test_asset_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(AssetId::of::<TestTexture>());
    set.insert(AssetId::of::<TestAudio>());
    assert_eq!(set.len(), 2);
    set.insert(AssetId::of::<TestTexture>());
    assert_eq!(set.len(), 2);
}

#[test]
fn test_asset_id_ord() {
    use std::collections::BTreeSet;
    let mut set = BTreeSet::new();
    set.insert(AssetId::of::<TestTexture>());
    set.insert(AssetId::of::<TestAudio>());
    assert_eq!(set.len(), 2);
}

#[test]
fn test_asset_id_clone() {
    let id1 = AssetId::of::<TestTexture>();
    let id2 = id1;
    assert_eq!(id1, id2);
}

#[test]
fn test_asset_id_copy() {
    let id1 = AssetId::of::<TestTexture>();
    let id2 = id1;
    assert_eq!(id1.type_id(), id2.type_id());
}

// =========================================================================
// AssetType Tests
// =========================================================================

#[test]
fn test_asset_type_all() {
    let types = AssetType::all();
    assert_eq!(types.len(), AssetType::count());
    assert!(types.contains(&AssetType::Custom));
    assert!(types.contains(&AssetType::Texture));
    assert!(types.contains(&AssetType::Audio));
}

#[test]
fn test_asset_type_count() {
    assert_eq!(AssetType::count(), 13);
}

#[test]
fn test_asset_type_is_gpu_asset() {
    assert!(AssetType::Texture.is_gpu_asset());
    assert!(AssetType::Mesh.is_gpu_asset());
    assert!(AssetType::Shader.is_gpu_asset());
    assert!(AssetType::Font.is_gpu_asset());
    assert!(!AssetType::Audio.is_gpu_asset());
    assert!(!AssetType::Config.is_gpu_asset());
    assert!(!AssetType::Custom.is_gpu_asset());
}

#[test]
fn test_asset_type_is_streamable() {
    assert!(AssetType::Audio.is_streamable());
    assert!(!AssetType::Texture.is_streamable());
    assert!(!AssetType::Mesh.is_streamable());
    assert!(!AssetType::Custom.is_streamable());
}

#[test]
fn test_asset_type_name() {
    assert_eq!(AssetType::Custom.name(), "Custom");
    assert_eq!(AssetType::Texture.name(), "Texture");
    assert_eq!(AssetType::Audio.name(), "Audio");
    assert_eq!(AssetType::Mesh.name(), "Mesh");
    assert_eq!(AssetType::Shader.name(), "Shader");
    assert_eq!(AssetType::Font.name(), "Font");
    assert_eq!(AssetType::Material.name(), "Material");
    assert_eq!(AssetType::Animation.name(), "Animation");
    assert_eq!(AssetType::TiledMap.name(), "TiledMap");
    assert_eq!(AssetType::Prefab.name(), "Prefab");
    assert_eq!(AssetType::Config.name(), "Config");
    assert_eq!(AssetType::Binary.name(), "Binary");
    assert_eq!(AssetType::Text.name(), "Text");
}

#[test]
fn test_asset_type_default() {
    assert_eq!(AssetType::default(), AssetType::Custom);
}

#[test]
fn test_asset_type_display() {
    assert_eq!(format!("{}", AssetType::Texture), "Texture");
    assert_eq!(format!("{}", AssetType::Audio), "Audio");
}

#[test]
fn test_asset_type_from_u8() {
    assert_eq!(u8::from(AssetType::Custom), 0);
    assert_eq!(u8::from(AssetType::Texture), 1);
    assert_eq!(u8::from(AssetType::Audio), 2);
    assert_eq!(u8::from(AssetType::Mesh), 3);
    assert_eq!(u8::from(AssetType::Shader), 4);
    assert_eq!(u8::from(AssetType::Font), 5);
    assert_eq!(u8::from(AssetType::Material), 6);
    assert_eq!(u8::from(AssetType::Animation), 7);
    assert_eq!(u8::from(AssetType::TiledMap), 8);
    assert_eq!(u8::from(AssetType::Prefab), 9);
    assert_eq!(u8::from(AssetType::Config), 10);
    assert_eq!(u8::from(AssetType::Binary), 11);
    assert_eq!(u8::from(AssetType::Text), 12);
}

#[test]
fn test_asset_type_try_from_u8() {
    assert_eq!(AssetType::try_from(0), Ok(AssetType::Custom));
    assert_eq!(AssetType::try_from(1), Ok(AssetType::Texture));
    assert_eq!(AssetType::try_from(2), Ok(AssetType::Audio));
    assert_eq!(AssetType::try_from(12), Ok(AssetType::Text));
    assert_eq!(AssetType::try_from(13), Err(13));
    assert_eq!(AssetType::try_from(255), Err(255));
}

#[test]
fn test_asset_type_roundtrip_conversion() {
    for asset_type in AssetType::all() {
        let value: u8 = (*asset_type).into();
        let recovered = AssetType::try_from(value).unwrap();
        assert_eq!(*asset_type, recovered);
    }
}

#[test]
fn test_asset_type_clone() {
    let t1 = AssetType::Texture;
    let t2 = t1;
    assert_eq!(t1, t2);
}

#[test]
fn test_asset_type_debug() {
    let debug_str = format!("{:?}", AssetType::Texture);
    assert!(debug_str.contains("Texture"));
}

#[test]
fn test_asset_type_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(AssetType::Texture);
    set.insert(AssetType::Audio);
    assert_eq!(set.len(), 2);
}
