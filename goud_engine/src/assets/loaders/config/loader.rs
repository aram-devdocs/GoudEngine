//! [`ConfigLoader`] -- parses configuration files into [`ConfigAsset`].

use crate::assets::{Asset, AssetLoadError, AssetLoader, LoadContext};

use super::{asset::ConfigAsset, format::ConfigFormat};

/// Asset loader for configuration files.
///
/// Supports JSON files natively. TOML support is available when the
/// `native` feature is enabled.
///
/// # Example
///
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::config::{ConfigLoader, ConfigAsset}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(ConfigLoader::default());
///
/// let handle = server.load::<ConfigAsset>("config/game.json");
/// ```
#[derive(Debug, Clone, Default)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// Creates a new config loader.
    pub fn new() -> Self {
        Self
    }

    /// Parses JSON bytes into a config asset.
    fn load_json(bytes: &[u8]) -> Result<ConfigAsset, AssetLoadError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("JSON parse error: {e}")))?;
        Ok(ConfigAsset::new(value, ConfigFormat::Json))
    }

    /// Parses TOML bytes into a config asset.
    ///
    /// Converts the TOML data to a `serde_json::Value` for uniform access.
    #[cfg(feature = "native")]
    fn load_toml(bytes: &[u8]) -> Result<ConfigAsset, AssetLoadError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|e| AssetLoadError::decode_failed(format!("TOML is not valid UTF-8: {e}")))?;
        let toml_value: toml::Value = text
            .parse()
            .map_err(|e| AssetLoadError::decode_failed(format!("TOML parse error: {e}")))?;
        let json_value = toml_value_to_json(toml_value);
        Ok(ConfigAsset::new(json_value, ConfigFormat::Toml))
    }
}

/// Converts a [`toml::Value`] into a [`serde_json::Value`].
///
/// This preserves the structure while normalizing the representation
/// so that all config data can be accessed through a single API.
#[cfg(feature = "native")]
fn toml_value_to_json(value: toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(s) => serde_json::Value::String(s),
        toml::Value::Integer(i) => serde_json::json!(i),
        toml::Value::Float(f) => serde_json::json!(f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(toml_value_to_json).collect())
        }
        toml::Value::Table(table) => {
            let map = table
                .into_iter()
                .map(|(k, v)| (k, toml_value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
    }
}

impl AssetLoader for ConfigLoader {
    type Asset = ConfigAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        ConfigAsset::extensions()
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let format = context.extension().and_then(ConfigFormat::from_extension);

        match format {
            Some(ConfigFormat::Json) => Self::load_json(bytes),
            #[cfg(feature = "native")]
            Some(ConfigFormat::Toml) => Self::load_toml(bytes),
            #[cfg(not(feature = "native"))]
            Some(ConfigFormat::Toml) => Err(AssetLoadError::unsupported_format("toml")),
            None => {
                // Try JSON as default fallback
                Self::load_json(bytes)
            }
        }
    }
}
