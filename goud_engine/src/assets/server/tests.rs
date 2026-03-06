//! Tests for `AssetServer`.

#[cfg(test)]
mod tests {
    use crate::assets::{
        Asset, AssetHandle, AssetLoadError, AssetLoader, AssetServer, AssetState, AssetType,
        LoadContext,
    };
    use std::path::Path;

    // Test asset types
    #[derive(Debug, Clone, PartialEq)]
    struct TestAsset {
        content: String,
    }

    impl Asset for TestAsset {
        fn asset_type_name() -> &'static str {
            "TestAsset"
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestTexture {
        width: u32,
        height: u32,
    }

    impl Asset for TestTexture {
        fn asset_type_name() -> &'static str {
            "TestTexture"
        }

        fn asset_type() -> AssetType {
            AssetType::Texture
        }
    }

    // Test loaders
    #[derive(Clone)]
    struct TestAssetLoader;

    impl AssetLoader for TestAssetLoader {
        type Asset = TestAsset;
        type Settings = ();

        fn extensions(&self) -> &[&str] {
            &["test"]
        }

        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            _settings: &'a Self::Settings,
            _context: &'a mut LoadContext,
        ) -> Result<Self::Asset, AssetLoadError> {
            let content = String::from_utf8(bytes.to_vec())
                .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
            Ok(TestAsset { content })
        }
    }

    #[derive(Clone)]
    struct TestTextureLoader;

    impl AssetLoader for TestTextureLoader {
        type Asset = TestTexture;
        type Settings = ();

        fn extensions(&self) -> &[&str] {
            &["png", "jpg"]
        }

        fn load<'a>(
            &'a self,
            bytes: &'a [u8],
            _settings: &'a Self::Settings,
            _context: &'a mut LoadContext,
        ) -> Result<Self::Asset, AssetLoadError> {
            // Simple fake loader that just reads size from first 8 bytes
            if bytes.len() < 8 {
                return Err(AssetLoadError::decode_failed("Not enough data"));
            }

            let width = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let height = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);

            Ok(TestTexture { width, height })
        }
    }

    // =============================================================================
    // AssetServer Tests
    // =============================================================================

    mod asset_server {
        use super::*;

        #[test]
        fn test_new() {
            let server = AssetServer::new();
            assert_eq!(server.asset_root(), Path::new("assets"));
            assert_eq!(server.total_loaded_count(), 0);
            assert_eq!(server.loader_count(), 0);
        }

        #[test]
        fn test_with_root() {
            let server = AssetServer::with_root("custom_assets");
            assert_eq!(server.asset_root(), Path::new("custom_assets"));
        }

        #[test]
        fn test_set_asset_root() {
            let mut server = AssetServer::new();
            server.set_asset_root("game_assets");
            assert_eq!(server.asset_root(), Path::new("game_assets"));
        }

        #[test]
        fn test_default() {
            let server = AssetServer::default();
            assert_eq!(server.asset_root(), Path::new("assets"));
        }

        #[test]
        fn test_debug() {
            let server = AssetServer::new();
            let debug_str = format!("{:?}", server);
            assert!(debug_str.contains("AssetServer"));
            assert!(debug_str.contains("assets"));
        }
    }

    mod loader_registration {
        use super::*;

        #[test]
        fn test_register_loader() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            assert_eq!(server.loader_count(), 1);
            assert!(server.has_loader_for_extension("test"));
            assert!(server.has_loader_for_type::<TestAsset>());
        }

        #[test]
        fn test_register_multiple_loaders() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);
            server.register_loader(TestTextureLoader);

            assert_eq!(server.loader_count(), 2);
            assert!(server.has_loader_for_extension("test"));
            assert!(server.has_loader_for_extension("png"));
            assert!(server.has_loader_for_extension("jpg"));
        }

        #[test]
        fn test_has_loader_for_extension() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            assert!(server.has_loader_for_extension("test"));
            assert!(server.has_loader_for_extension("TEST")); // Case-insensitive
            assert!(!server.has_loader_for_extension("unknown"));
        }

        #[test]
        fn test_has_loader_for_type() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            assert!(server.has_loader_for_type::<TestAsset>());
            assert!(!server.has_loader_for_type::<TestTexture>());
        }

        #[test]
        fn test_register_loader_with_settings() {
            let mut server = AssetServer::new();
            server.register_loader_with_settings(TestAssetLoader, ());

            assert!(server.has_loader_for_type::<TestAsset>());
        }
    }

    mod asset_operations {
        use super::*;
        use std::fs;
        use std::io::Write;
        use tempfile::TempDir;

        #[test]
        fn test_loaded_count() {
            let server = AssetServer::new();
            assert_eq!(server.loaded_count::<TestAsset>(), 0);
            assert_eq!(server.total_loaded_count(), 0);
        }

        #[test]
        fn test_registered_type_count() {
            let server = AssetServer::new();
            assert_eq!(server.registered_type_count(), 0);
        }

        #[test]
        fn test_handles_iterator() {
            let server = AssetServer::new();
            let handles: Vec<_> = server.handles::<TestAsset>().collect();
            assert_eq!(handles.len(), 0);
        }

        #[test]
        fn test_iter() {
            let server = AssetServer::new();
            let assets: Vec<_> = server.iter::<TestAsset>().collect();
            assert_eq!(assets.len(), 0);
        }

        #[test]
        fn test_clear_type() {
            let mut server = AssetServer::new();
            server.clear_type::<TestAsset>();
            // Should not panic
        }

        #[test]
        fn test_clear() {
            let mut server = AssetServer::new();
            server.clear();
            assert_eq!(server.total_loaded_count(), 0);
        }

        #[test]
        fn test_load_and_get() {
            // Create temp directory with test file
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();

            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"Hello, World!").unwrap();

            // Load asset
            let handle = server.load::<TestAsset>("test.test");

            // Should be loaded
            assert!(server.is_loaded(&handle));
            assert_eq!(server.loaded_count::<TestAsset>(), 1);

            // Get asset
            let asset = server.get(&handle);
            assert!(asset.is_some());
            assert_eq!(asset.unwrap().content, "Hello, World!");
        }

        #[test]
        fn test_load_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let mut server = AssetServer::with_root(temp_dir.path());
            server.register_loader(TestAssetLoader);

            let handle = server.load::<TestAsset>("nonexistent.test");

            // Handle is valid but asset is not loaded
            assert!(handle.is_valid());
            assert!(!server.is_loaded(&handle));

            // Check state is Failed
            let state = server.get_load_state(&handle);
            assert!(state.is_some());
            assert!(state.unwrap().is_failed());
        }

        #[test]
        fn test_load_unsupported_extension() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create file with unsupported extension
            let test_path = asset_root.join("test.unknown");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"data").unwrap();

            let handle = server.load::<TestAsset>("test.unknown");

            assert!(!server.is_loaded(&handle));
            let state = server.get_load_state(&handle);
            assert!(state.is_some());
            assert!(state.unwrap().is_failed());
        }

        #[test]
        fn test_load_deduplication() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"content").unwrap();

            // Load same asset twice
            let handle1 = server.load::<TestAsset>("test.test");
            let handle2 = server.load::<TestAsset>("test.test");

            // Should return same handle
            assert_eq!(handle1, handle2);
            assert_eq!(server.loaded_count::<TestAsset>(), 1);
        }

        #[test]
        fn test_unload() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create and load test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"data").unwrap();

            let handle = server.load::<TestAsset>("test.test");
            assert!(server.is_loaded(&handle));

            // Unload
            let asset = server.unload(&handle);
            assert!(asset.is_some());
            assert!(!server.is_loaded(&handle));
            assert_eq!(server.loaded_count::<TestAsset>(), 0);
        }

        #[test]
        fn test_get_mut() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);

            // Create and load test file
            let test_path = asset_root.join("test.test");
            let mut file = fs::File::create(&test_path).unwrap();
            file.write_all(b"original").unwrap();

            let handle = server.load::<TestAsset>("test.test");

            // Modify through get_mut
            if let Some(asset) = server.get_mut(&handle) {
                asset.content = "modified".to_string();
            }

            // Check modification
            assert_eq!(server.get(&handle).unwrap().content, "modified");
        }

        #[test]
        fn test_multiple_asset_types() {
            let temp_dir = TempDir::new().unwrap();
            let asset_root = temp_dir.path();
            let mut server = AssetServer::with_root(asset_root);
            server.register_loader(TestAssetLoader);
            server.register_loader(TestTextureLoader);

            // Create test files
            let test1 = asset_root.join("test.test");
            fs::File::create(&test1)
                .unwrap()
                .write_all(b"text")
                .unwrap();

            let test2 = asset_root.join("image.png");
            fs::File::create(&test2)
                .unwrap()
                .write_all(&[100, 0, 0, 0, 50, 0, 0, 0])
                .unwrap();

            // Load both types
            let handle1 = server.load::<TestAsset>("test.test");
            let handle2 = server.load::<TestTexture>("image.png");

            assert!(server.is_loaded(&handle1));
            assert!(server.is_loaded(&handle2));
            assert_eq!(server.loaded_count::<TestAsset>(), 1);
            assert_eq!(server.loaded_count::<TestTexture>(), 1);
            assert_eq!(server.total_loaded_count(), 2);
        }
    }

    mod load_from_bytes {
        use super::*;

        #[test]
        fn test_load_from_bytes_success() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            let handle = server.load_from_bytes::<TestAsset>("greeting.test", b"Hello from bytes");

            assert!(server.is_loaded(&handle));
            assert_eq!(server.get(&handle).unwrap().content, "Hello from bytes");
        }

        #[test]
        fn test_load_from_bytes_deduplication() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            let h1 = server.load_from_bytes::<TestAsset>("dup.test", b"first");
            let h2 = server.load_from_bytes::<TestAsset>("dup.test", b"second");

            assert_eq!(h1, h2);
            assert_eq!(server.loaded_count::<TestAsset>(), 1);
            assert_eq!(server.get(&h1).unwrap().content, "first");
        }

        #[test]
        fn test_load_from_bytes_unsupported_extension() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);

            let handle = server.load_from_bytes::<TestAsset>("data.unknown", b"data");

            assert!(!server.is_loaded(&handle));
            let state = server.get_load_state(&handle);
            assert!(state.unwrap().is_failed());
        }

        #[test]
        fn test_load_from_bytes_decode_error() {
            let mut server = AssetServer::new();
            server.register_loader(TestTextureLoader);

            // TestTextureLoader requires at least 8 bytes
            let handle = server.load_from_bytes::<TestTexture>("tiny.png", &[1, 2]);

            assert!(!server.is_loaded(&handle));
            assert!(server.get_load_state(&handle).unwrap().is_failed());
        }

        #[test]
        fn test_load_from_bytes_multiple_types() {
            let mut server = AssetServer::new();
            server.register_loader(TestAssetLoader);
            server.register_loader(TestTextureLoader);

            let h1 = server.load_from_bytes::<TestAsset>("hello.test", b"text data");
            let h2 =
                server.load_from_bytes::<TestTexture>("sprite.png", &[64, 0, 0, 0, 32, 0, 0, 0]);

            assert!(server.is_loaded(&h1));
            assert!(server.is_loaded(&h2));
            assert_eq!(server.get(&h1).unwrap().content, "text data");
            let tex = server.get(&h2).unwrap();
            assert_eq!(tex.width, 64);
            assert_eq!(tex.height, 32);
        }
    }

    mod thread_safety {
        use super::*;

        #[test]
        fn test_asset_server_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetServer>();
        }

        // Note: AssetServer is intentionally NOT Sync
        // It should be accessed from a single thread
    }
}
