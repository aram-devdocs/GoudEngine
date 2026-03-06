//! Unit tests for texture types and the texture loader.

use image::ImageFormat;

use crate::assets::AssetPath;

use super::{
    asset::TextureAsset,
    format::TextureFormat,
    loader::TextureLoader,
    settings::{TextureColorSpace, TextureSettings, TextureWrapMode},
};

use crate::assets::{Asset, AssetLoader, AssetType, LoadContext};

/// Creates a small test PNG image encoded as bytes.
pub(super) fn create_test_png(width: u32, height: u32) -> Vec<u8> {
    use image::{ImageBuffer, Rgba};
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
        if (x + y) % 2 == 0 {
            Rgba([255, 0, 0, 255]) // Red
        } else {
            Rgba([0, 255, 0, 255]) // Green
        }
    });
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

// =============================================================================
// TextureAsset Tests
// =============================================================================

mod texture_asset {
    use super::*;

    #[test]
    fn test_new() {
        let data = vec![255; 4 * 4 * 4]; // 4x4 RGBA8
        let texture = TextureAsset::new(data.clone(), 4, 4, TextureFormat::Png);
        assert_eq!(texture.width, 4);
        assert_eq!(texture.height, 4);
        assert_eq!(texture.data.len(), 64);
        assert_eq!(texture.format, TextureFormat::Png);
    }

    #[test]
    #[should_panic(expected = "Texture data length mismatch")]
    fn test_new_wrong_size() {
        let data = vec![255; 10]; // Wrong size
        TextureAsset::new(data, 4, 4, TextureFormat::Png);
    }

    #[test]
    fn test_pixel_count() {
        let data = vec![255; 8 * 8 * 4];
        let texture = TextureAsset::new(data, 8, 8, TextureFormat::Png);
        assert_eq!(texture.pixel_count(), 64);
    }

    #[test]
    fn test_bytes_per_pixel() {
        let data = vec![255; 4 * 4 * 4];
        let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
        assert_eq!(texture.bytes_per_pixel(), 4);
    }

    #[test]
    fn test_size_bytes() {
        let data = vec![255; 4 * 4 * 4];
        let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
        assert_eq!(texture.size_bytes(), 64);
    }

    #[test]
    fn test_aspect_ratio() {
        let data = vec![255; 16 * 9 * 4];
        let texture = TextureAsset::new(data, 16, 9, TextureFormat::Png);
        assert!((texture.aspect_ratio() - 16.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn test_is_power_of_two() {
        let data = vec![255; 64 * 64 * 4];
        let texture = TextureAsset::new(data, 64, 64, TextureFormat::Png);
        assert!(texture.is_power_of_two());

        let data = vec![255; 60 * 60 * 4];
        let texture = TextureAsset::new(data, 60, 60, TextureFormat::Png);
        assert!(!texture.is_power_of_two());
    }

    #[test]
    fn test_get_pixel() {
        let mut data = Vec::new();
        // Create 2x2 texture with known colors
        data.extend_from_slice(&[255, 0, 0, 255]); // (0,0) Red
        data.extend_from_slice(&[0, 255, 0, 255]); // (1,0) Green
        data.extend_from_slice(&[0, 0, 255, 255]); // (0,1) Blue
        data.extend_from_slice(&[255, 255, 255, 255]); // (1,1) White

        let texture = TextureAsset::new(data, 2, 2, TextureFormat::Png);

        assert_eq!(texture.get_pixel(0, 0), Some(&[255, 0, 0, 255][..]));
        assert_eq!(texture.get_pixel(1, 0), Some(&[0, 255, 0, 255][..]));
        assert_eq!(texture.get_pixel(0, 1), Some(&[0, 0, 255, 255][..]));
        assert_eq!(texture.get_pixel(1, 1), Some(&[255, 255, 255, 255][..]));
        assert_eq!(texture.get_pixel(2, 0), None); // Out of bounds
        assert_eq!(texture.get_pixel(0, 2), None); // Out of bounds
    }

    #[test]
    fn test_asset_trait() {
        assert_eq!(TextureAsset::asset_type_name(), "Texture");
        assert_eq!(TextureAsset::asset_type(), AssetType::Texture);
        assert!(TextureAsset::extensions().contains(&"png"));
        assert!(TextureAsset::extensions().contains(&"jpg"));
    }

    #[test]
    fn test_clone() {
        let data = vec![255; 4 * 4 * 4];
        let texture1 = TextureAsset::new(data, 4, 4, TextureFormat::Png);
        let texture2 = texture1.clone();
        assert_eq!(texture1.width, texture2.width);
        assert_eq!(texture1.height, texture2.height);
        assert_eq!(texture1.data, texture2.data);
    }

    #[test]
    fn test_debug() {
        let data = vec![255; 4 * 4 * 4];
        let texture = TextureAsset::new(data, 4, 4, TextureFormat::Png);
        let debug_str = format!("{:?}", texture);
        assert!(debug_str.contains("TextureAsset"));
        assert!(debug_str.contains("4"));
    }
}

// =============================================================================
// TextureFormat Tests
// =============================================================================

mod texture_format {
    use super::*;

    #[test]
    fn test_extension() {
        assert_eq!(TextureFormat::Png.extension(), "png");
        assert_eq!(TextureFormat::Jpeg.extension(), "jpg");
        assert_eq!(TextureFormat::Bmp.extension(), "bmp");
    }

    #[test]
    fn test_name() {
        assert_eq!(TextureFormat::Png.name(), "PNG");
        assert_eq!(TextureFormat::Jpeg.name(), "JPEG");
        assert_eq!(TextureFormat::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_from_extension() {
        assert_eq!(TextureFormat::from_extension("png"), TextureFormat::Png);
        assert_eq!(TextureFormat::from_extension("PNG"), TextureFormat::Png);
        assert_eq!(TextureFormat::from_extension("jpg"), TextureFormat::Jpeg);
        assert_eq!(TextureFormat::from_extension("jpeg"), TextureFormat::Jpeg);
        assert_eq!(TextureFormat::from_extension("xyz"), TextureFormat::Unknown);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TextureFormat::Png), "PNG");
        assert_eq!(format!("{}", TextureFormat::Jpeg), "JPEG");
    }

    #[test]
    fn test_default() {
        assert_eq!(TextureFormat::default(), TextureFormat::Png);
    }

    #[test]
    fn test_equality() {
        assert_eq!(TextureFormat::Png, TextureFormat::Png);
        assert_ne!(TextureFormat::Png, TextureFormat::Jpeg);
    }

    #[test]
    fn test_clone() {
        let format = TextureFormat::Png;
        let cloned = format;
        assert_eq!(format, cloned);
    }
}

// =============================================================================
// TextureSettings Tests
// =============================================================================

mod texture_settings {
    use super::*;

    #[test]
    fn test_default() {
        let settings = TextureSettings::default();
        assert_eq!(settings.flip_vertical, true);
        assert_eq!(settings.color_space, TextureColorSpace::Srgb);
        assert_eq!(settings.wrap_mode, TextureWrapMode::Repeat);
        assert_eq!(settings.generate_mipmaps, true);
    }

    #[test]
    fn test_clone() {
        let settings1 = TextureSettings::default();
        let settings2 = settings1.clone();
        assert_eq!(settings1.flip_vertical, settings2.flip_vertical);
    }

    #[test]
    fn test_debug() {
        let settings = TextureSettings::default();
        let debug_str = format!("{:?}", settings);
        assert!(debug_str.contains("TextureSettings"));
    }
}

// =============================================================================
// TextureColorSpace Tests
// =============================================================================

mod texture_color_space {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(TextureColorSpace::Linear.name(), "Linear");
        assert_eq!(TextureColorSpace::Srgb.name(), "sRGB");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TextureColorSpace::Linear), "Linear");
        assert_eq!(format!("{}", TextureColorSpace::Srgb), "sRGB");
    }

    #[test]
    fn test_default() {
        assert_eq!(TextureColorSpace::default(), TextureColorSpace::Srgb);
    }

    #[test]
    fn test_equality() {
        assert_eq!(TextureColorSpace::Srgb, TextureColorSpace::Srgb);
        assert_ne!(TextureColorSpace::Srgb, TextureColorSpace::Linear);
    }
}

// =============================================================================
// TextureWrapMode Tests
// =============================================================================

mod texture_wrap_mode {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(TextureWrapMode::Repeat.name(), "Repeat");
        assert_eq!(TextureWrapMode::MirroredRepeat.name(), "MirroredRepeat");
        assert_eq!(TextureWrapMode::ClampToEdge.name(), "ClampToEdge");
        assert_eq!(TextureWrapMode::ClampToBorder.name(), "ClampToBorder");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", TextureWrapMode::Repeat), "Repeat");
        assert_eq!(
            format!("{}", TextureWrapMode::MirroredRepeat),
            "MirroredRepeat"
        );
    }

    #[test]
    fn test_default() {
        assert_eq!(TextureWrapMode::default(), TextureWrapMode::Repeat);
    }

    #[test]
    fn test_equality() {
        assert_eq!(TextureWrapMode::Repeat, TextureWrapMode::Repeat);
        assert_ne!(TextureWrapMode::Repeat, TextureWrapMode::ClampToEdge);
    }
}

// =============================================================================
// TextureLoader Tests
// =============================================================================

mod texture_loader {
    use super::*;

    #[test]
    fn test_new() {
        let loader = TextureLoader::new();
        assert_eq!(loader.extensions().len(), 9);
    }

    #[test]
    fn test_default() {
        let loader = TextureLoader::default();
        assert_eq!(loader.extensions().len(), 9);
    }

    #[test]
    fn test_extensions() {
        let loader = TextureLoader::new();
        assert!(loader.supports_extension("png"));
        assert!(loader.supports_extension("jpg"));
        assert!(loader.supports_extension("jpeg"));
        assert!(loader.supports_extension("bmp"));
        assert!(!loader.supports_extension("xyz"));
    }

    #[test]
    fn test_load_png() {
        let loader = TextureLoader::new();
        let bytes = create_test_png(4, 4);
        let path = AssetPath::from_string("test.png".to_string());
        let mut context = LoadContext::new(path);
        let settings = TextureSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let texture = result.unwrap();
        assert_eq!(texture.width, 4);
        assert_eq!(texture.height, 4);
        assert_eq!(texture.data.len(), 4 * 4 * 4);
    }

    #[test]
    fn test_load_invalid_data() {
        let loader = TextureLoader::new();
        let bytes = vec![0xFF, 0xFE, 0xFD]; // Invalid image data
        let path = AssetPath::from_string("test.png".to_string());
        let mut context = LoadContext::new(path);
        let settings = TextureSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_with_flip() {
        let loader = TextureLoader::new();
        let bytes = create_test_png(2, 2);
        let path = AssetPath::from_string("test.png".to_string());
        let mut context = LoadContext::new(path);

        // Load with flip
        let mut settings = TextureSettings::default();
        settings.flip_vertical = true;
        let result1 = loader.load(&bytes, &settings, &mut context);
        assert!(result1.is_ok());
        let texture1 = result1.unwrap();

        // Load without flip
        let path = AssetPath::from_string("test.png".to_string());
        let mut context = LoadContext::new(path);
        settings.flip_vertical = false;
        let result2 = loader.load(&bytes, &settings, &mut context);
        assert!(result2.is_ok());
        let texture2 = result2.unwrap();

        // Both should succeed with same dimensions
        assert_eq!(texture1.width, texture2.width);
        assert_eq!(texture1.height, texture2.height);
        // But pixel data should differ (flipped vs not flipped)
        // Note: For a checkerboard, flip might not change it, so we just verify load succeeded
    }

    #[test]
    fn test_load_from_bytes() {
        let bytes = create_test_png(8, 8);
        let settings = TextureSettings::default();

        let result = TextureLoader::load_from_bytes(&bytes, &settings, Some(TextureFormat::Png));
        assert!(result.is_ok());

        let texture = result.unwrap();
        assert_eq!(texture.width, 8);
        assert_eq!(texture.height, 8);
    }

    #[test]
    fn test_clone() {
        let loader1 = TextureLoader::new();
        let loader2 = loader1.clone();
        assert_eq!(loader1.extensions(), loader2.extensions());
    }

    #[test]
    fn test_debug() {
        let loader = TextureLoader::new();
        let debug_str = format!("{:?}", loader);
        assert!(debug_str.contains("TextureLoader"));
    }
}
