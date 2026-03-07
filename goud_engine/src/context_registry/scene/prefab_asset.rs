//! Asset wrapper for [`PrefabData`], enabling loading via the asset system.
//!
//! The [`PrefabAssetLoader`] parses JSON files (`.prefab`, `.prefab.json`)
//! into [`PrefabAsset`] instances that the [`AssetServer`](crate::assets::AssetServer)
//! can cache and serve through handles.

use crate::assets::{Asset, AssetLoadError, AssetLoader, LoadContext};

use super::prefab::PrefabData;

// =============================================================================
// PrefabAsset
// =============================================================================

/// Asset wrapper for [`PrefabData`].
///
/// Wrapping `PrefabData` in a dedicated asset type lets the
/// [`AssetServer`](crate::assets::AssetServer) manage prefab lifetime
/// and caching like any other asset.
#[derive(Debug, Clone)]
pub struct PrefabAsset {
    /// The deserialized prefab data.
    pub data: PrefabData,
}

impl Asset for PrefabAsset {
    fn asset_type_name() -> &'static str {
        "PrefabAsset"
    }
}

// =============================================================================
// PrefabAssetLoader
// =============================================================================

/// Loads [`PrefabAsset`] instances from JSON files.
///
/// Supports the `.prefab` and `.prefab.json` extensions.
#[derive(Clone)]
pub struct PrefabAssetLoader;

impl AssetLoader for PrefabAssetLoader {
    type Asset = PrefabAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["prefab", "prefab.json"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let json = std::str::from_utf8(bytes)
            .map_err(|e| AssetLoadError::DecodeFailed(e.to_string()))?;
        let data: PrefabData = serde_json::from_str(json)
            .map_err(|e| AssetLoadError::DecodeFailed(e.to_string()))?;
        Ok(PrefabAsset { data })
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{AssetPath, LoadContext};
    use crate::context_registry::scene::data::EntityData;
    use crate::context_registry::scene::data::SerializedEntity;
    use std::collections::HashMap;

    #[test]
    fn test_prefab_asset_loader_roundtrip() {
        let prefab_data = PrefabData {
            name: "test_prefab".to_string(),
            entities: vec![EntityData {
                id: SerializedEntity {
                    index: 0,
                    generation: 1,
                },
                components: {
                    let mut map = HashMap::new();
                    map.insert(
                        "Name".to_string(),
                        serde_json::json!({"name": "hero"}),
                    );
                    map
                },
            }],
        };

        let json_bytes = serde_json::to_vec(&prefab_data).unwrap();

        let loader = PrefabAssetLoader;
        let path = AssetPath::from_string("test.prefab".to_string());
        let mut context = LoadContext::new(path);

        let asset = loader.load(&json_bytes, &(), &mut context).unwrap();

        assert_eq!(asset.data.name, "test_prefab");
        assert_eq!(asset.data.entities.len(), 1);
        assert!(asset.data.entities[0].components.contains_key("Name"));
    }

    #[test]
    fn test_prefab_asset_loader_invalid_json_errors() {
        let loader = PrefabAssetLoader;
        let path = AssetPath::from_string("bad.prefab".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(b"{{not json}}", &(), &mut context);
        assert!(result.is_err());
    }

    #[test]
    fn test_prefab_asset_loader_invalid_utf8_errors() {
        let loader = PrefabAssetLoader;
        let path = AssetPath::from_string("bad.prefab".to_string());
        let mut context = LoadContext::new(path);

        let result = loader.load(&[0xFF, 0xFE], &(), &mut context);
        assert!(result.is_err());
    }
}
