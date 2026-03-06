//! Runtime metadata about an asset type.

use super::asset_id::AssetId;
use super::asset_type::AssetType;
use super::trait_def::Asset;
use std::fmt;

/// Runtime metadata about an asset type.
///
/// Contains information about an asset type that can be queried at runtime.
/// Useful for debugging, logging, and asset management UI.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetInfo, AssetType};
///
/// struct Texture {
///     width: u32,
///     height: u32,
///     data: Vec<u8>,
/// }
///
/// impl Asset for Texture {
///     fn asset_type_name() -> &'static str {
///         "Texture"
///     }
///
///     fn asset_type() -> AssetType {
///         AssetType::Texture
///     }
///
///     fn extensions() -> &'static [&'static str] {
///         &["png", "jpg"]
///     }
/// }
///
/// let info = AssetInfo::of::<Texture>();
/// assert_eq!(info.name, "Texture");
/// assert_eq!(info.asset_type, AssetType::Texture);
/// assert!(info.extensions.contains(&"png"));
/// ```
#[derive(Debug, Clone)]
pub struct AssetInfo {
    /// Unique identifier for this asset type.
    pub id: AssetId,

    /// Human-readable type name.
    pub name: &'static str,

    /// Size of the asset type in bytes.
    pub size: usize,

    /// Alignment of the asset type in bytes.
    pub align: usize,

    /// Asset category.
    pub asset_type: AssetType,

    /// Supported file extensions.
    pub extensions: &'static [&'static str],
}

impl AssetInfo {
    /// Creates `AssetInfo` for a specific asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetInfo};
    ///
    /// struct MyAsset {
    ///     value: i32,
    /// }
    /// impl Asset for MyAsset {}
    ///
    /// let info = AssetInfo::of::<MyAsset>();
    /// assert_eq!(info.size, std::mem::size_of::<MyAsset>());
    /// ```
    pub fn of<T: Asset>() -> Self {
        Self {
            id: AssetId::of::<T>(),
            name: T::asset_type_name(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            asset_type: T::asset_type(),
            extensions: T::extensions(),
        }
    }
}

impl fmt::Display for AssetInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AssetInfo {{ name: \"{}\", type: {}, size: {}, extensions: {:?} }}",
            self.name, self.asset_type, self.size, self.extensions
        )
    }
}
