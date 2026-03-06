//! Tests for async (non-blocking) asset loading.

use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetServer, AssetType, LoadContext};
use std::time::Duration;

// ---------------------------------------------------------------------------
// Test asset types (mirror the ones in tests.rs)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Test loaders
// ---------------------------------------------------------------------------

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
        if bytes.len() < 8 {
            return Err(AssetLoadError::decode_failed("Not enough data"));
        }
        let width = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let height = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        Ok(TestTexture { width, height })
    }
}

// ---------------------------------------------------------------------------
// Helper: poll process_loads until at least one result arrives or timeout
// ---------------------------------------------------------------------------

fn poll_until_processed(server: &mut AssetServer, max_iters: usize) -> usize {
    let mut total = 0;
    for _ in 0..max_iters {
        let n = server.process_loads();
        total += n;
        if total > 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    total
}

/// Poll until the expected number of results arrive.
fn poll_until_count(server: &mut AssetServer, expected: usize, max_iters: usize) -> usize {
    let mut total = 0;
    for _ in 0..max_iters {
        total += server.process_loads();
        if total >= expected {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    total
}

// =============================================================================
// Async loading tests
// =============================================================================

#[cfg(feature = "native")]
mod async_loading {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_load_async_returns_handle_in_loading_state() {
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        let mut server = AssetServer::with_root(asset_root);
        server.register_loader(TestAssetLoader);

        // Create test file
        let test_path = asset_root.join("hello.test");
        fs::File::create(&test_path)
            .unwrap()
            .write_all(b"async hello")
            .unwrap();

        let handle = server.load_async::<TestAsset>("hello.test");

        // Handle should be valid
        assert!(handle.is_valid());

        // State should be Loading (not yet processed)
        let state = server.get_load_state(&handle);
        assert!(state.is_some());
        assert!(
            state.as_ref().unwrap().is_loading(),
            "Expected Loading state, got {:?}",
            state
        );
    }

    #[test]
    fn test_load_async_becomes_loaded_after_process() {
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        let mut server = AssetServer::with_root(asset_root);
        server.register_loader(TestAssetLoader);

        let test_path = asset_root.join("data.test");
        fs::File::create(&test_path)
            .unwrap()
            .write_all(b"async data")
            .unwrap();

        let handle = server.load_async::<TestAsset>("data.test");

        // Poll until processed
        let processed = poll_until_processed(&mut server, 50);
        assert!(processed > 0, "Expected at least one load to be processed");

        // Should now be loaded
        assert!(server.is_loaded(&handle));
        let asset = server.get(&handle).unwrap();
        assert_eq!(asset.content, "async data");
    }

    #[test]
    fn test_load_async_failed_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut server = AssetServer::with_root(temp_dir.path());
        server.register_loader(TestAssetLoader);

        let handle = server.load_async::<TestAsset>("nonexistent.test");

        // Poll until result arrives
        poll_until_processed(&mut server, 50);

        // Should be failed
        assert!(!server.is_loaded(&handle));
        let state = server.get_load_state(&handle);
        assert!(state.is_some());
        assert!(
            state.as_ref().unwrap().is_failed(),
            "Expected Failed state, got {:?}",
            state
        );
    }

    #[test]
    fn test_load_async_deduplication() {
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        let mut server = AssetServer::with_root(asset_root);
        server.register_loader(TestAssetLoader);

        let test_path = asset_root.join("dup.test");
        fs::File::create(&test_path)
            .unwrap()
            .write_all(b"dedup")
            .unwrap();

        let handle1 = server.load_async::<TestAsset>("dup.test");
        let handle2 = server.load_async::<TestAsset>("dup.test");

        // Same path should return the same handle
        assert_eq!(handle1, handle2);
    }

    #[test]
    fn test_load_async_no_loader_fails_immediately() {
        let temp_dir = TempDir::new().unwrap();
        let mut server = AssetServer::with_root(temp_dir.path());
        // Do NOT register any loaders

        let handle = server.load_async::<TestAsset>("data.test");

        // Should be failed immediately (no process_loads needed)
        let state = server.get_load_state(&handle);
        assert!(state.is_some());
        assert!(
            state.as_ref().unwrap().is_failed(),
            "Expected immediate Failed state for missing loader, got {:?}",
            state
        );
    }

    #[test]
    fn test_process_loads_returns_count() {
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        let mut server = AssetServer::with_root(asset_root);
        server.register_loader(TestAssetLoader);

        // Create two files
        fs::File::create(asset_root.join("a.test"))
            .unwrap()
            .write_all(b"aaa")
            .unwrap();
        fs::File::create(asset_root.join("b.test"))
            .unwrap()
            .write_all(b"bbb")
            .unwrap();

        let _h1 = server.load_async::<TestAsset>("a.test");
        let _h2 = server.load_async::<TestAsset>("b.test");

        // Poll until both are processed
        let total = poll_until_count(&mut server, 2, 50);
        assert_eq!(total, 2);
    }

    #[test]
    fn test_load_async_multiple_asset_types() {
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        let mut server = AssetServer::with_root(asset_root);
        server.register_loader(TestAssetLoader);
        server.register_loader(TestTextureLoader);

        // Create test files
        fs::File::create(asset_root.join("text.test"))
            .unwrap()
            .write_all(b"hello text")
            .unwrap();
        fs::File::create(asset_root.join("image.png"))
            .unwrap()
            .write_all(&[128, 0, 0, 0, 64, 0, 0, 0])
            .unwrap();

        let h_text = server.load_async::<TestAsset>("text.test");
        let h_tex = server.load_async::<TestTexture>("image.png");

        // Poll until both complete
        poll_until_count(&mut server, 2, 50);

        // Verify both loaded correctly
        assert!(server.is_loaded(&h_text));
        assert!(server.is_loaded(&h_tex));

        assert_eq!(server.get(&h_text).unwrap().content, "hello text");
        let tex = server.get(&h_tex).unwrap();
        assert_eq!(tex.width, 128);
        assert_eq!(tex.height, 64);
    }

    #[test]
    fn test_main_thread_not_blocked() {
        let temp_dir = TempDir::new().unwrap();
        let asset_root = temp_dir.path();

        let mut server = AssetServer::with_root(asset_root);
        server.register_loader(TestAssetLoader);

        let test_path = asset_root.join("slow.test");
        fs::File::create(&test_path)
            .unwrap()
            .write_all(b"data")
            .unwrap();

        // load_async returns immediately (non-blocking)
        let handle = server.load_async::<TestAsset>("slow.test");

        // We can still interact with the server before calling process_loads
        assert!(handle.is_valid());
        assert_eq!(server.loaded_count::<TestAsset>(), 1); // 1 reserved entry

        // Now process and verify
        poll_until_processed(&mut server, 50);
        assert!(server.is_loaded(&handle));
    }
}
