//! Core asset trait and type identification.
//!
//! This module defines the [`Asset`] trait that all loadable assets must implement,
//! along with supporting types for asset identification and metadata.
//!
//! # Design Philosophy
//!
//! Assets in GoudEngine are designed to be:
//!
//! 1. **Type-Safe**: Each asset type has a unique [`AssetId`] for runtime type checking
//! 2. **Thread-Safe**: Assets must be `Send + Sync` for parallel loading
//! 3. **Self-Describing**: Assets can report their type name and metadata
//! 4. **FFI-Compatible**: All IDs and enums are designed for cross-language use
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetId, AssetType};
//!
//! // Custom texture asset
//! struct Texture {
//!     width: u32,
//!     height: u32,
//!     gpu_id: u32,
//! }
//!
//! impl Asset for Texture {
//!     fn asset_type_name() -> &'static str {
//!         "Texture"
//!     }
//!
//!     fn asset_type() -> AssetType {
//!         AssetType::Texture
//!     }
//! }
//!
//! // Get runtime type info
//! let id = AssetId::of::<Texture>();
//! println!("Texture type ID: {:?}", id);
//! ```

use std::any::TypeId;
use std::fmt;
use std::hash::Hash;

// =============================================================================
// Asset Trait
// =============================================================================

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

// =============================================================================
// AssetId
// =============================================================================

/// Unique identifier for an asset type at runtime.
///
/// `AssetId` wraps a `TypeId` to identify asset types. This is used internally
/// for asset storage, loader registration, and type-safe access.
///
/// Unlike [`Handle`](crate::core::handle::Handle), which identifies a specific
/// asset instance, `AssetId` identifies an asset *type*.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetId};
///
/// struct Texture { /* ... */ }
/// impl Asset for Texture {}
///
/// struct Mesh { /* ... */ }
/// impl Asset for Mesh {}
///
/// let tex_id = AssetId::of::<Texture>();
/// let mesh_id = AssetId::of::<Mesh>();
///
/// assert_ne!(tex_id, mesh_id);
/// assert_eq!(tex_id, AssetId::of::<Texture>());
/// ```
///
/// # FFI Considerations
///
/// `AssetId` is NOT FFI-safe (contains `TypeId`). For FFI, use the
/// `AssetType` enum or string-based type names.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetId(TypeId);

impl AssetId {
    /// Returns the `AssetId` for a specific asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetId};
    ///
    /// struct MyAsset;
    /// impl Asset for MyAsset {}
    ///
    /// let id = AssetId::of::<MyAsset>();
    /// ```
    #[inline]
    pub fn of<T: Asset>() -> Self {
        Self(TypeId::of::<T>())
    }

    /// Returns the `AssetId` for any `'static` type.
    ///
    /// This is useful for internal operations where the type may not
    /// implement `Asset` yet (e.g., during registration).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetId;
    ///
    /// let id = AssetId::of_raw::<String>();
    /// ```
    #[inline]
    pub fn of_raw<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    /// Returns the underlying `TypeId`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetId};
    /// use std::any::TypeId;
    ///
    /// struct MyAsset;
    /// impl Asset for MyAsset {}
    ///
    /// let id = AssetId::of::<MyAsset>();
    /// assert_eq!(id.type_id(), TypeId::of::<MyAsset>());
    /// ```
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.0
    }
}

impl fmt::Debug for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId({:?})", self.0)
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetId({:?})", self.0)
    }
}

// =============================================================================
// AssetType
// =============================================================================

/// Categories of assets for engine-level organization.
///
/// Asset types help the engine determine how to handle different assets:
/// - GPU resources may need special upload handling
/// - Audio may be streamed or fully loaded
/// - Data assets may be parsed into structured data
///
/// # FFI Safety
///
/// This enum is `#[repr(u8)]` for stable, FFI-compatible representation.
/// Each variant can be converted to/from an integer for cross-language use.
///
/// # Example
///
/// ```
/// use goud_engine::assets::AssetType;
///
/// let asset_type = AssetType::Texture;
///
/// // FFI: convert to integer
/// let value: u8 = asset_type.into();
/// assert_eq!(value, 1);
///
/// // FFI: convert from integer
/// let recovered = AssetType::try_from(value).unwrap();
/// assert_eq!(recovered, AssetType::Texture);
/// ```
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AssetType {
    /// Custom asset type not in any predefined category.
    #[default]
    Custom = 0,

    /// Image data (PNG, JPG, etc.) uploaded to GPU.
    Texture = 1,

    /// Audio data (WAV, OGG, MP3, etc.).
    Audio = 2,

    /// 3D mesh data (vertices, indices, etc.).
    Mesh = 3,

    /// Shader source or compiled shader program.
    Shader = 4,

    /// Font data (TTF, OTF, etc.).
    Font = 5,

    /// Material definition (properties, textures, shader refs).
    Material = 6,

    /// Animation data (clips, keyframes, etc.).
    Animation = 7,

    /// Tiled map data (TMX, etc.).
    TiledMap = 8,

    /// Prefab/scene definition.
    Prefab = 9,

    /// Configuration data (JSON, TOML, etc.).
    Config = 10,

    /// Generic binary data.
    Binary = 11,

    /// Text content.
    Text = 12,
}

impl AssetType {
    /// Returns all asset type variants.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// let types = AssetType::all();
    /// assert!(types.contains(&AssetType::Texture));
    /// assert_eq!(types.len(), 13);
    /// ```
    pub const fn all() -> &'static [AssetType] {
        &[
            AssetType::Custom,
            AssetType::Texture,
            AssetType::Audio,
            AssetType::Mesh,
            AssetType::Shader,
            AssetType::Font,
            AssetType::Material,
            AssetType::Animation,
            AssetType::TiledMap,
            AssetType::Prefab,
            AssetType::Config,
            AssetType::Binary,
            AssetType::Text,
        ]
    }

    /// Returns the number of asset type variants.
    pub const fn count() -> usize {
        13
    }

    /// Returns true if this is a GPU-uploadable asset type.
    ///
    /// GPU assets require special handling for upload to graphics memory.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// assert!(AssetType::Texture.is_gpu_asset());
    /// assert!(AssetType::Mesh.is_gpu_asset());
    /// assert!(!AssetType::Audio.is_gpu_asset());
    /// ```
    #[inline]
    pub const fn is_gpu_asset(&self) -> bool {
        matches!(
            self,
            AssetType::Texture | AssetType::Mesh | AssetType::Shader | AssetType::Font
        )
    }

    /// Returns true if this is a streamable asset type.
    ///
    /// Streamable assets can be partially loaded and played before
    /// fully loaded (e.g., audio, video).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// assert!(AssetType::Audio.is_streamable());
    /// assert!(!AssetType::Texture.is_streamable());
    /// ```
    #[inline]
    pub const fn is_streamable(&self) -> bool {
        matches!(self, AssetType::Audio)
    }

    /// Returns a human-readable name for this asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetType;
    ///
    /// assert_eq!(AssetType::Texture.name(), "Texture");
    /// assert_eq!(AssetType::Audio.name(), "Audio");
    /// ```
    #[inline]
    pub const fn name(&self) -> &'static str {
        match self {
            AssetType::Custom => "Custom",
            AssetType::Texture => "Texture",
            AssetType::Audio => "Audio",
            AssetType::Mesh => "Mesh",
            AssetType::Shader => "Shader",
            AssetType::Font => "Font",
            AssetType::Material => "Material",
            AssetType::Animation => "Animation",
            AssetType::TiledMap => "TiledMap",
            AssetType::Prefab => "Prefab",
            AssetType::Config => "Config",
            AssetType::Binary => "Binary",
            AssetType::Text => "Text",
        }
    }
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<AssetType> for u8 {
    #[inline]
    fn from(value: AssetType) -> u8 {
        value as u8
    }
}

impl TryFrom<u8> for AssetType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AssetType::Custom),
            1 => Ok(AssetType::Texture),
            2 => Ok(AssetType::Audio),
            3 => Ok(AssetType::Mesh),
            4 => Ok(AssetType::Shader),
            5 => Ok(AssetType::Font),
            6 => Ok(AssetType::Material),
            7 => Ok(AssetType::Animation),
            8 => Ok(AssetType::TiledMap),
            9 => Ok(AssetType::Prefab),
            10 => Ok(AssetType::Config),
            11 => Ok(AssetType::Binary),
            12 => Ok(AssetType::Text),
            _ => Err(value),
        }
    }
}

// =============================================================================
// AssetState
// =============================================================================

/// Loading state of an asset.
///
/// Tracks the lifecycle of an asset from initial request to loaded (or failed).
/// Used by the asset server to report loading progress.
///
/// # State Transitions
///
/// ```text
/// NotLoaded ──▶ Loading ──▶ Loaded
///                  │
///                  └──▶ Failed
/// ```
///
/// # FFI Safety
///
/// This enum is `#[repr(u8)]` for stable, FFI-compatible representation.
///
/// # Example
///
/// ```
/// use goud_engine::assets::AssetState;
///
/// let state = AssetState::Loading { progress: 0.5 };
///
/// match state {
///     AssetState::Loading { progress } => {
///         println!("Loading: {}%", progress * 100.0);
///     }
///     AssetState::Loaded => println!("Ready!"),
///     AssetState::Failed { .. } => println!("Error!"),
///     _ => {}
/// }
/// ```
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AssetState {
    /// Asset has not been requested for loading.
    #[default]
    NotLoaded = 0,

    /// Asset is currently being loaded.
    ///
    /// Progress is a value from 0.0 to 1.0.
    Loading {
        /// Loading progress (0.0 to 1.0).
        progress: f32,
    } = 1,

    /// Asset has been successfully loaded and is ready for use.
    Loaded = 2,

    /// Asset loading failed.
    Failed {
        /// Error message describing the failure.
        error: String,
    } = 3,

    /// Asset was loaded but has been unloaded from memory.
    Unloaded = 4,
}

impl AssetState {
    /// Returns true if the asset is ready for use.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetState;
    ///
    /// assert!(AssetState::Loaded.is_ready());
    /// assert!(!AssetState::Loading { progress: 0.5 }.is_ready());
    /// ```
    #[inline]
    pub fn is_ready(&self) -> bool {
        matches!(self, AssetState::Loaded)
    }

    /// Returns true if the asset is currently loading.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetState;
    ///
    /// assert!(AssetState::Loading { progress: 0.5 }.is_loading());
    /// assert!(!AssetState::Loaded.is_loading());
    /// ```
    #[inline]
    pub fn is_loading(&self) -> bool {
        matches!(self, AssetState::Loading { .. })
    }

    /// Returns true if asset loading failed.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetState;
    ///
    /// let state = AssetState::Failed { error: "File not found".to_string() };
    /// assert!(state.is_failed());
    /// ```
    #[inline]
    pub fn is_failed(&self) -> bool {
        matches!(self, AssetState::Failed { .. })
    }

    /// Returns the loading progress if currently loading.
    ///
    /// Returns `None` for non-loading states.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetState;
    ///
    /// let state = AssetState::Loading { progress: 0.75 };
    /// assert_eq!(state.progress(), Some(0.75));
    ///
    /// assert_eq!(AssetState::Loaded.progress(), None);
    /// ```
    #[inline]
    pub fn progress(&self) -> Option<f32> {
        match self {
            AssetState::Loading { progress } => Some(*progress),
            _ => None,
        }
    }

    /// Returns the error message if loading failed.
    ///
    /// Returns `None` for non-failed states.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetState;
    ///
    /// let state = AssetState::Failed { error: "Invalid format".to_string() };
    /// assert_eq!(state.error(), Some("Invalid format"));
    ///
    /// assert_eq!(AssetState::Loaded.error(), None);
    /// ```
    #[inline]
    pub fn error(&self) -> Option<&str> {
        match self {
            AssetState::Failed { error } => Some(error),
            _ => None,
        }
    }

    /// Returns the discriminant as a u8 for FFI.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetState;
    ///
    /// assert_eq!(AssetState::NotLoaded.discriminant(), 0);
    /// assert_eq!(AssetState::Loading { progress: 0.0 }.discriminant(), 1);
    /// assert_eq!(AssetState::Loaded.discriminant(), 2);
    /// ```
    #[inline]
    pub fn discriminant(&self) -> u8 {
        match self {
            AssetState::NotLoaded => 0,
            AssetState::Loading { .. } => 1,
            AssetState::Loaded => 2,
            AssetState::Failed { .. } => 3,
            AssetState::Unloaded => 4,
        }
    }
}

impl fmt::Display for AssetState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssetState::NotLoaded => write!(f, "NotLoaded"),
            AssetState::Loading { progress } => write!(f, "Loading({:.0}%)", progress * 100.0),
            AssetState::Loaded => write!(f, "Loaded"),
            AssetState::Failed { error } => write!(f, "Failed({error})"),
            AssetState::Unloaded => write!(f, "Unloaded"),
        }
    }
}

// =============================================================================
// AssetInfo
// =============================================================================

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

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test asset types
    struct TestTexture {
        width: u32,
        height: u32,
        data: Vec<u8>,
    }

    impl Asset for TestTexture {
        fn asset_type_name() -> &'static str {
            "TestTexture"
        }

        fn asset_type() -> AssetType {
            AssetType::Texture
        }

        fn extensions() -> &'static [&'static str] {
            &["png", "jpg", "jpeg"]
        }
    }

    struct TestAudio {
        samples: Vec<f32>,
        sample_rate: u32,
    }

    impl Asset for TestAudio {
        fn asset_type_name() -> &'static str {
            "TestAudio"
        }

        fn asset_type() -> AssetType {
            AssetType::Audio
        }

        fn extensions() -> &'static [&'static str] {
            &["wav", "ogg", "mp3"]
        }
    }

    // Simple asset using defaults
    struct SimpleAsset {
        value: i32,
    }

    impl Asset for SimpleAsset {}

    // =========================================================================
    // Asset Trait Tests
    // =========================================================================

    mod asset_trait {
        use super::*;

        #[test]
        fn test_asset_type_name() {
            assert_eq!(TestTexture::asset_type_name(), "TestTexture");
            assert_eq!(TestAudio::asset_type_name(), "TestAudio");
        }

        #[test]
        fn test_asset_type_name_default() {
            // Default implementation uses std::any::type_name
            let name = SimpleAsset::asset_type_name();
            assert!(name.contains("SimpleAsset"));
        }

        #[test]
        fn test_asset_type() {
            assert_eq!(TestTexture::asset_type(), AssetType::Texture);
            assert_eq!(TestAudio::asset_type(), AssetType::Audio);
        }

        #[test]
        fn test_asset_type_default() {
            assert_eq!(SimpleAsset::asset_type(), AssetType::Custom);
        }

        #[test]
        fn test_extensions() {
            assert!(TestTexture::extensions().contains(&"png"));
            assert!(TestTexture::extensions().contains(&"jpg"));
            assert!(TestAudio::extensions().contains(&"wav"));
        }

        #[test]
        fn test_extensions_default() {
            assert!(SimpleAsset::extensions().is_empty());
        }

        #[test]
        fn test_asset_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<TestTexture>();
            requires_send::<TestAudio>();
            requires_send::<SimpleAsset>();
        }

        #[test]
        fn test_asset_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<TestTexture>();
            requires_sync::<TestAudio>();
            requires_sync::<SimpleAsset>();
        }

        #[test]
        fn test_asset_is_static() {
            fn requires_static<T: 'static>() {}
            requires_static::<TestTexture>();
            requires_static::<TestAudio>();
            requires_static::<SimpleAsset>();
        }

        #[test]
        fn test_asset_trait_bounds() {
            fn requires_asset<T: Asset>() {}
            requires_asset::<TestTexture>();
            requires_asset::<TestAudio>();
            requires_asset::<SimpleAsset>();
        }
    }

    // =========================================================================
    // AssetId Tests
    // =========================================================================

    mod asset_id {
        use super::*;

        #[test]
        fn test_of() {
            let id1 = AssetId::of::<TestTexture>();
            let id2 = AssetId::of::<TestTexture>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_different_types() {
            let tex_id = AssetId::of::<TestTexture>();
            let audio_id = AssetId::of::<TestAudio>();
            assert_ne!(tex_id, audio_id);
        }

        #[test]
        fn test_of_raw() {
            let id1 = AssetId::of::<TestTexture>();
            let id2 = AssetId::of_raw::<TestTexture>();
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_type_id() {
            let id = AssetId::of::<TestTexture>();
            assert_eq!(id.type_id(), TypeId::of::<TestTexture>());
        }

        #[test]
        fn test_debug() {
            let id = AssetId::of::<TestTexture>();
            let debug_str = format!("{id:?}");
            assert!(debug_str.contains("AssetId"));
        }

        #[test]
        fn test_display() {
            let id = AssetId::of::<TestTexture>();
            let display_str = format!("{id}");
            assert!(display_str.contains("AssetId"));
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(AssetId::of::<TestTexture>());
            set.insert(AssetId::of::<TestAudio>());
            assert_eq!(set.len(), 2);

            // Same type should not add again
            set.insert(AssetId::of::<TestTexture>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_ord() {
            use std::collections::BTreeSet;
            let mut set = BTreeSet::new();
            set.insert(AssetId::of::<TestTexture>());
            set.insert(AssetId::of::<TestAudio>());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_clone() {
            let id1 = AssetId::of::<TestTexture>();
            let id2 = id1;
            assert_eq!(id1, id2);
        }

        #[test]
        fn test_copy() {
            let id1 = AssetId::of::<TestTexture>();
            let id2 = id1;
            // Both should be valid
            assert_eq!(id1.type_id(), id2.type_id());
        }
    }

    // =========================================================================
    // AssetType Tests
    // =========================================================================

    mod asset_type {
        use super::*;

        #[test]
        fn test_all() {
            let types = AssetType::all();
            assert_eq!(types.len(), AssetType::count());
            assert!(types.contains(&AssetType::Custom));
            assert!(types.contains(&AssetType::Texture));
            assert!(types.contains(&AssetType::Audio));
        }

        #[test]
        fn test_count() {
            assert_eq!(AssetType::count(), 13);
        }

        #[test]
        fn test_is_gpu_asset() {
            assert!(AssetType::Texture.is_gpu_asset());
            assert!(AssetType::Mesh.is_gpu_asset());
            assert!(AssetType::Shader.is_gpu_asset());
            assert!(AssetType::Font.is_gpu_asset());

            assert!(!AssetType::Audio.is_gpu_asset());
            assert!(!AssetType::Config.is_gpu_asset());
            assert!(!AssetType::Custom.is_gpu_asset());
        }

        #[test]
        fn test_is_streamable() {
            assert!(AssetType::Audio.is_streamable());

            assert!(!AssetType::Texture.is_streamable());
            assert!(!AssetType::Mesh.is_streamable());
            assert!(!AssetType::Custom.is_streamable());
        }

        #[test]
        fn test_name() {
            assert_eq!(AssetType::Custom.name(), "Custom");
            assert_eq!(AssetType::Texture.name(), "Texture");
            assert_eq!(AssetType::Audio.name(), "Audio");
            assert_eq!(AssetType::Mesh.name(), "Mesh");
            assert_eq!(AssetType::Shader.name(), "Shader");
            assert_eq!(AssetType::Font.name(), "Font");
            assert_eq!(AssetType::Material.name(), "Material");
            assert_eq!(AssetType::Animation.name(), "Animation");
            assert_eq!(AssetType::TiledMap.name(), "TiledMap");
            assert_eq!(AssetType::Prefab.name(), "Prefab");
            assert_eq!(AssetType::Config.name(), "Config");
            assert_eq!(AssetType::Binary.name(), "Binary");
            assert_eq!(AssetType::Text.name(), "Text");
        }

        #[test]
        fn test_default() {
            assert_eq!(AssetType::default(), AssetType::Custom);
        }

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", AssetType::Texture), "Texture");
            assert_eq!(format!("{}", AssetType::Audio), "Audio");
        }

        #[test]
        fn test_from_u8() {
            assert_eq!(u8::from(AssetType::Custom), 0);
            assert_eq!(u8::from(AssetType::Texture), 1);
            assert_eq!(u8::from(AssetType::Audio), 2);
            assert_eq!(u8::from(AssetType::Mesh), 3);
            assert_eq!(u8::from(AssetType::Shader), 4);
            assert_eq!(u8::from(AssetType::Font), 5);
            assert_eq!(u8::from(AssetType::Material), 6);
            assert_eq!(u8::from(AssetType::Animation), 7);
            assert_eq!(u8::from(AssetType::TiledMap), 8);
            assert_eq!(u8::from(AssetType::Prefab), 9);
            assert_eq!(u8::from(AssetType::Config), 10);
            assert_eq!(u8::from(AssetType::Binary), 11);
            assert_eq!(u8::from(AssetType::Text), 12);
        }

        #[test]
        fn test_try_from_u8() {
            assert_eq!(AssetType::try_from(0), Ok(AssetType::Custom));
            assert_eq!(AssetType::try_from(1), Ok(AssetType::Texture));
            assert_eq!(AssetType::try_from(2), Ok(AssetType::Audio));
            assert_eq!(AssetType::try_from(12), Ok(AssetType::Text));

            assert_eq!(AssetType::try_from(13), Err(13));
            assert_eq!(AssetType::try_from(255), Err(255));
        }

        #[test]
        fn test_roundtrip_conversion() {
            for asset_type in AssetType::all() {
                let value: u8 = (*asset_type).into();
                let recovered = AssetType::try_from(value).unwrap();
                assert_eq!(*asset_type, recovered);
            }
        }

        #[test]
        fn test_clone() {
            let t1 = AssetType::Texture;
            let t2 = t1;
            assert_eq!(t1, t2);
        }

        #[test]
        fn test_debug() {
            let debug_str = format!("{:?}", AssetType::Texture);
            assert!(debug_str.contains("Texture"));
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;
            let mut set = HashSet::new();
            set.insert(AssetType::Texture);
            set.insert(AssetType::Audio);
            assert_eq!(set.len(), 2);
        }
    }

    // =========================================================================
    // AssetState Tests
    // =========================================================================

    mod asset_state {
        use super::*;

        #[test]
        fn test_is_ready() {
            assert!(AssetState::Loaded.is_ready());
            assert!(!AssetState::NotLoaded.is_ready());
            assert!(!AssetState::Loading { progress: 0.5 }.is_ready());
            assert!(!AssetState::Failed {
                error: "test".to_string()
            }
            .is_ready());
            assert!(!AssetState::Unloaded.is_ready());
        }

        #[test]
        fn test_is_loading() {
            assert!(AssetState::Loading { progress: 0.0 }.is_loading());
            assert!(AssetState::Loading { progress: 0.5 }.is_loading());
            assert!(AssetState::Loading { progress: 1.0 }.is_loading());
            assert!(!AssetState::NotLoaded.is_loading());
            assert!(!AssetState::Loaded.is_loading());
        }

        #[test]
        fn test_is_failed() {
            assert!(AssetState::Failed {
                error: "error".to_string()
            }
            .is_failed());
            assert!(!AssetState::Loaded.is_failed());
            assert!(!AssetState::Loading { progress: 0.5 }.is_failed());
        }

        #[test]
        fn test_progress() {
            assert_eq!(AssetState::Loading { progress: 0.0 }.progress(), Some(0.0));
            assert_eq!(AssetState::Loading { progress: 0.5 }.progress(), Some(0.5));
            assert_eq!(AssetState::Loading { progress: 1.0 }.progress(), Some(1.0));
            assert_eq!(AssetState::NotLoaded.progress(), None);
            assert_eq!(AssetState::Loaded.progress(), None);
        }

        #[test]
        fn test_error() {
            let state = AssetState::Failed {
                error: "File not found".to_string(),
            };
            assert_eq!(state.error(), Some("File not found"));

            assert_eq!(AssetState::Loaded.error(), None);
            assert_eq!(AssetState::Loading { progress: 0.5 }.error(), None);
        }

        #[test]
        fn test_discriminant() {
            assert_eq!(AssetState::NotLoaded.discriminant(), 0);
            assert_eq!(AssetState::Loading { progress: 0.0 }.discriminant(), 1);
            assert_eq!(AssetState::Loaded.discriminant(), 2);
            assert_eq!(
                AssetState::Failed {
                    error: "".to_string()
                }
                .discriminant(),
                3
            );
            assert_eq!(AssetState::Unloaded.discriminant(), 4);
        }

        #[test]
        fn test_default() {
            assert_eq!(AssetState::default(), AssetState::NotLoaded);
        }

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", AssetState::NotLoaded), "NotLoaded");
            assert_eq!(
                format!("{}", AssetState::Loading { progress: 0.5 }),
                "Loading(50%)"
            );
            assert_eq!(format!("{}", AssetState::Loaded), "Loaded");
            assert_eq!(
                format!(
                    "{}",
                    AssetState::Failed {
                        error: "oops".to_string()
                    }
                ),
                "Failed(oops)"
            );
            assert_eq!(format!("{}", AssetState::Unloaded), "Unloaded");
        }

        #[test]
        fn test_clone() {
            let state1 = AssetState::Loading { progress: 0.75 };
            let state2 = state1.clone();
            assert_eq!(state1, state2);
        }

        #[test]
        fn test_eq() {
            assert_eq!(AssetState::Loaded, AssetState::Loaded);
            assert_ne!(AssetState::Loaded, AssetState::NotLoaded);

            assert_eq!(
                AssetState::Loading { progress: 0.5 },
                AssetState::Loading { progress: 0.5 }
            );
            assert_ne!(
                AssetState::Loading { progress: 0.5 },
                AssetState::Loading { progress: 0.6 }
            );
        }

        #[test]
        fn test_debug() {
            let debug_str = format!("{:?}", AssetState::Loaded);
            assert!(debug_str.contains("Loaded"));
        }
    }

    // =========================================================================
    // AssetInfo Tests
    // =========================================================================

    mod asset_info {
        use super::*;

        #[test]
        fn test_of() {
            let info = AssetInfo::of::<TestTexture>();
            assert_eq!(info.name, "TestTexture");
            assert_eq!(info.asset_type, AssetType::Texture);
        }

        #[test]
        fn test_id() {
            let info = AssetInfo::of::<TestTexture>();
            assert_eq!(info.id, AssetId::of::<TestTexture>());
        }

        #[test]
        fn test_size() {
            let info = AssetInfo::of::<TestTexture>();
            assert_eq!(info.size, std::mem::size_of::<TestTexture>());
        }

        #[test]
        fn test_align() {
            let info = AssetInfo::of::<TestTexture>();
            assert_eq!(info.align, std::mem::align_of::<TestTexture>());
        }

        #[test]
        fn test_extensions() {
            let info = AssetInfo::of::<TestTexture>();
            assert!(info.extensions.contains(&"png"));
            assert!(info.extensions.contains(&"jpg"));
        }

        #[test]
        fn test_display() {
            let info = AssetInfo::of::<TestTexture>();
            let display_str = format!("{info}");
            assert!(display_str.contains("TestTexture"));
            assert!(display_str.contains("Texture"));
        }

        #[test]
        fn test_debug() {
            let info = AssetInfo::of::<TestTexture>();
            let debug_str = format!("{info:?}");
            assert!(debug_str.contains("AssetInfo"));
            assert!(debug_str.contains("TestTexture"));
        }

        #[test]
        fn test_clone() {
            let info1 = AssetInfo::of::<TestTexture>();
            let info2 = info1.clone();
            assert_eq!(info1.name, info2.name);
            assert_eq!(info1.id, info2.id);
        }

        #[test]
        fn test_default_asset() {
            let info = AssetInfo::of::<SimpleAsset>();
            assert_eq!(info.asset_type, AssetType::Custom);
            assert!(info.extensions.is_empty());
        }
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_asset_id_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetId>();
        }

        #[test]
        fn test_asset_id_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetId>();
        }

        #[test]
        fn test_asset_type_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetType>();
        }

        #[test]
        fn test_asset_type_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetType>();
        }

        #[test]
        fn test_asset_state_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetState>();
        }

        #[test]
        fn test_asset_state_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetState>();
        }

        #[test]
        fn test_asset_info_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetInfo>();
        }

        #[test]
        fn test_asset_info_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetInfo>();
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_asset_workflow() {
            // Define asset
            struct GameTexture {
                id: u32,
                width: u32,
                height: u32,
            }

            impl Asset for GameTexture {
                fn asset_type_name() -> &'static str {
                    "GameTexture"
                }

                fn asset_type() -> AssetType {
                    AssetType::Texture
                }

                fn extensions() -> &'static [&'static str] {
                    &["png", "dds"]
                }
            }

            // Get asset info
            let info = AssetInfo::of::<GameTexture>();
            assert_eq!(info.name, "GameTexture");
            assert!(info.asset_type.is_gpu_asset());

            // Get asset ID
            let id = AssetId::of::<GameTexture>();
            assert_eq!(id, info.id);
        }

        #[test]
        fn test_multiple_asset_types() {
            use std::collections::HashMap;

            let mut registry: HashMap<AssetId, AssetInfo> = HashMap::new();

            registry.insert(AssetId::of::<TestTexture>(), AssetInfo::of::<TestTexture>());
            registry.insert(AssetId::of::<TestAudio>(), AssetInfo::of::<TestAudio>());
            registry.insert(AssetId::of::<SimpleAsset>(), AssetInfo::of::<SimpleAsset>());

            assert_eq!(registry.len(), 3);

            // Lookup by ID
            let tex_info = registry.get(&AssetId::of::<TestTexture>()).unwrap();
            assert_eq!(tex_info.name, "TestTexture");

            let audio_info = registry.get(&AssetId::of::<TestAudio>()).unwrap();
            assert_eq!(audio_info.name, "TestAudio");
        }

        #[test]
        fn test_asset_state_transitions() {
            let mut state = AssetState::NotLoaded;
            assert!(!state.is_ready());

            state = AssetState::Loading { progress: 0.0 };
            assert!(state.is_loading());
            assert_eq!(state.progress(), Some(0.0));

            state = AssetState::Loading { progress: 0.5 };
            assert_eq!(state.progress(), Some(0.5));

            state = AssetState::Loading { progress: 1.0 };
            assert_eq!(state.progress(), Some(1.0));

            state = AssetState::Loaded;
            assert!(state.is_ready());
            assert!(!state.is_loading());
        }

        #[test]
        fn test_asset_state_failure_path() {
            let mut state = AssetState::NotLoaded;

            state = AssetState::Loading { progress: 0.3 };
            assert!(state.is_loading());

            state = AssetState::Failed {
                error: "File not found: player.png".to_string(),
            };
            assert!(state.is_failed());
            assert_eq!(state.error(), Some("File not found: player.png"));
            assert!(!state.is_ready());
        }
    }
}
