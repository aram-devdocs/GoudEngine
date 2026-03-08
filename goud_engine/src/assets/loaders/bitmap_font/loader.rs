//! [`BitmapFontLoader`] implementing the [`AssetLoader`] trait for BMFont files.

use crate::assets::{asset::Asset, AssetLoadError, AssetLoader, LoadContext};

use super::asset::BitmapFontAsset;
use super::parser::parse_bmfont;

/// BMFont asset loader for `.fnt` files (text format).
///
/// Parses BMFont text-format files and returns a [`BitmapFontAsset`]
/// containing character metrics, kerning, and a texture path reference.
///
/// # Supported Formats
/// - BMFont text format (.fnt)
#[derive(Clone, Debug, Default)]
pub struct BitmapFontLoader;

impl AssetLoader for BitmapFontLoader {
    type Asset = BitmapFontAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        BitmapFontAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let content = std::str::from_utf8(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("Invalid UTF-8: {e}")))?;

        let asset = parse_bmfont(content)
            .map_err(|e| AssetLoadError::decode_failed(format!("BMFont parse error: {e}")))?;

        // Declare dependency on the texture atlas.
        context.add_dependency(&asset.texture_path);

        Ok(asset)
    }
}
