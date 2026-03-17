//! [`SpriteSheetAsset`] descriptor for named atlas regions.

use std::collections::HashMap;

use crate::assets::{Asset, AssetType};
use crate::core::math::Rect;

/// Named region inside a sprite sheet or texture atlas.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpriteRegion {
    /// Pixel-space rectangle inside the backing texture.
    pub rect: Rect,
}

impl SpriteRegion {
    /// Creates a named region from pixel coordinates.
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            rect: Rect::new(x, y, width, height),
        }
    }
}

/// Asset-backed sprite-sheet descriptor.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SpriteSheetAsset {
    /// Optional human-readable name for the atlas.
    #[serde(default)]
    pub name: Option<String>,
    /// Path to the texture asset containing the atlas pixels.
    pub texture_path: String,
    /// Named frame regions in pixel coordinates.
    pub regions: HashMap<String, SpriteRegion>,
}

impl SpriteSheetAsset {
    /// Creates a sprite-sheet descriptor from a backing texture and region map.
    pub fn new(texture_path: impl Into<String>, regions: HashMap<String, SpriteRegion>) -> Self {
        Self {
            name: None,
            texture_path: texture_path.into(),
            regions,
        }
    }

    /// Returns the backing texture path.
    #[inline]
    pub fn texture_path(&self) -> &str {
        &self.texture_path
    }

    /// Returns a named region, if present.
    #[inline]
    pub fn region(&self, name: &str) -> Option<&SpriteRegion> {
        self.regions.get(name)
    }
}

impl Asset for SpriteSheetAsset {
    fn asset_type_name() -> &'static str {
        "SpriteSheet"
    }

    fn asset_type() -> AssetType {
        AssetType::Config
    }

    fn extensions() -> &'static [&'static str] {
        &["sheet.json", "atlas.json"]
    }
}
