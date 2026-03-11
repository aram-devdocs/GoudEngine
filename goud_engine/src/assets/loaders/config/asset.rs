//! [`ConfigAsset`] -- parsed configuration data.

use serde::de::DeserializeOwned;

use crate::assets::{Asset, AssetType};

use super::format::ConfigFormat;

/// A loaded configuration asset containing parsed data.
///
/// `ConfigAsset` stores configuration data as a [`serde_json::Value`],
/// which can represent any JSON-compatible structure. TOML files are
/// converted to this same representation during loading.
///
/// Use [`deserialize`](ConfigAsset::deserialize) to convert the data
/// into a strongly-typed struct.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{AssetLoader, AssetPath, LoadContext};
/// use goud_engine::assets::loaders::config::{ConfigFormat, ConfigLoader};
///
/// let loader = ConfigLoader::new();
/// let path = AssetPath::from_string("config/game.json".to_string());
/// let mut context = LoadContext::new(path);
/// let asset = loader
///     .load(br#"{"name":"player","speed":5.0}"#, &(), &mut context)
///     .unwrap();
///
/// assert_eq!(asset.format(), ConfigFormat::Json);
/// assert_eq!(asset.get("name").unwrap(), "player");
/// ```
#[derive(Debug, Clone)]
pub struct ConfigAsset {
    /// The parsed configuration data.
    data: serde_json::Value,
    /// The format the config was loaded from.
    format: ConfigFormat,
}

impl ConfigAsset {
    /// Creates a new config asset from parsed data.
    pub fn new(data: serde_json::Value, format: ConfigFormat) -> Self {
        Self { data, format }
    }

    /// Returns the format this config was loaded from.
    #[inline]
    pub fn format(&self) -> ConfigFormat {
        self.format
    }

    /// Returns a reference to the underlying JSON value.
    #[inline]
    pub fn data(&self) -> &serde_json::Value {
        &self.data
    }

    /// Consumes the asset and returns the underlying JSON value.
    #[inline]
    pub fn into_data(self) -> serde_json::Value {
        self.data
    }

    /// Deserializes the config data into a strongly-typed struct.
    ///
    /// # Errors
    ///
    /// Returns an error if the data does not match the expected type `T`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Deserialize;
    /// use goud_engine::assets::{AssetLoader, AssetPath, LoadContext};
    /// use goud_engine::assets::loaders::config::ConfigLoader;
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct GameConfig {
    ///     title: String,
    ///     width: u32,
    /// }
    ///
    /// let loader = ConfigLoader::new();
    /// let path = AssetPath::from_string("config/game.json".to_string());
    /// let mut context = LoadContext::new(path);
    /// let asset = loader
    ///     .load(br#"{"title":"My Game","width":800}"#, &(), &mut context)
    ///     .unwrap();
    ///
    /// let config: GameConfig = asset.deserialize().unwrap();
    /// assert_eq!(config.title, "My Game");
    /// assert_eq!(config.width, 800);
    /// ```
    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.data.clone())
    }

    /// Returns a value at the given key path (top-level only).
    ///
    /// Returns `None` if the key does not exist or the data is not an object.
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    /// Returns true if the config data is an object (map).
    #[inline]
    pub fn is_object(&self) -> bool {
        self.data.is_object()
    }

    /// Returns true if the config data is an array.
    #[inline]
    pub fn is_array(&self) -> bool {
        self.data.is_array()
    }
}

impl Asset for ConfigAsset {
    fn asset_type_name() -> &'static str {
        "Config"
    }

    fn asset_type() -> AssetType {
        AssetType::Config
    }

    fn extensions() -> &'static [&'static str] {
        &["json", "toml"]
    }
}
