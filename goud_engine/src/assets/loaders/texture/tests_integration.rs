//! Integration and thread-safety tests for the texture loader.

use super::{
    asset::TextureAsset, format::TextureFormat, loader::TextureLoader, settings::TextureSettings,
    tests::create_test_png,
};

use crate::assets::{AssetLoader, AssetPath, LoadContext};

// =============================================================================
// Integration Tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // Create a test image
        let bytes = create_test_png(16, 16);

        // Create loader and load
        let loader = TextureLoader::new();
        let path = AssetPath::from_string("textures/player.png".to_string());
        let mut context = LoadContext::new(path);
        let settings = TextureSettings::default();

        let result = loader.load(&bytes, &settings, &mut context);
        assert!(result.is_ok());

        let texture = result.unwrap();
        assert_eq!(texture.width, 16);
        assert_eq!(texture.height, 16);
        assert_eq!(texture.pixel_count(), 256);
        assert!(texture.is_power_of_two());
    }

    #[test]
    fn test_different_formats() {
        use image::ImageFormat;

        let formats = vec![
            (ImageFormat::Png, "test.png"),
            (ImageFormat::Jpeg, "test.jpg"),
            (ImageFormat::Bmp, "test.bmp"),
        ];

        let loader = TextureLoader::new();

        for (format, filename) in formats {
            // Create test image in specific format
            let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> =
                image::ImageBuffer::from_pixel(4, 4, image::Rgba([255, 0, 0, 255]));
            let mut bytes = Vec::new();
            image::DynamicImage::ImageRgba8(img)
                .write_to(&mut std::io::Cursor::new(&mut bytes), format)
                .unwrap();

            let path = AssetPath::from_string(filename.to_string());
            let mut context = LoadContext::new(path);
            let settings = TextureSettings::default();

            let result = loader.load(&bytes, &settings, &mut context);
            assert!(result.is_ok(), "Failed to load {}", filename);

            let texture = result.unwrap();
            assert_eq!(texture.width, 4);
            assert_eq!(texture.height, 4);
        }
    }

    #[test]
    fn test_error_handling() {
        let loader = TextureLoader::new();
        let path = AssetPath::from_string("test.png".to_string());
        let mut context = LoadContext::new(path);
        let settings = TextureSettings::default();

        // Empty data
        let result = loader.load(&[], &settings, &mut context);
        assert!(result.is_err());

        // Invalid PNG header
        let path = AssetPath::from_string("test.png".to_string());
        let mut context = LoadContext::new(path);
        let result = loader.load(b"not a png", &settings, &mut context);
        assert!(result.is_err());
    }
}

// =============================================================================
// Thread Safety Tests
// =============================================================================

mod thread_safety {
    use super::*;

    #[test]
    fn test_texture_asset_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TextureAsset>();
    }

    #[test]
    fn test_texture_asset_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TextureAsset>();
    }

    #[test]
    fn test_texture_loader_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TextureLoader>();
    }

    #[test]
    fn test_texture_loader_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TextureLoader>();
    }

    #[test]
    fn test_texture_settings_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TextureSettings>();
    }

    #[test]
    fn test_texture_settings_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<TextureSettings>();
    }
}
