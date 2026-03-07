//! Integration tests for asset dependency tracking via AssetServer.

use crate::assets::{Asset, AssetLoadError, AssetLoader, AssetServer, LoadContext};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

// Re-use TestAssetLoader from main tests
#[derive(Debug, Clone, PartialEq)]
struct TestAsset {
    content: String,
}
impl Asset for TestAsset {
    fn asset_type_name() -> &'static str {
        "TestAsset"
    }
}

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

/// Loader that declares a dependency during load.
#[derive(Clone)]
struct DependentAssetLoader;

#[derive(Debug, Clone, PartialEq)]
struct DependentAsset {
    content: String,
}
impl Asset for DependentAsset {
    fn asset_type_name() -> &'static str {
        "DependentAsset"
    }
}

impl AssetLoader for DependentAssetLoader {
    type Asset = DependentAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["dep"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
        context.add_dependency("base.test");
        Ok(DependentAsset { content })
    }
}

#[test]
fn test_load_records_dependencies_in_graph() {
    let temp_dir = TempDir::new().unwrap();
    let asset_root = temp_dir.path();
    let mut server = AssetServer::with_root(asset_root);
    server.register_loader(TestAssetLoader);
    server.register_loader(DependentAssetLoader);

    let base_file = asset_root.join("base.test");
    fs::File::create(&base_file)
        .unwrap()
        .write_all(b"base content")
        .unwrap();
    let dep_file = asset_root.join("child.dep");
    fs::File::create(&dep_file)
        .unwrap()
        .write_all(b"child content")
        .unwrap();

    let _base_handle = server.load::<TestAsset>("base.test");
    let dep_handle = server.load::<DependentAsset>("child.dep");
    assert!(server.is_loaded(&dep_handle));

    let cascade = server.get_cascade_order("base.test");
    assert!(
        cascade.contains(&"child.dep".to_string()),
        "child.dep should appear in cascade order when base.test changes"
    );
}

#[test]
fn test_dependency_graph_accessible() {
    let server = AssetServer::new();
    assert_eq!(server.dependency_graph().asset_count(), 0);
}

#[test]
fn test_manual_dependency_and_cascade() {
    let mut server = AssetServer::new();

    server
        .dependency_graph_mut()
        .add_dependency("material.mat.json", "shader.vert")
        .unwrap();
    server
        .dependency_graph_mut()
        .add_dependency("material.mat.json", "texture.png")
        .unwrap();

    let cascade = server.get_cascade_order("shader.vert");
    assert!(cascade.contains(&"material.mat.json".to_string()));

    let cascade = server.get_cascade_order("texture.png");
    assert!(cascade.contains(&"material.mat.json".to_string()));
}

#[test]
fn test_unload_cleans_dependency_graph() {
    let temp_dir = TempDir::new().unwrap();
    let asset_root = temp_dir.path();
    let mut server = AssetServer::with_root(asset_root);
    server.register_loader(DependentAssetLoader);

    let dep_file = asset_root.join("child.dep");
    fs::File::create(&dep_file)
        .unwrap()
        .write_all(b"data")
        .unwrap();

    let handle = server.load::<DependentAsset>("child.dep");
    assert!(server.is_loaded(&handle));

    let cascade = server.get_cascade_order("base.test");
    assert!(cascade.contains(&"child.dep".to_string()));

    server.unload(&handle);
    let cascade = server.get_cascade_order("base.test");
    assert!(
        !cascade.contains(&"child.dep".to_string()),
        "child.dep should be removed from cascade after unload"
    );
}

/// Tests that compound extensions (e.g. "mat.json") are properly resolved
/// when loading via AssetServer.
#[test]
fn test_compound_extension_loader_lookup() {
    use crate::assets::loaders::config::{ConfigAsset, ConfigLoader};

    let mut server = AssetServer::new();
    server.register_loader(ConfigLoader::default());

    // load_from_bytes with a .json extension should work via ConfigLoader
    let handle = server.load_from_bytes::<ConfigAsset>("settings.json", br#"{"key": "value"}"#);
    assert!(
        server.is_loaded(&handle),
        "ConfigLoader should handle .json files"
    );
}
