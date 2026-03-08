//! Pipeline and advanced asset server tests.
//!
//! This module tests asset loading pipelines, archive operations, reference counting,
//! and fallback substitution.

use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetServer, AssetType, LoadContext};

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

mod archive_round_trip {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_package_and_load_via_archive() {
        // Arrange: create a temp directory with test files
        let input_dir = TempDir::new().unwrap();
        let input_path = input_dir.path();

        let mut f1 = fs::File::create(input_path.join("greeting.test")).unwrap();
        f1.write_all(b"Hello from archive").unwrap();

        fs::create_dir_all(input_path.join("sub")).unwrap();
        let mut f2 = fs::File::create(input_path.join("sub/nested.test")).unwrap();
        f2.write_all(b"Nested content").unwrap();

        // Act: package into an archive
        let archive_dir = TempDir::new().unwrap();
        let archive_path = archive_dir.path().join("test.goud");
        crate::assets::packager::package_directory(input_path, &archive_path).unwrap();

        // Read archive bytes and create server from them
        let archive_bytes = fs::read(&archive_path).unwrap();
        let mut server = AssetServer::with_archive(archive_bytes).unwrap();
        server.register_loader(TestAssetLoader);

        // Load assets from the archive
        let h1 = server.load::<TestAsset>("greeting.test");
        let h2 = server.load::<TestAsset>("sub/nested.test");

        // Assert: loaded data matches originals
        assert!(server.is_loaded(&h1));
        assert!(server.is_loaded(&h2));
        assert_eq!(server.get(&h1).unwrap().content, "Hello from archive");
        assert_eq!(server.get(&h2).unwrap().content, "Nested content");
    }
}

mod ref_count_lifecycle {
    use super::*;

    #[test]
    fn test_retain_release_deferred_unload() {
        let mut server = AssetServer::new();
        server.register_loader(TestAssetLoader);

        // Insert an asset via load_from_bytes (starts with ref count 1)
        let handle = server.load_from_bytes::<TestAsset>("rc.test", b"ref counted");
        assert!(server.is_loaded(&handle));
        assert_eq!(server.ref_count(&handle), 1);

        // retain increments ref count
        let count = server.retain(&handle);
        assert_eq!(count, Some(2));
        assert_eq!(server.ref_count(&handle), 2);

        // first release decrements to 1
        let count = server.release(&handle);
        assert_eq!(count, Some(1));
        assert_eq!(server.ref_count(&handle), 1);

        // second release decrements to 0 and queues deferred unload
        let count = server.release(&handle);
        assert_eq!(count, Some(0));

        // Asset is still accessible before processing deferred unloads
        assert!(server.get(&handle).is_some());

        // Process deferred unloads removes the asset
        server.process_deferred_unloads();
        assert!(server.get(&handle).is_none());
    }
}

mod fallback_substitution {
    use super::*;

    #[test]
    fn test_fallback_on_missing_asset() {
        let mut server = AssetServer::new();
        server.register_loader(TestAssetLoader);

        // Register a fallback for TestAsset
        let fallback = TestAsset {
            content: "default fallback".to_string(),
        };
        server.register_fallback(fallback.clone());

        // Load a nonexistent path -- should trigger fallback
        let handle = server.load::<TestAsset>("does_not_exist.test");

        // Handle should be valid and asset should be loaded (not failed)
        assert!(handle.is_valid());
        assert!(server.is_loaded(&handle));

        // Entry should be marked as fallback
        let entry = server.storage.get_entry(&handle);
        assert!(entry.is_some());
        assert!(entry.unwrap().is_fallback());

        // Asset data matches the registered fallback
        let asset = server.get(&handle).unwrap();
        assert_eq!(asset.content, "default fallback");
    }
}
