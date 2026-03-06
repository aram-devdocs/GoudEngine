//! Core [`Asset`] trait definition.
//!
//! This module defines the [`Asset`] trait that all loadable assets must implement.

use super::asset_type::AssetType;

/// Marker trait for types that can be managed by the asset system.
///
/// The `Asset` trait indicates that a type represents loadable content that
/// can be cached, reference-counted, and accessed through handles.
///
/// # Requirements
///
/// Assets must be:
/// - `Send + Sync`: For parallel loading and multi-threaded access
/// - `'static`: For type erasure in storage
///
/// Unlike [`Component`](crate::ecs::Component), assets provide additional
/// metadata through associated methods:
///
/// - `asset_type_name()`: Human-readable type name for debugging
/// - `asset_type()`: Category classification for the asset
///
/// # Implementing Asset
///
/// ```
/// use goud_engine::assets::{Asset, AssetType};
///
/// struct AudioClip {
///     samples: Vec<f32>,
///     sample_rate: u32,
///     channels: u8,
/// }
///
/// impl Asset for AudioClip {
///     fn asset_type_name() -> &'static str {
///         "AudioClip"
///     }
///
///     fn asset_type() -> AssetType {
///         AssetType::Audio
///     }
/// }
/// ```
///
/// # Thread Safety
///
/// Assets must be thread-safe because:
/// - Asset loaders may run on background threads
/// - Multiple systems may access the same asset concurrently
/// - The asset cache is shared across the engine
///
/// # FFI Considerations
///
/// When exposing assets through FFI:
/// - Use handles, never raw asset pointers
/// - Asset data should be copied across the FFI boundary
/// - Asset modification should go through engine functions
pub trait Asset: Send + Sync + 'static {
    /// Returns the human-readable name of this asset type.
    ///
    /// This is used for debugging, logging, and error messages.
    /// Should be a short, descriptive name like "Texture" or "AudioClip".
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::Asset;
    ///
    /// struct Shader { /* ... */ }
    ///
    /// impl Asset for Shader {
    ///     fn asset_type_name() -> &'static str {
    ///         "Shader"
    ///     }
    /// }
    ///
    /// assert_eq!(Shader::asset_type_name(), "Shader");
    /// ```
    fn asset_type_name() -> &'static str
    where
        Self: Sized,
    {
        // Default implementation uses Rust's type name
        std::any::type_name::<Self>()
    }

    /// Returns the category of this asset type.
    ///
    /// Asset categories help the engine organize and manage assets:
    /// - GPU resources (textures, meshes) may have special upload handling
    /// - Audio assets may be streamed vs fully loaded
    /// - Custom assets have no special handling
    ///
    /// # Default
    ///
    /// Returns `AssetType::Custom` by default. Override for built-in types.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetType};
    ///
    /// struct MyTexture { /* ... */ }
    ///
    /// impl Asset for MyTexture {
    ///     fn asset_type() -> AssetType {
    ///         AssetType::Texture
    ///     }
    /// }
    ///
    /// assert_eq!(MyTexture::asset_type(), AssetType::Texture);
    /// ```
    fn asset_type() -> AssetType
    where
        Self: Sized,
    {
        AssetType::Custom
    }

    /// Returns the file extensions typically associated with this asset type.
    ///
    /// Used by asset loaders to determine which loader handles which files.
    /// Returns an empty slice by default.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::Asset;
    ///
    /// struct Texture { /* ... */ }
    ///
    /// impl Asset for Texture {
    ///     fn extensions() -> &'static [&'static str] {
    ///         &["png", "jpg", "jpeg", "bmp", "tga"]
    ///     }
    /// }
    ///
    /// assert!(Texture::extensions().contains(&"png"));
    /// ```
    fn extensions() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &[]
    }
}
