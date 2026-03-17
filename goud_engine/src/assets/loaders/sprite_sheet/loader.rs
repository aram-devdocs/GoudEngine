//! [`SpriteSheetLoader`] for atlas descriptor JSON files.

use serde::Deserialize;
use std::collections::HashMap;

use crate::assets::{AssetLoadError, AssetLoader, LoadContext};
use crate::core::math::Rect;

use super::{SpriteRegion, SpriteSheetAsset};

#[derive(Debug, Clone, Deserialize)]
struct SpriteRegionDescriptor {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct SpriteSheetDescriptor {
    #[serde(default)]
    name: Option<String>,
    texture_path: String,
    regions: HashMap<String, SpriteRegionDescriptor>,
}

/// Loads `.sheet.json` and `.atlas.json` descriptors.
#[derive(Debug, Clone, Default)]
pub struct SpriteSheetLoader;

impl AssetLoader for SpriteSheetLoader {
    type Asset = SpriteSheetAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["sheet.json", "atlas.json"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let descriptor: SpriteSheetDescriptor = serde_json::from_slice(bytes).map_err(|e| {
            AssetLoadError::decode_failed(format!("Sprite sheet JSON parse error: {e}"))
        })?;

        if descriptor.texture_path.trim().is_empty() {
            return Err(AssetLoadError::decode_failed(
                "Sprite sheet must declare a texture_path".to_string(),
            ));
        }

        let mut regions = HashMap::with_capacity(descriptor.regions.len());
        for (name, region) in descriptor.regions {
            if region.width <= 0.0 || region.height <= 0.0 {
                return Err(AssetLoadError::decode_failed(format!(
                    "Sprite region '{name}' must have positive width and height"
                )));
            }
            regions.insert(
                name,
                SpriteRegion {
                    rect: Rect::new(region.x, region.y, region.width, region.height),
                },
            );
        }

        context.add_dependency(&descriptor.texture_path);

        Ok(SpriteSheetAsset {
            name: descriptor.name,
            texture_path: descriptor.texture_path,
            regions,
        })
    }
}
