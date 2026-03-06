//! Test asset types and loaders used in unit tests and doctests.

use crate::assets::Asset;

use super::{AssetLoadError, AssetLoader, LoadContext};

/// A simple text asset for testing purposes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct TextAsset {
    /// The text content of the asset.
    pub content: String,
}

impl Asset for TextAsset {}

/// A simple binary asset for testing purposes.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryAsset {
    /// The raw binary data of the asset.
    pub data: Vec<u8>,
}

impl Asset for BinaryAsset {}

/// A loader for plain text assets (`.txt`, `.text` extensions).
#[allow(dead_code)]
#[derive(Clone)]
pub struct TextAssetLoader;

impl AssetLoader for TextAssetLoader {
    type Asset = TextAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["txt", "text"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let content = String::from_utf8(bytes.to_vec())
            .map_err(|e| AssetLoadError::decode_failed(e.to_string()))?;
        Ok(TextAsset { content })
    }
}

/// A loader for raw binary assets (`.bin`, `.dat` extensions).
#[allow(dead_code)]
#[derive(Clone)]
pub struct BinaryAssetLoader;

impl AssetLoader for BinaryAssetLoader {
    type Asset = BinaryAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["bin", "dat"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        Ok(BinaryAsset {
            data: bytes.to_vec(),
        })
    }
}

/// Settings for the settings-aware loader, used to demonstrate loader configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct LoaderSettings {
    /// Maximum allowed asset size in bytes.
    pub max_size: usize,
}

/// A loader that enforces a maximum asset size via [`LoaderSettings`].
#[allow(dead_code)]
#[derive(Clone)]
pub struct SettingsLoader;

impl AssetLoader for SettingsLoader {
    type Asset = BinaryAsset;
    type Settings = LoaderSettings;

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        settings: &'a Self::Settings,
        _context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        if bytes.len() > settings.max_size {
            return Err(AssetLoadError::custom(format!(
                "Asset too large: {} > {}",
                bytes.len(),
                settings.max_size
            )));
        }
        Ok(BinaryAsset {
            data: bytes.to_vec(),
        })
    }
}
