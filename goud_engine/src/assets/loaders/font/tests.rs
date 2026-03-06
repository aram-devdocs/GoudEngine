//! Tests for font asset loading types.

use crate::assets::{asset::Asset, AssetLoader, AssetPath, AssetServer, AssetType, LoadContext};

use super::{
    asset::{FontAsset, FontStyle},
    format::FontFormat,
    loader::FontLoader,
    settings::FontSettings,
};

// ============================================================================
// FontFormat Tests
// ============================================================================

#[test]
fn test_font_format_from_extension_ttf() {
    assert_eq!(FontFormat::from_extension("ttf"), FontFormat::Ttf);
}

#[test]
fn test_font_format_from_extension_otf() {
    assert_eq!(FontFormat::from_extension("otf"), FontFormat::Otf);
}

#[test]
fn test_font_format_from_extension_case_insensitive() {
    assert_eq!(FontFormat::from_extension("TTF"), FontFormat::Ttf);
    assert_eq!(FontFormat::from_extension("OTF"), FontFormat::Otf);
}

#[test]
fn test_font_format_from_extension_unknown() {
    assert_eq!(FontFormat::from_extension("woff"), FontFormat::Unknown);
    assert_eq!(FontFormat::from_extension("xyz"), FontFormat::Unknown);
}

#[test]
fn test_font_format_extension() {
    assert_eq!(FontFormat::Ttf.extension(), "ttf");
    assert_eq!(FontFormat::Otf.extension(), "otf");
    assert_eq!(FontFormat::Unknown.extension(), "");
}

#[test]
fn test_font_format_name() {
    assert_eq!(FontFormat::Ttf.name(), "TrueType");
    assert_eq!(FontFormat::Otf.name(), "OpenType");
    assert_eq!(FontFormat::Unknown.name(), "Unknown");
}

#[test]
fn test_font_format_default() {
    assert_eq!(FontFormat::default(), FontFormat::Unknown);
}

#[test]
fn test_font_format_display() {
    assert_eq!(format!("{}", FontFormat::Ttf), "TrueType");
    assert_eq!(format!("{}", FontFormat::Otf), "OpenType");
    assert_eq!(format!("{}", FontFormat::Unknown), "Unknown");
}

#[test]
fn test_font_format_clone_and_copy() {
    let f1 = FontFormat::Ttf;
    let f2 = f1;
    assert_eq!(f1, f2);
}

#[test]
fn test_font_format_eq() {
    assert_eq!(FontFormat::Ttf, FontFormat::Ttf);
    assert_ne!(FontFormat::Ttf, FontFormat::Otf);
}

#[test]
fn test_font_format_debug() {
    let debug_str = format!("{:?}", FontFormat::Ttf);
    assert!(debug_str.contains("Ttf"));
}

// ============================================================================
// FontSettings Tests
// ============================================================================

#[test]
fn test_font_settings_default() {
    let settings = FontSettings::default();
    assert!((settings.default_size_px - 16.0).abs() < f32::EPSILON);
    assert_eq!(settings.collection_index, 0);
}

#[test]
fn test_font_settings_custom() {
    let settings = FontSettings {
        default_size_px: 24.0,
        collection_index: 2,
    };
    assert!((settings.default_size_px - 24.0).abs() < f32::EPSILON);
    assert_eq!(settings.collection_index, 2);
}

#[test]
fn test_font_settings_clone() {
    let s1 = FontSettings::default();
    let s2 = s1.clone();
    assert!((s1.default_size_px - s2.default_size_px).abs() < f32::EPSILON);
    assert_eq!(s1.collection_index, s2.collection_index);
}

#[test]
fn test_font_settings_debug() {
    let settings = FontSettings::default();
    let debug_str = format!("{:?}", settings);
    assert!(debug_str.contains("FontSettings"));
}

// ============================================================================
// FontAsset Tests
// ============================================================================

#[test]
fn test_font_asset_new() {
    let data = vec![1, 2, 3, 4];
    let asset = FontAsset::new(
        data.clone(),
        "TestFont".to_string(),
        FontStyle::Bold,
        FontFormat::Ttf,
        1000,
        95,
    );

    assert_eq!(asset.data(), &data);
    assert_eq!(asset.family_name(), "TestFont");
    assert_eq!(asset.style(), FontStyle::Bold);
    assert_eq!(asset.format(), FontFormat::Ttf);
    assert_eq!(asset.units_per_em(), 1000);
    assert_eq!(asset.glyph_count(), 95);
    assert_eq!(asset.size_bytes(), 4);
    assert!(!asset.is_empty());
}

#[test]
fn test_font_asset_is_empty() {
    let empty = FontAsset::new(
        vec![],
        String::new(),
        FontStyle::Regular,
        FontFormat::Unknown,
        0,
        0,
    );
    assert!(empty.is_empty());
    assert_eq!(empty.size_bytes(), 0);
}

#[test]
fn test_font_asset_size_bytes() {
    let asset = FontAsset::new(
        vec![0u8; 100],
        "Test".to_string(),
        FontStyle::Regular,
        FontFormat::Ttf,
        1000,
        50,
    );
    assert_eq!(asset.size_bytes(), 100);
}

#[test]
fn test_font_asset_trait_type_name() {
    assert_eq!(FontAsset::asset_type_name(), "Font");
}

#[test]
fn test_font_asset_trait_asset_type() {
    assert_eq!(FontAsset::asset_type(), AssetType::Font);
}

#[test]
fn test_font_asset_trait_extensions() {
    assert_eq!(FontAsset::extensions(), &["ttf", "otf"]);
}

#[test]
fn test_font_style_default() {
    assert_eq!(FontStyle::default(), FontStyle::Regular);
}

#[test]
fn test_font_asset_clone() {
    let a1 = FontAsset::new(
        vec![1, 2, 3],
        "Clone".to_string(),
        FontStyle::Italic,
        FontFormat::Otf,
        2048,
        200,
    );
    let a2 = a1.clone();
    assert_eq!(a1, a2);
}

#[test]
fn test_font_asset_debug() {
    let asset = FontAsset::new(
        vec![],
        "Debug".to_string(),
        FontStyle::Regular,
        FontFormat::Ttf,
        0,
        0,
    );
    let debug_str = format!("{:?}", asset);
    assert!(debug_str.contains("FontAsset"));
}

// ============================================================================
// FontLoader Tests
// ============================================================================

#[test]
fn test_font_loader_extensions() {
    let loader = FontLoader::default();
    assert_eq!(loader.extensions(), &["ttf", "otf"]);
}

#[test]
fn test_font_loader_load_invalid_bytes_returns_decode_failed() {
    let loader = FontLoader::default();
    let mut context = LoadContext::new("test.ttf".into());
    let settings = FontSettings::default();
    let invalid_bytes = vec![0, 1, 2, 3, 4, 5];

    let result = loader.load(&invalid_bytes, &settings, &mut context);
    assert!(result.is_err());

    let err = result.unwrap_err();
    assert!(err.is_decode_failed());
}

#[test]
fn test_font_loader_load_empty_bytes_returns_error() {
    let loader = FontLoader::default();
    let mut context = LoadContext::new("empty.otf".into());
    let settings = FontSettings::default();

    let result = loader.load(&[], &settings, &mut context);
    assert!(result.is_err());
}

#[test]
fn test_font_loader_clone() {
    let l1 = FontLoader::default();
    let l2 = l1.clone();
    assert_eq!(l1.extensions(), l2.extensions());
}

#[test]
fn test_font_loader_debug() {
    let loader = FontLoader::default();
    let debug_str = format!("{:?}", loader);
    assert!(debug_str.contains("FontLoader"));
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
fn test_font_asset_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<FontAsset>();
}

#[test]
fn test_font_asset_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<FontAsset>();
}

#[test]
fn test_font_loader_is_send() {
    fn assert_send<T: Send>() {}
    assert_send::<FontLoader>();
}

#[test]
fn test_font_loader_is_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<FontLoader>();
}

// ============================================================================
// Registration Test
// ============================================================================

#[test]
fn test_font_loader_registration_with_asset_server() {
    let mut server = AssetServer::new();
    server.register_loader(FontLoader::default());
    // Registration succeeds without panic -- loader is now available
    // for "ttf" and "otf" extensions.
    assert!(server.has_loader_for_extension("ttf"));
    assert!(server.has_loader_for_extension("otf"));
}

#[test]
fn test_font_loader_load_valid_ttf_success() {
    // Test that FontLoader successfully loads valid TTF bytes using an embedded
    // minimal TTF fixture that works on all platforms (Windows, macOS, Linux).
    // The fixture is a minimal but valid TTF file that fontdue can parse.

    let loader = FontLoader::default();

    // Use embedded minimal TTF fixture for platform-independent testing
    let ttf_bytes = include_bytes!("../../../../test_assets/fonts/minimal.ttf");

    let path = AssetPath::from_string("test.ttf".to_string());
    let mut context = LoadContext::new(path);
    let settings = FontSettings::default();

    let result = loader.load(ttf_bytes, &settings, &mut context);
    assert!(result.is_ok(), "Valid TTF should load successfully");

    let font = result.unwrap();
    assert_eq!(font.format(), FontFormat::Ttf);
    assert!(!font.is_empty());
    assert_eq!(font.size_bytes(), ttf_bytes.len());
    // Verify metadata was extracted
    assert!(
        !font.family_name().is_empty(),
        "Font should have a family name"
    );
}
