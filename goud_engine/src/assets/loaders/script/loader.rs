//! [`ScriptLoader`] -- reads script files into [`ScriptAsset`].

use crate::assets::{Asset, AssetLoadError, AssetLoader, LoadContext};

use super::asset::ScriptAsset;

/// Asset loader for script files (currently Lua).
///
/// Reads the raw bytes as UTF-8 text and stores them in a [`ScriptAsset`].
#[derive(Debug, Clone, Default)]
pub struct ScriptLoader;

impl AssetLoader for ScriptLoader {
    type Asset = ScriptAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        ScriptAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let source = std::str::from_utf8(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("Script is not valid UTF-8: {e}")))?
            .to_string();

        let file_name = context
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown.lua".to_string());

        Ok(ScriptAsset { source, file_name })
    }
}
