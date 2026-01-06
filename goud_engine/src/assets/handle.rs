//! Asset handles for type-safe, reference-counted asset access.
//!
//! This module provides specialized handle types for the asset system:
//!
//! - [`AssetHandle<A>`]: A typed handle to a specific asset type
//! - [`UntypedAssetHandle`]: A type-erased handle for dynamic asset access
//! - [`AssetPath`]: A path-based identifier for assets
//!
//! # Design Philosophy
//!
//! Asset handles differ from regular [`Handle<T>`](crate::core::handle::Handle) in several ways:
//!
//! 1. **Load State Tracking**: Asset handles know if their asset is loading, loaded, or failed
//! 2. **Path Association**: Assets can be loaded by path, and handles preserve this association
//! 3. **Type Erasure**: Untyped handles allow heterogeneous asset collections
//! 4. **Reference Semantics**: Multiple handles can reference the same asset
//!
//! # FFI Safety
//!
//! Both `AssetHandle<A>` and `UntypedAssetHandle` are FFI-compatible:
//! - Fixed size (16 bytes for typed, 24 bytes for untyped)
//! - `#[repr(C)]` layout
//! - Can be converted to/from integer representations
//!
//! # Example
//!
//! ```
//! use goud_engine::assets::{Asset, AssetHandle, AssetPath, HandleLoadState};
//!
//! // Define a texture asset type
//! struct Texture { width: u32, height: u32 }
//! impl Asset for Texture {}
//!
//! // Create a handle (normally done by AssetServer)
//! let handle: AssetHandle<Texture> = AssetHandle::new(0, 1);
//!
//! // Check handle state
//! assert!(handle.is_valid());
//!
//! // Asset handles can be associated with paths
//! let path = AssetPath::new("textures/player.png");
//! assert_eq!(path.extension(), Some("png"));
//! ```

use crate::assets::{Asset, AssetId, AssetState, AssetType};
use std::borrow::Cow;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::path::Path;

// =============================================================================
// AssetHandle<A>
// =============================================================================

/// A typed handle to an asset of type `A`.
///
/// `AssetHandle` is the primary way to reference loaded assets. It provides:
///
/// - **Type Safety**: `AssetHandle<Texture>` and `AssetHandle<Audio>` are distinct types
/// - **Generation Counting**: Prevents use-after-free when assets are unloaded
/// - **FFI Compatibility**: Can be passed across language boundaries
///
/// # Handle States
///
/// An asset handle can be in several states:
/// - **Invalid**: The sentinel value `INVALID`, representing "no asset"
/// - **Valid but Loading**: Points to an asset slot where loading is in progress
/// - **Valid and Loaded**: Points to a fully loaded, usable asset
/// - **Stale**: The asset was unloaded, handle generation no longer matches
///
/// Use `AssetServer::get_state()` to check the current state of an asset.
///
/// # FFI Layout
///
/// ```text
/// Offset 0:  index      (u32, 4 bytes)
/// Offset 4:  generation (u32, 4 bytes)
/// Offset 8:  _marker    (PhantomData, 0 bytes)
/// Total:     8 bytes, alignment 4
/// ```
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle};
///
/// struct Shader { /* ... */ }
/// impl Asset for Shader {}
///
/// // Create an invalid handle (default)
/// let handle: AssetHandle<Shader> = AssetHandle::INVALID;
/// assert!(!handle.is_valid());
///
/// // Create a valid handle (normally done by AssetServer)
/// let handle: AssetHandle<Shader> = AssetHandle::new(42, 1);
/// assert!(handle.is_valid());
/// assert_eq!(handle.index(), 42);
/// assert_eq!(handle.generation(), 1);
/// ```
#[repr(C)]
pub struct AssetHandle<A: Asset> {
    /// Slot index in the asset storage.
    index: u32,

    /// Generation counter for stale handle detection.
    generation: u32,

    /// Zero-sized marker for type safety.
    _marker: PhantomData<A>,
}

impl<A: Asset> AssetHandle<A> {
    /// The invalid handle constant, representing "no asset".
    ///
    /// This is the default value for `AssetHandle` and is guaranteed to never
    /// match any valid asset handle.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle};
    ///
    /// struct Mesh;
    /// impl Asset for Mesh {}
    ///
    /// let handle: AssetHandle<Mesh> = AssetHandle::INVALID;
    /// assert!(!handle.is_valid());
    /// assert_eq!(handle.index(), u32::MAX);
    /// assert_eq!(handle.generation(), 0);
    /// ```
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    /// Creates a new asset handle with the given index and generation.
    ///
    /// This is typically called by the asset system, not by user code.
    ///
    /// # Arguments
    ///
    /// * `index` - Slot index in the asset storage
    /// * `generation` - Generation counter for this slot
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle};
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let handle: AssetHandle<Audio> = AssetHandle::new(10, 3);
    /// assert_eq!(handle.index(), 10);
    /// assert_eq!(handle.generation(), 3);
    /// ```
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the index component of this handle.
    ///
    /// The index is the slot number in the asset storage array.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component of this handle.
    ///
    /// The generation is incremented each time a slot is reused,
    /// preventing stale handles from accessing wrong assets.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns `true` if this handle is not the `INVALID` sentinel.
    ///
    /// Note: A "valid" handle may still be stale if the asset was unloaded.
    /// Use `AssetServer::is_alive()` for definitive liveness checks.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle};
    ///
    /// struct Font;
    /// impl Asset for Font {}
    ///
    /// let valid: AssetHandle<Font> = AssetHandle::new(0, 1);
    /// assert!(valid.is_valid());
    ///
    /// let invalid: AssetHandle<Font> = AssetHandle::INVALID;
    /// assert!(!invalid.is_valid());
    /// ```
    #[inline]
    pub const fn is_valid(&self) -> bool {
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Converts this handle to a type-erased `UntypedAssetHandle`.
    ///
    /// The untyped handle preserves the asset type ID for runtime type checking.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, AssetId};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(5, 2);
    /// let untyped = typed.untyped();
    ///
    /// assert_eq!(untyped.index(), 5);
    /// assert_eq!(untyped.generation(), 2);
    /// assert_eq!(untyped.asset_id(), AssetId::of::<Texture>());
    /// ```
    #[inline]
    pub fn untyped(&self) -> UntypedAssetHandle {
        UntypedAssetHandle::new(self.index, self.generation, AssetId::of::<A>())
    }

    /// Packs this handle into a single u64 value for FFI.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    ///
    /// Note: This does NOT preserve the asset type; use `untyped()` if you
    /// need type information across FFI boundaries.
    #[inline]
    pub const fn to_u64(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }

    /// Creates a handle from a packed u64 value.
    ///
    /// Format: upper 32 bits = generation, lower 32 bits = index.
    #[inline]
    pub const fn from_u64(packed: u64) -> Self {
        let index = packed as u32;
        let generation = (packed >> 32) as u32;
        Self::new(index, generation)
    }

    /// Returns the asset type ID for this handle's asset type.
    ///
    /// This is a convenience method equivalent to `AssetId::of::<A>()`.
    #[inline]
    pub fn asset_id() -> AssetId {
        AssetId::of::<A>()
    }

    /// Returns the asset type category for this handle's asset type.
    ///
    /// This is a convenience method equivalent to `A::asset_type()`.
    #[inline]
    pub fn asset_type() -> AssetType {
        A::asset_type()
    }
}

// Trait implementations for AssetHandle<A>

impl<A: Asset> Clone for AssetHandle<A> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Asset> Copy for AssetHandle<A> {}

impl<A: Asset> Default for AssetHandle<A> {
    /// Returns `AssetHandle::INVALID`.
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

impl<A: Asset> fmt::Debug for AssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = A::asset_type_name();
        if self.is_valid() {
            write!(
                f,
                "AssetHandle<{}>({}:{})",
                type_name, self.index, self.generation
            )
        } else {
            write!(f, "AssetHandle<{}>(INVALID)", type_name)
        }
    }
}

impl<A: Asset> fmt::Display for AssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "{}:{}", self.index, self.generation)
        } else {
            write!(f, "INVALID")
        }
    }
}

impl<A: Asset> PartialEq for AssetHandle<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<A: Asset> Eq for AssetHandle<A> {}

impl<A: Asset> Hash for AssetHandle<A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_u64().hash(state);
    }
}

impl<A: Asset> PartialOrd for AssetHandle<A> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A: Asset> Ord for AssetHandle<A> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_u64().cmp(&other.to_u64())
    }
}

// Conversions
impl<A: Asset> From<AssetHandle<A>> for u64 {
    #[inline]
    fn from(handle: AssetHandle<A>) -> u64 {
        handle.to_u64()
    }
}

impl<A: Asset> From<u64> for AssetHandle<A> {
    #[inline]
    fn from(packed: u64) -> Self {
        Self::from_u64(packed)
    }
}

// =============================================================================
// UntypedAssetHandle
// =============================================================================

/// A type-erased handle to any asset type.
///
/// `UntypedAssetHandle` allows storing handles to different asset types in the
/// same collection, at the cost of runtime type checking instead of compile-time.
///
/// # Use Cases
///
/// - Asset registries that store mixed asset types
/// - Dynamic asset loading where type is determined at runtime
/// - FFI where type information must be preserved explicitly
///
/// # FFI Layout
///
/// ```text
/// Offset 0:  index      (u32, 4 bytes)
/// Offset 4:  generation (u32, 4 bytes)
/// Offset 8:  asset_id   (AssetId, varies by platform)
/// ```
///
/// For FFI, use `to_packed()` which provides a consistent representation.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle, AssetId, UntypedAssetHandle};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// struct Audio;
/// impl Asset for Audio {}
///
/// // Create typed handles
/// let tex_handle: AssetHandle<Texture> = AssetHandle::new(1, 1);
/// let audio_handle: AssetHandle<Audio> = AssetHandle::new(2, 1);
///
/// // Convert to untyped for storage in a single collection
/// let handles: Vec<UntypedAssetHandle> = vec![
///     tex_handle.untyped(),
///     audio_handle.untyped(),
/// ];
///
/// // Runtime type checking
/// assert_eq!(handles[0].asset_id(), AssetId::of::<Texture>());
/// assert_eq!(handles[1].asset_id(), AssetId::of::<Audio>());
///
/// // Convert back to typed (with type check)
/// let recovered: Option<AssetHandle<Texture>> = handles[0].typed::<Texture>();
/// assert!(recovered.is_some());
///
/// // Wrong type returns None
/// let wrong: Option<AssetHandle<Audio>> = handles[0].typed::<Audio>();
/// assert!(wrong.is_none());
/// ```
#[derive(Clone, Copy)]
pub struct UntypedAssetHandle {
    /// Slot index in the asset storage.
    index: u32,

    /// Generation counter for stale handle detection.
    generation: u32,

    /// The asset type this handle refers to.
    asset_id: AssetId,
}

impl UntypedAssetHandle {
    /// The invalid untyped handle constant.
    ///
    /// Uses `AssetId::of_raw::<()>()` as a placeholder type ID.
    pub fn invalid() -> Self {
        Self {
            index: u32::MAX,
            generation: 0,
            asset_id: AssetId::of_raw::<()>(),
        }
    }

    /// Creates a new untyped handle.
    ///
    /// # Arguments
    ///
    /// * `index` - Slot index in asset storage
    /// * `generation` - Generation counter
    /// * `asset_id` - The asset type identifier
    #[inline]
    pub const fn new(index: u32, generation: u32, asset_id: AssetId) -> Self {
        Self {
            index,
            generation,
            asset_id,
        }
    }

    /// Creates an untyped handle from a typed handle.
    ///
    /// Equivalent to calling `typed_handle.untyped()`.
    #[inline]
    pub fn from_typed<A: Asset>(handle: AssetHandle<A>) -> Self {
        handle.untyped()
    }

    /// Returns the index component.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns the asset type identifier.
    #[inline]
    pub const fn asset_id(&self) -> AssetId {
        self.asset_id
    }

    /// Returns `true` if this is not the invalid sentinel.
    #[inline]
    pub fn is_valid(&self) -> bool {
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Attempts to convert this untyped handle to a typed handle.
    ///
    /// Returns `Some(handle)` if the asset type matches, `None` otherwise.
    ///
    /// # Type Safety
    ///
    /// This performs a runtime type check using `AssetId`. If you're certain
    /// of the type, you can use `typed_unchecked()` instead.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, UntypedAssetHandle};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(1, 1);
    /// let untyped = typed.untyped();
    ///
    /// // Correct type succeeds
    /// assert!(untyped.typed::<Texture>().is_some());
    ///
    /// // Wrong type fails
    /// assert!(untyped.typed::<Audio>().is_none());
    /// ```
    #[inline]
    pub fn typed<A: Asset>(&self) -> Option<AssetHandle<A>> {
        if self.asset_id == AssetId::of::<A>() {
            Some(AssetHandle::new(self.index, self.generation))
        } else {
            None
        }
    }

    /// Converts this untyped handle to a typed handle without checking the type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that this handle was created from an asset of type `A`.
    /// Using a handle with the wrong type leads to undefined behavior when
    /// accessing the asset data.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, UntypedAssetHandle};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(1, 1);
    /// let untyped = typed.untyped();
    ///
    /// // SAFETY: We know this was created from a Texture handle
    /// let recovered: AssetHandle<Texture> = unsafe { untyped.typed_unchecked() };
    /// assert_eq!(typed, recovered);
    /// ```
    #[inline]
    pub unsafe fn typed_unchecked<A: Asset>(&self) -> AssetHandle<A> {
        AssetHandle::new(self.index, self.generation)
    }

    /// Checks if this handle's type matches the given asset type.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandle, UntypedAssetHandle};
    ///
    /// struct Texture;
    /// impl Asset for Texture {}
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let typed: AssetHandle<Texture> = AssetHandle::new(1, 1);
    /// let untyped = typed.untyped();
    ///
    /// assert!(untyped.is_type::<Texture>());
    /// assert!(!untyped.is_type::<Audio>());
    /// ```
    #[inline]
    pub fn is_type<A: Asset>(&self) -> bool {
        self.asset_id == AssetId::of::<A>()
    }

    /// Packs index and generation into a u64 (does not include type info).
    #[inline]
    pub const fn to_u64(&self) -> u64 {
        ((self.generation as u64) << 32) | (self.index as u64)
    }
}

impl Default for UntypedAssetHandle {
    #[inline]
    fn default() -> Self {
        Self::invalid()
    }
}

impl fmt::Debug for UntypedAssetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(
                f,
                "UntypedAssetHandle({}:{}, {:?})",
                self.index, self.generation, self.asset_id
            )
        } else {
            write!(f, "UntypedAssetHandle(INVALID)")
        }
    }
}

impl fmt::Display for UntypedAssetHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "{}:{}", self.index, self.generation)
        } else {
            write!(f, "INVALID")
        }
    }
}

impl PartialEq for UntypedAssetHandle {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
            && self.generation == other.generation
            && self.asset_id == other.asset_id
    }
}

impl Eq for UntypedAssetHandle {}

impl Hash for UntypedAssetHandle {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
        self.asset_id.hash(state);
    }
}

// =============================================================================
// HandleLoadState
// =============================================================================

/// Combined handle and load state for convenient asset status checking.
///
/// This type wraps an asset handle together with its current loading state,
/// allowing users to check both validity and load progress in one place.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle, AssetState, HandleLoadState};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// // Simulate an asset that's loading
/// let handle: AssetHandle<Texture> = AssetHandle::new(0, 1);
/// let state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.5 });
///
/// assert!(state.is_loading());
/// assert_eq!(state.progress(), Some(0.5));
///
/// // When loaded
/// let state = HandleLoadState::new(handle, AssetState::Loaded);
/// assert!(state.is_ready());
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct HandleLoadState<A: Asset> {
    /// The asset handle.
    handle: AssetHandle<A>,

    /// Current loading state.
    state: AssetState,
}

impl<A: Asset> HandleLoadState<A> {
    /// Creates a new handle load state.
    #[inline]
    pub fn new(handle: AssetHandle<A>, state: AssetState) -> Self {
        Self { handle, state }
    }

    /// Creates a handle load state for an invalid handle.
    #[inline]
    pub fn invalid() -> Self {
        Self {
            handle: AssetHandle::INVALID,
            state: AssetState::NotLoaded,
        }
    }

    /// Returns a reference to the handle.
    #[inline]
    pub fn handle(&self) -> &AssetHandle<A> {
        &self.handle
    }

    /// Returns a reference to the asset state.
    #[inline]
    pub fn state(&self) -> &AssetState {
        &self.state
    }

    /// Returns `true` if the handle is valid (not INVALID sentinel).
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.handle.is_valid()
    }

    /// Returns `true` if the asset is fully loaded and ready for use.
    #[inline]
    pub fn is_ready(&self) -> bool {
        self.handle.is_valid() && self.state.is_ready()
    }

    /// Returns `true` if the asset is currently loading.
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns `true` if the asset failed to load.
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.state.is_failed()
    }

    /// Returns the loading progress if currently loading.
    #[inline]
    pub fn progress(&self) -> Option<f32> {
        self.state.progress()
    }

    /// Returns the error message if loading failed.
    #[inline]
    pub fn error(&self) -> Option<&str> {
        self.state.error()
    }

    /// Consumes self and returns the inner handle.
    #[inline]
    pub fn into_handle(self) -> AssetHandle<A> {
        self.handle
    }

    /// Updates the state.
    #[inline]
    pub fn set_state(&mut self, state: AssetState) {
        self.state = state;
    }
}

impl<A: Asset> Default for HandleLoadState<A> {
    #[inline]
    fn default() -> Self {
        Self::invalid()
    }
}

// =============================================================================
// AssetPath
// =============================================================================

/// A path identifier for assets.
///
/// `AssetPath` represents the path used to load an asset, supporting both
/// owned and borrowed string data. It provides utility methods for working
/// with asset paths.
///
/// # Path Format
///
/// Asset paths use forward slashes as separators, regardless of platform:
/// - `textures/player.png`
/// - `audio/music/theme.ogg`
/// - `shaders/basic.vert`
///
/// # FFI Considerations
///
/// For FFI, convert to a C string using `as_str()` and standard FFI string
/// handling. The path does not include a null terminator by default.
///
/// # Example
///
/// ```
/// use goud_engine::assets::AssetPath;
///
/// let path = AssetPath::new("textures/player.png");
///
/// assert_eq!(path.as_str(), "textures/player.png");
/// assert_eq!(path.file_name(), Some("player.png"));
/// assert_eq!(path.extension(), Some("png"));
/// assert_eq!(path.directory(), Some("textures"));
///
/// // From owned string
/// let owned = AssetPath::from_string("audio/sfx/jump.wav".to_string());
/// assert_eq!(owned.extension(), Some("wav"));
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AssetPath<'a> {
    /// The path string, either borrowed or owned.
    path: Cow<'a, str>,
}

impl<'a> AssetPath<'a> {
    /// Creates a new asset path from a string slice.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let path = AssetPath::new("textures/player.png");
    /// assert_eq!(path.as_str(), "textures/player.png");
    /// ```
    #[inline]
    pub fn new(path: &'a str) -> Self {
        Self {
            path: Cow::Borrowed(path),
        }
    }

    /// Creates a new asset path from an owned string.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let path = AssetPath::from_string("textures/player.png".to_string());
    /// assert_eq!(path.as_str(), "textures/player.png");
    /// ```
    #[inline]
    pub fn from_string(path: String) -> AssetPath<'static> {
        AssetPath {
            path: Cow::Owned(path),
        }
    }

    /// Returns the path as a string slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Returns `true` if the path is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// Returns the length of the path in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.path.len()
    }

    /// Returns the file name component of the path.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("textures/player.png").file_name(), Some("player.png"));
    /// assert_eq!(AssetPath::new("player.png").file_name(), Some("player.png"));
    /// assert_eq!(AssetPath::new("textures/").file_name(), None);
    /// ```
    pub fn file_name(&self) -> Option<&str> {
        let path = self.path.as_ref();
        if path.ends_with('/') {
            return None;
        }
        path.rsplit('/').next().filter(|s| !s.is_empty())
    }

    /// Returns the file extension, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("player.png").extension(), Some("png"));
    /// assert_eq!(AssetPath::new("textures/player.png").extension(), Some("png"));
    /// assert_eq!(AssetPath::new("Makefile").extension(), None);
    /// assert_eq!(AssetPath::new(".gitignore").extension(), None);
    /// ```
    pub fn extension(&self) -> Option<&str> {
        let file_name = self.file_name()?;
        let dot_pos = file_name.rfind('.')?;

        // Handle hidden files like ".gitignore" (no extension)
        if dot_pos == 0 {
            return None;
        }

        Some(&file_name[dot_pos + 1..])
    }

    /// Returns the directory component of the path.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("textures/player.png").directory(), Some("textures"));
    /// assert_eq!(AssetPath::new("a/b/c/file.txt").directory(), Some("a/b/c"));
    /// assert_eq!(AssetPath::new("file.txt").directory(), None);
    /// ```
    pub fn directory(&self) -> Option<&str> {
        let path = self.path.as_ref();
        let pos = path.rfind('/')?;
        if pos == 0 {
            return None;
        }
        Some(&path[..pos])
    }

    /// Returns the file stem (file name without extension).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// assert_eq!(AssetPath::new("player.png").stem(), Some("player"));
    /// assert_eq!(AssetPath::new("textures/player.png").stem(), Some("player"));
    /// assert_eq!(AssetPath::new("archive.tar.gz").stem(), Some("archive.tar"));
    /// assert_eq!(AssetPath::new(".gitignore").stem(), Some(".gitignore"));
    /// ```
    pub fn stem(&self) -> Option<&str> {
        let file_name = self.file_name()?;
        if let Some(dot_pos) = file_name.rfind('.') {
            if dot_pos == 0 {
                // Hidden file with no extension
                Some(file_name)
            } else {
                Some(&file_name[..dot_pos])
            }
        } else {
            // No extension
            Some(file_name)
        }
    }

    /// Converts this path to an owned `AssetPath<'static>`.
    ///
    /// If the path is already owned, this is a no-op. If borrowed,
    /// the string is cloned.
    pub fn into_owned(self) -> AssetPath<'static> {
        AssetPath {
            path: Cow::Owned(self.path.into_owned()),
        }
    }

    /// Creates an `AssetPath` from a `std::path::Path`.
    ///
    /// Converts backslashes to forward slashes for platform consistency.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    /// use std::path::Path;
    ///
    /// let path = AssetPath::from_path(Path::new("textures/player.png"));
    /// assert_eq!(path.as_str(), "textures/player.png");
    /// ```
    pub fn from_path(path: &Path) -> AssetPath<'static> {
        let path_str = path.to_string_lossy();
        // Normalize to forward slashes
        let normalized = path_str.replace('\\', "/");
        AssetPath::from_string(normalized)
    }

    /// Joins this path with another path component.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let base = AssetPath::new("textures");
    /// let full = base.join("player.png");
    /// assert_eq!(full.as_str(), "textures/player.png");
    ///
    /// // Handles trailing slashes
    /// let base = AssetPath::new("textures/");
    /// let full = base.join("player.png");
    /// assert_eq!(full.as_str(), "textures/player.png");
    /// ```
    pub fn join(&self, other: &str) -> AssetPath<'static> {
        let base = self.path.trim_end_matches('/');
        let other = other.trim_start_matches('/');

        if base.is_empty() {
            AssetPath::from_string(other.to_string())
        } else if other.is_empty() {
            AssetPath::from_string(base.to_string())
        } else {
            AssetPath::from_string(format!("{}/{}", base, other))
        }
    }

    /// Returns the path with a different extension.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::AssetPath;
    ///
    /// let path = AssetPath::new("textures/player.png");
    /// let new_path = path.with_extension("jpg");
    /// assert_eq!(new_path.as_str(), "textures/player.jpg");
    ///
    /// // Add extension to file without one
    /// let path = AssetPath::new("Makefile");
    /// let new_path = path.with_extension("bak");
    /// assert_eq!(new_path.as_str(), "Makefile.bak");
    /// ```
    pub fn with_extension(&self, ext: &str) -> AssetPath<'static> {
        if let Some(stem) = self.stem() {
            if let Some(dir) = self.directory() {
                AssetPath::from_string(format!("{}/{}.{}", dir, stem, ext))
            } else {
                AssetPath::from_string(format!("{}.{}", stem, ext))
            }
        } else {
            // No file name, just append
            AssetPath::from_string(format!("{}.{}", self.path, ext))
        }
    }
}

impl<'a> fmt::Debug for AssetPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AssetPath({:?})", self.path)
    }
}

impl<'a> fmt::Display for AssetPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl<'a> AsRef<str> for AssetPath<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.path
    }
}

impl<'a> From<&'a str> for AssetPath<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl From<String> for AssetPath<'static> {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl<'a> PartialEq<str> for AssetPath<'a> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.path.as_ref() == other
    }
}

impl<'a> PartialEq<&str> for AssetPath<'a> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.path.as_ref() == *other
    }
}

// =============================================================================
// WeakAssetHandle
// =============================================================================

/// A weak reference to an asset that doesn't prevent unloading.
///
/// Unlike `AssetHandle`, a `WeakAssetHandle` does not contribute to the
/// reference count of an asset. This is useful for caches and lookup tables
/// where you want to reference assets without keeping them loaded.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandle, WeakAssetHandle};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// let strong: AssetHandle<Texture> = AssetHandle::new(1, 1);
/// let weak: WeakAssetHandle<Texture> = WeakAssetHandle::from_handle(&strong);
///
/// assert!(weak.is_valid());
/// assert_eq!(weak.index(), 1);
/// assert_eq!(weak.generation(), 1);
///
/// // Upgrade to strong handle for access (liveness must be checked separately)
/// let upgraded = weak.upgrade();
/// assert_eq!(upgraded.index(), 1);
/// ```
#[repr(C)]
pub struct WeakAssetHandle<A: Asset> {
    /// Slot index.
    index: u32,

    /// Generation counter.
    generation: u32,

    /// Zero-sized marker.
    _marker: PhantomData<A>,
}

impl<A: Asset> WeakAssetHandle<A> {
    /// The invalid weak handle constant.
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    /// Creates a weak handle from a strong handle.
    #[inline]
    pub fn from_handle(handle: &AssetHandle<A>) -> Self {
        Self {
            index: handle.index,
            generation: handle.generation,
            _marker: PhantomData,
        }
    }

    /// Creates a new weak handle with the given index and generation.
    #[inline]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns the index component.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation component.
    #[inline]
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns `true` if not the INVALID sentinel.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        !(self.index == u32::MAX && self.generation == 0)
    }

    /// Upgrades to a strong handle.
    ///
    /// Note: This does NOT check if the asset is still alive. Use
    /// `AssetServer::is_alive()` to check liveness before using the handle.
    #[inline]
    pub fn upgrade(&self) -> AssetHandle<A> {
        AssetHandle::new(self.index, self.generation)
    }
}

impl<A: Asset> Clone for WeakAssetHandle<A> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<A: Asset> Copy for WeakAssetHandle<A> {}

impl<A: Asset> Default for WeakAssetHandle<A> {
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

impl<A: Asset> fmt::Debug for WeakAssetHandle<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = A::asset_type_name();
        if self.is_valid() {
            write!(
                f,
                "WeakAssetHandle<{}>({}:{})",
                type_name, self.index, self.generation
            )
        } else {
            write!(f, "WeakAssetHandle<{}>(INVALID)", type_name)
        }
    }
}

impl<A: Asset> PartialEq for WeakAssetHandle<A> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<A: Asset> Eq for WeakAssetHandle<A> {}

impl<A: Asset> Hash for WeakAssetHandle<A> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<A: Asset> From<&AssetHandle<A>> for WeakAssetHandle<A> {
    #[inline]
    fn from(handle: &AssetHandle<A>) -> Self {
        Self::from_handle(handle)
    }
}

// =============================================================================
// AssetHandleAllocator
// =============================================================================

/// Allocator for asset handles with generation counting and slot reuse.
///
/// `AssetHandleAllocator` manages the allocation and deallocation of asset handles,
/// similar to [`HandleAllocator`](crate::core::handle::HandleAllocator) but specialized
/// for asset-specific use cases.
///
/// # Features
///
/// - **Generation Counting**: Prevents use-after-free by invalidating stale handles
/// - **Slot Reuse**: Deallocated slots are recycled via a free list
/// - **Type Safety**: Each allocator is generic over the asset type
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetHandleAllocator};
///
/// struct Texture;
/// impl Asset for Texture {}
///
/// let mut allocator: AssetHandleAllocator<Texture> = AssetHandleAllocator::new();
///
/// // Allocate handles
/// let h1 = allocator.allocate();
/// let h2 = allocator.allocate();
///
/// assert!(allocator.is_alive(h1));
/// assert!(allocator.is_alive(h2));
///
/// // Deallocate
/// allocator.deallocate(h1);
/// assert!(!allocator.is_alive(h1));
///
/// // Slot is reused with new generation
/// let h3 = allocator.allocate();
/// assert_ne!(h1, h3); // Different generations
/// ```
pub struct AssetHandleAllocator<A: Asset> {
    /// Generation counter for each slot.
    /// Generation starts at 1 (0 reserved for INVALID).
    generations: Vec<u32>,

    /// Free list of available slot indices.
    free_list: Vec<u32>,

    /// Phantom marker for type parameter.
    _marker: PhantomData<A>,
}

impl<A: Asset> AssetHandleAllocator<A> {
    /// Creates a new, empty allocator.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Audio;
    /// impl Asset for Audio {}
    ///
    /// let allocator: AssetHandleAllocator<Audio> = AssetHandleAllocator::new();
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_list: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Creates a new allocator with pre-allocated capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Number of slots to pre-allocate
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Mesh;
    /// impl Asset for Mesh {}
    ///
    /// let allocator: AssetHandleAllocator<Mesh> = AssetHandleAllocator::with_capacity(1000);
    /// assert!(allocator.is_empty());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            generations: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Allocates a new handle.
    ///
    /// Reuses slots from the free list when available, otherwise allocates new slots.
    ///
    /// # Returns
    ///
    /// A new, valid `AssetHandle<A>`.
    ///
    /// # Panics
    ///
    /// Panics if the number of slots exceeds `u32::MAX - 1`.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Shader;
    /// impl Asset for Shader {}
    ///
    /// let mut allocator: AssetHandleAllocator<Shader> = AssetHandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(handle.is_valid());
    /// assert!(allocator.is_alive(handle));
    /// ```
    pub fn allocate(&mut self) -> AssetHandle<A> {
        if let Some(index) = self.free_list.pop() {
            // Reuse slot
            let generation = self.generations[index as usize];
            AssetHandle::new(index, generation)
        } else {
            // Allocate new slot
            let index = self.generations.len();
            assert!(
                index < u32::MAX as usize,
                "AssetHandleAllocator exceeded maximum capacity"
            );

            // New slots start at generation 1
            self.generations.push(1);
            AssetHandle::new(index as u32, 1)
        }
    }

    /// Deallocates a handle, making it stale.
    ///
    /// The slot's generation is incremented, invalidating any handles that
    /// reference the old generation. The slot is added to the free list.
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to deallocate
    ///
    /// # Returns
    ///
    /// `true` if the handle was valid and successfully deallocated,
    /// `false` if the handle was invalid or stale.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Font;
    /// impl Asset for Font {}
    ///
    /// let mut allocator: AssetHandleAllocator<Font> = AssetHandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(allocator.deallocate(handle));
    /// assert!(!allocator.is_alive(handle));
    /// assert!(!allocator.deallocate(handle)); // Already deallocated
    /// ```
    pub fn deallocate(&mut self, handle: AssetHandle<A>) -> bool {
        if !handle.is_valid() {
            return false;
        }

        let index = handle.index() as usize;

        // Check bounds
        if index >= self.generations.len() {
            return false;
        }

        // Check generation matches
        if self.generations[index] != handle.generation() {
            return false;
        }

        // Increment generation (wrap to 1 if overflows to 0)
        let new_gen = self.generations[index].wrapping_add(1);
        self.generations[index] = if new_gen == 0 { 1 } else { new_gen };

        // Add to free list
        self.free_list.push(handle.index());

        true
    }

    /// Checks if a handle is still alive (not deallocated).
    ///
    /// # Arguments
    ///
    /// * `handle` - The handle to check
    ///
    /// # Returns
    ///
    /// `true` if the handle is valid and its generation matches the current slot generation.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Material;
    /// impl Asset for Material {}
    ///
    /// let mut allocator: AssetHandleAllocator<Material> = AssetHandleAllocator::new();
    /// let handle = allocator.allocate();
    ///
    /// assert!(allocator.is_alive(handle));
    ///
    /// allocator.deallocate(handle);
    /// assert!(!allocator.is_alive(handle));
    /// ```
    #[inline]
    pub fn is_alive(&self, handle: AssetHandle<A>) -> bool {
        if !handle.is_valid() {
            return false;
        }

        let index = handle.index() as usize;
        index < self.generations.len() && self.generations[index] == handle.generation()
    }

    /// Returns the number of currently allocated (alive) handles.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Sprite;
    /// impl Asset for Sprite {}
    ///
    /// let mut allocator: AssetHandleAllocator<Sprite> = AssetHandleAllocator::new();
    /// assert_eq!(allocator.len(), 0);
    ///
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    /// assert_eq!(allocator.len(), 2);
    ///
    /// allocator.deallocate(h1);
    /// assert_eq!(allocator.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.generations.len() - self.free_list.len()
    }

    /// Returns the total capacity (number of slots).
    ///
    /// This includes both active and free slots.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.generations.len()
    }

    /// Returns `true` if no handles are currently allocated.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all allocations, invalidating all existing handles.
    ///
    /// This increments all generations and rebuilds the free list.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::assets::{Asset, AssetHandleAllocator};
    ///
    /// struct Animation;
    /// impl Asset for Animation {}
    ///
    /// let mut allocator: AssetHandleAllocator<Animation> = AssetHandleAllocator::new();
    ///
    /// let h1 = allocator.allocate();
    /// let h2 = allocator.allocate();
    /// assert_eq!(allocator.len(), 2);
    ///
    /// allocator.clear();
    /// assert_eq!(allocator.len(), 0);
    /// assert!(!allocator.is_alive(h1));
    /// assert!(!allocator.is_alive(h2));
    /// ```
    pub fn clear(&mut self) {
        // Increment all generations
        for gen in &mut self.generations {
            let new_gen = gen.wrapping_add(1);
            *gen = if new_gen == 0 { 1 } else { new_gen };
        }

        // Rebuild free list
        self.free_list.clear();
        self.free_list.reserve(self.generations.len());
        for i in (0..self.generations.len()).rev() {
            self.free_list.push(i as u32);
        }
    }

    /// Shrinks the free list to fit its contents.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.free_list.shrink_to_fit();
    }

    /// Returns the current generation for a slot index.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn generation_at(&self, index: u32) -> Option<u32> {
        self.generations.get(index as usize).copied()
    }
}

impl<A: Asset> Default for AssetHandleAllocator<A> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Asset> fmt::Debug for AssetHandleAllocator<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = A::asset_type_name();
        f.debug_struct(&format!("AssetHandleAllocator<{}>", type_name))
            .field("len", &self.len())
            .field("capacity", &self.capacity())
            .field("free_slots", &self.free_list.len())
            .finish()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test asset types
    #[derive(Clone, Debug, PartialEq)]
    struct TestTexture {
        #[allow(dead_code)]
        width: u32,
    }

    impl Asset for TestTexture {
        fn asset_type_name() -> &'static str {
            "TestTexture"
        }

        fn asset_type() -> AssetType {
            AssetType::Texture
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    struct TestAudio {
        #[allow(dead_code)]
        duration: f32,
    }

    impl Asset for TestAudio {
        fn asset_type_name() -> &'static str {
            "TestAudio"
        }

        fn asset_type() -> AssetType {
            AssetType::Audio
        }
    }

    // Simple asset with defaults
    struct SimpleAsset;
    impl Asset for SimpleAsset {}

    // =========================================================================
    // AssetHandle Tests
    // =========================================================================

    mod asset_handle {
        use super::*;

        #[test]
        fn test_new() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
            assert_eq!(handle.index(), 42);
            assert_eq!(handle.generation(), 7);
        }

        #[test]
        fn test_invalid() {
            let handle: AssetHandle<TestTexture> = AssetHandle::INVALID;
            assert_eq!(handle.index(), u32::MAX);
            assert_eq!(handle.generation(), 0);
            assert!(!handle.is_valid());
        }

        #[test]
        fn test_is_valid() {
            let valid: AssetHandle<TestTexture> = AssetHandle::new(0, 1);
            assert!(valid.is_valid());

            let invalid: AssetHandle<TestTexture> = AssetHandle::INVALID;
            assert!(!invalid.is_valid());

            // Edge case: index=MAX but gen!=0 is still valid
            let edge: AssetHandle<TestTexture> = AssetHandle::new(u32::MAX, 1);
            assert!(edge.is_valid());
        }

        #[test]
        fn test_default() {
            let handle: AssetHandle<TestTexture> = Default::default();
            assert!(!handle.is_valid());
            assert_eq!(handle, AssetHandle::INVALID);
        }

        #[test]
        fn test_clone_copy() {
            let h1: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let h2 = h1; // Copy
            let h3 = h1.clone();

            assert_eq!(h1, h2);
            assert_eq!(h1, h3);
        }

        #[test]
        fn test_equality() {
            let h1: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let h2: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let h3: AssetHandle<TestTexture> = AssetHandle::new(1, 2);
            let h4: AssetHandle<TestTexture> = AssetHandle::new(2, 1);

            assert_eq!(h1, h2);
            assert_ne!(h1, h3); // Different generation
            assert_ne!(h1, h4); // Different index
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;

            let mut set = HashSet::new();
            set.insert(AssetHandle::<TestTexture>::new(1, 1));
            set.insert(AssetHandle::<TestTexture>::new(2, 1));

            assert_eq!(set.len(), 2);

            // Same handle shouldn't add again
            set.insert(AssetHandle::<TestTexture>::new(1, 1));
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_ord() {
            use std::collections::BTreeSet;

            let mut set = BTreeSet::new();
            set.insert(AssetHandle::<TestTexture>::new(3, 1));
            set.insert(AssetHandle::<TestTexture>::new(1, 1));
            set.insert(AssetHandle::<TestTexture>::new(2, 1));

            let vec: Vec<_> = set.iter().collect();
            assert!(vec[0].index() < vec[1].index());
            assert!(vec[1].index() < vec[2].index());
        }

        #[test]
        fn test_debug() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
            let debug_str = format!("{:?}", handle);
            assert!(debug_str.contains("AssetHandle"));
            assert!(debug_str.contains("TestTexture"));
            assert!(debug_str.contains("42"));
            assert!(debug_str.contains("7"));

            let invalid: AssetHandle<TestTexture> = AssetHandle::INVALID;
            let debug_str = format!("{:?}", invalid);
            assert!(debug_str.contains("INVALID"));
        }

        #[test]
        fn test_display() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
            assert_eq!(format!("{}", handle), "42:7");

            let invalid: AssetHandle<TestTexture> = AssetHandle::INVALID;
            assert_eq!(format!("{}", invalid), "INVALID");
        }

        #[test]
        fn test_to_u64() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(42, 7);
            let packed = handle.to_u64();

            // Upper 32 = generation, lower 32 = index
            assert_eq!(packed & 0xFFFFFFFF, 42);
            assert_eq!(packed >> 32, 7);
        }

        #[test]
        fn test_from_u64() {
            let packed: u64 = (7u64 << 32) | 42u64;
            let handle: AssetHandle<TestTexture> = AssetHandle::from_u64(packed);

            assert_eq!(handle.index(), 42);
            assert_eq!(handle.generation(), 7);
        }

        #[test]
        fn test_u64_roundtrip() {
            let original: AssetHandle<TestTexture> = AssetHandle::new(12345, 99);
            let packed = original.to_u64();
            let recovered: AssetHandle<TestTexture> = AssetHandle::from_u64(packed);

            assert_eq!(original, recovered);
        }

        #[test]
        fn test_from_into_u64() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(10, 20);
            let packed: u64 = handle.into();
            let recovered: AssetHandle<TestTexture> = packed.into();

            assert_eq!(handle, recovered);
        }

        #[test]
        fn test_untyped() {
            let typed: AssetHandle<TestTexture> = AssetHandle::new(5, 3);
            let untyped = typed.untyped();

            assert_eq!(untyped.index(), 5);
            assert_eq!(untyped.generation(), 3);
            assert_eq!(untyped.asset_id(), AssetId::of::<TestTexture>());
        }

        #[test]
        fn test_asset_id() {
            assert_eq!(
                AssetHandle::<TestTexture>::asset_id(),
                AssetId::of::<TestTexture>()
            );
            assert_ne!(
                AssetHandle::<TestTexture>::asset_id(),
                AssetHandle::<TestAudio>::asset_id()
            );
        }

        #[test]
        fn test_asset_type() {
            assert_eq!(AssetHandle::<TestTexture>::asset_type(), AssetType::Texture);
            assert_eq!(AssetHandle::<TestAudio>::asset_type(), AssetType::Audio);
        }

        #[test]
        fn test_size_and_align() {
            // Should be 8 bytes (2 x u32), PhantomData is zero-sized
            assert_eq!(std::mem::size_of::<AssetHandle<TestTexture>>(), 8);
            assert_eq!(std::mem::align_of::<AssetHandle<TestTexture>>(), 4);
        }

        #[test]
        fn test_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetHandle<TestTexture>>();
        }

        #[test]
        fn test_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<AssetHandle<TestTexture>>();
        }
    }

    // =========================================================================
    // UntypedAssetHandle Tests
    // =========================================================================

    mod untyped_asset_handle {
        use super::*;

        #[test]
        fn test_new() {
            let handle = UntypedAssetHandle::new(42, 7, AssetId::of::<TestTexture>());
            assert_eq!(handle.index(), 42);
            assert_eq!(handle.generation(), 7);
            assert_eq!(handle.asset_id(), AssetId::of::<TestTexture>());
        }

        #[test]
        fn test_invalid() {
            let handle = UntypedAssetHandle::invalid();
            assert!(!handle.is_valid());
        }

        #[test]
        fn test_from_typed() {
            let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let untyped = UntypedAssetHandle::from_typed(typed);

            assert_eq!(untyped.index(), 10);
            assert_eq!(untyped.generation(), 5);
            assert_eq!(untyped.asset_id(), AssetId::of::<TestTexture>());
        }

        #[test]
        fn test_typed() {
            let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let untyped = typed.untyped();

            // Correct type succeeds
            let recovered: Option<AssetHandle<TestTexture>> = untyped.typed();
            assert!(recovered.is_some());
            assert_eq!(recovered.unwrap(), typed);

            // Wrong type fails
            let wrong: Option<AssetHandle<TestAudio>> = untyped.typed();
            assert!(wrong.is_none());
        }

        #[test]
        fn test_typed_unchecked() {
            let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let untyped = typed.untyped();

            // SAFETY: We know this was created from TestTexture
            let recovered: AssetHandle<TestTexture> = unsafe { untyped.typed_unchecked() };
            assert_eq!(typed, recovered);
        }

        #[test]
        fn test_is_type() {
            let typed: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let untyped = typed.untyped();

            assert!(untyped.is_type::<TestTexture>());
            assert!(!untyped.is_type::<TestAudio>());
        }

        #[test]
        fn test_equality() {
            let h1 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>());
            let h2 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>());
            let h3 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestAudio>()); // Different type
            let h4 = UntypedAssetHandle::new(1, 2, AssetId::of::<TestTexture>()); // Different gen

            assert_eq!(h1, h2);
            assert_ne!(h1, h3); // Different asset type
            assert_ne!(h1, h4); // Different generation
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;

            let mut set = HashSet::new();
            set.insert(UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>()));
            set.insert(UntypedAssetHandle::new(1, 1, AssetId::of::<TestAudio>()));

            // Different types are different entries
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_debug() {
            let handle = UntypedAssetHandle::new(42, 7, AssetId::of::<TestTexture>());
            let debug_str = format!("{:?}", handle);
            assert!(debug_str.contains("UntypedAssetHandle"));
            assert!(debug_str.contains("42"));
            assert!(debug_str.contains("7"));

            let invalid = UntypedAssetHandle::invalid();
            let debug_str = format!("{:?}", invalid);
            assert!(debug_str.contains("INVALID"));
        }

        #[test]
        fn test_display() {
            let handle = UntypedAssetHandle::new(42, 7, AssetId::of::<TestTexture>());
            assert_eq!(format!("{}", handle), "42:7");

            let invalid = UntypedAssetHandle::invalid();
            assert_eq!(format!("{}", invalid), "INVALID");
        }

        #[test]
        fn test_clone_copy() {
            let h1 = UntypedAssetHandle::new(1, 1, AssetId::of::<TestTexture>());
            let h2 = h1; // Copy
            let h3 = h1.clone();

            assert_eq!(h1, h2);
            assert_eq!(h1, h3);
        }

        #[test]
        fn test_default() {
            let handle: UntypedAssetHandle = Default::default();
            assert!(!handle.is_valid());
        }

        #[test]
        fn test_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<UntypedAssetHandle>();
        }

        #[test]
        fn test_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<UntypedAssetHandle>();
        }
    }

    // =========================================================================
    // HandleLoadState Tests
    // =========================================================================

    mod handle_load_state {
        use super::*;

        #[test]
        fn test_new() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let state = HandleLoadState::new(handle, AssetState::Loaded);

            assert_eq!(*state.handle(), handle);
            assert!(state.is_ready());
        }

        #[test]
        fn test_invalid() {
            let state: HandleLoadState<TestTexture> = HandleLoadState::invalid();
            assert!(!state.is_valid());
            assert!(!state.is_ready());
            assert_eq!(*state.state(), AssetState::NotLoaded);
        }

        #[test]
        fn test_is_ready() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);

            let loaded = HandleLoadState::new(handle, AssetState::Loaded);
            assert!(loaded.is_ready());

            let loading = HandleLoadState::new(handle, AssetState::Loading { progress: 0.5 });
            assert!(!loading.is_ready());

            let failed = HandleLoadState::new(
                handle,
                AssetState::Failed {
                    error: "test".to_string(),
                },
            );
            assert!(!failed.is_ready());
        }

        #[test]
        fn test_is_loading() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.5 });

            assert!(state.is_loading());
            assert_eq!(state.progress(), Some(0.5));
        }

        #[test]
        fn test_is_failed() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let state = HandleLoadState::new(
                handle,
                AssetState::Failed {
                    error: "File not found".to_string(),
                },
            );

            assert!(state.is_failed());
            assert_eq!(state.error(), Some("File not found"));
        }

        #[test]
        fn test_into_handle() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let state = HandleLoadState::new(handle, AssetState::Loaded);

            let recovered = state.into_handle();
            assert_eq!(recovered, handle);
        }

        #[test]
        fn test_set_state() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let mut state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.0 });

            assert!(state.is_loading());

            state.set_state(AssetState::Loaded);
            assert!(state.is_ready());
        }

        #[test]
        fn test_default() {
            let state: HandleLoadState<TestTexture> = Default::default();
            assert!(!state.is_valid());
        }

        #[test]
        fn test_clone() {
            let handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let state1 = HandleLoadState::new(handle, AssetState::Loaded);
            let state2 = state1.clone();

            assert_eq!(state1, state2);
        }
    }

    // =========================================================================
    // AssetPath Tests
    // =========================================================================

    mod asset_path {
        use super::*;

        #[test]
        fn test_new() {
            let path = AssetPath::new("textures/player.png");
            assert_eq!(path.as_str(), "textures/player.png");
        }

        #[test]
        fn test_from_string() {
            let path = AssetPath::from_string("textures/player.png".to_string());
            assert_eq!(path.as_str(), "textures/player.png");
        }

        #[test]
        fn test_is_empty() {
            let empty = AssetPath::new("");
            assert!(empty.is_empty());
            assert_eq!(empty.len(), 0);

            let non_empty = AssetPath::new("file.txt");
            assert!(!non_empty.is_empty());
        }

        #[test]
        fn test_file_name() {
            assert_eq!(
                AssetPath::new("textures/player.png").file_name(),
                Some("player.png")
            );
            assert_eq!(AssetPath::new("player.png").file_name(), Some("player.png"));
            assert_eq!(
                AssetPath::new("a/b/c/file.txt").file_name(),
                Some("file.txt")
            );
            assert_eq!(AssetPath::new("textures/").file_name(), None);
            assert_eq!(AssetPath::new("").file_name(), None);
        }

        #[test]
        fn test_extension() {
            assert_eq!(AssetPath::new("player.png").extension(), Some("png"));
            assert_eq!(
                AssetPath::new("textures/player.png").extension(),
                Some("png")
            );
            assert_eq!(AssetPath::new("archive.tar.gz").extension(), Some("gz"));
            assert_eq!(AssetPath::new("Makefile").extension(), None);
            assert_eq!(AssetPath::new(".gitignore").extension(), None);
            assert_eq!(AssetPath::new("").extension(), None);
        }

        #[test]
        fn test_directory() {
            assert_eq!(
                AssetPath::new("textures/player.png").directory(),
                Some("textures")
            );
            assert_eq!(AssetPath::new("a/b/c/file.txt").directory(), Some("a/b/c"));
            assert_eq!(AssetPath::new("file.txt").directory(), None);
            assert_eq!(AssetPath::new("").directory(), None);
        }

        #[test]
        fn test_stem() {
            assert_eq!(AssetPath::new("player.png").stem(), Some("player"));
            assert_eq!(AssetPath::new("textures/player.png").stem(), Some("player"));
            assert_eq!(AssetPath::new("archive.tar.gz").stem(), Some("archive.tar"));
            assert_eq!(AssetPath::new(".gitignore").stem(), Some(".gitignore"));
            assert_eq!(AssetPath::new("Makefile").stem(), Some("Makefile"));
        }

        #[test]
        fn test_into_owned() {
            let borrowed = AssetPath::new("textures/player.png");
            let owned = borrowed.into_owned();
            assert_eq!(owned.as_str(), "textures/player.png");
        }

        #[test]
        fn test_from_path() {
            let path = AssetPath::from_path(Path::new("textures/player.png"));
            assert_eq!(path.as_str(), "textures/player.png");
        }

        #[test]
        fn test_join() {
            let base = AssetPath::new("textures");
            let full = base.join("player.png");
            assert_eq!(full.as_str(), "textures/player.png");

            // With trailing slash
            let base = AssetPath::new("textures/");
            let full = base.join("player.png");
            assert_eq!(full.as_str(), "textures/player.png");

            // With leading slash in other
            let base = AssetPath::new("textures");
            let full = base.join("/player.png");
            assert_eq!(full.as_str(), "textures/player.png");

            // Empty base
            let base = AssetPath::new("");
            let full = base.join("player.png");
            assert_eq!(full.as_str(), "player.png");

            // Empty other
            let base = AssetPath::new("textures");
            let full = base.join("");
            assert_eq!(full.as_str(), "textures");
        }

        #[test]
        fn test_with_extension() {
            let path = AssetPath::new("textures/player.png");
            let new_path = path.with_extension("jpg");
            assert_eq!(new_path.as_str(), "textures/player.jpg");

            // No extension originally
            let path = AssetPath::new("Makefile");
            let new_path = path.with_extension("bak");
            assert_eq!(new_path.as_str(), "Makefile.bak");

            // No directory
            let path = AssetPath::new("player.png");
            let new_path = path.with_extension("jpg");
            assert_eq!(new_path.as_str(), "player.jpg");
        }

        #[test]
        fn test_equality() {
            let p1 = AssetPath::new("textures/player.png");
            let p2 = AssetPath::new("textures/player.png");
            let p3 = AssetPath::new("textures/enemy.png");

            assert_eq!(p1, p2);
            assert_ne!(p1, p3);
        }

        #[test]
        fn test_equality_with_str() {
            let path = AssetPath::new("textures/player.png");
            assert!(path == "textures/player.png");
            // Note: Can't compare with &&str, only &str, which is fine
            let str_ref: &str = "textures/player.png";
            assert!(path == str_ref);
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;

            let mut set = HashSet::new();
            set.insert(AssetPath::new("a.txt").into_owned());
            set.insert(AssetPath::new("b.txt").into_owned());

            assert_eq!(set.len(), 2);

            set.insert(AssetPath::new("a.txt").into_owned());
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_debug() {
            let path = AssetPath::new("textures/player.png");
            let debug_str = format!("{:?}", path);
            assert!(debug_str.contains("AssetPath"));
            assert!(debug_str.contains("textures/player.png"));
        }

        #[test]
        fn test_display() {
            let path = AssetPath::new("textures/player.png");
            assert_eq!(format!("{}", path), "textures/player.png");
        }

        #[test]
        fn test_as_ref() {
            let path = AssetPath::new("textures/player.png");
            let s: &str = path.as_ref();
            assert_eq!(s, "textures/player.png");
        }

        #[test]
        fn test_from_str() {
            let path: AssetPath = "textures/player.png".into();
            assert_eq!(path.as_str(), "textures/player.png");
        }

        #[test]
        fn test_from_string_into() {
            let path: AssetPath<'static> = "textures/player.png".to_string().into();
            assert_eq!(path.as_str(), "textures/player.png");
        }
    }

    // =========================================================================
    // WeakAssetHandle Tests
    // =========================================================================

    mod weak_asset_handle {
        use super::*;

        #[test]
        fn test_from_handle() {
            let strong: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let weak = WeakAssetHandle::from_handle(&strong);

            assert_eq!(weak.index(), 10);
            assert_eq!(weak.generation(), 5);
        }

        #[test]
        fn test_new() {
            let weak: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(42, 7);
            assert_eq!(weak.index(), 42);
            assert_eq!(weak.generation(), 7);
        }

        #[test]
        fn test_invalid() {
            let weak: WeakAssetHandle<TestTexture> = WeakAssetHandle::INVALID;
            assert!(!weak.is_valid());
        }

        #[test]
        fn test_upgrade() {
            let strong: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let weak = WeakAssetHandle::from_handle(&strong);
            let upgraded = weak.upgrade();

            assert_eq!(upgraded, strong);
        }

        #[test]
        fn test_clone_copy() {
            let w1: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 1);
            let w2 = w1; // Copy
            let w3 = w1.clone();

            assert_eq!(w1, w2);
            assert_eq!(w1, w3);
        }

        #[test]
        fn test_default() {
            let weak: WeakAssetHandle<TestTexture> = Default::default();
            assert!(!weak.is_valid());
        }

        #[test]
        fn test_equality() {
            let w1: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 1);
            let w2: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 1);
            let w3: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(1, 2);

            assert_eq!(w1, w2);
            assert_ne!(w1, w3);
        }

        #[test]
        fn test_hash() {
            use std::collections::HashSet;

            let mut set = HashSet::new();
            set.insert(WeakAssetHandle::<TestTexture>::new(1, 1));
            set.insert(WeakAssetHandle::<TestTexture>::new(2, 1));

            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_debug() {
            let weak: WeakAssetHandle<TestTexture> = WeakAssetHandle::new(42, 7);
            let debug_str = format!("{:?}", weak);
            assert!(debug_str.contains("WeakAssetHandle"));
            assert!(debug_str.contains("TestTexture"));
            assert!(debug_str.contains("42"));
        }

        #[test]
        fn test_from_ref() {
            let strong: AssetHandle<TestTexture> = AssetHandle::new(10, 5);
            let weak: WeakAssetHandle<TestTexture> = (&strong).into();

            assert_eq!(weak.index(), 10);
            assert_eq!(weak.generation(), 5);
        }

        #[test]
        fn test_size_and_align() {
            // Should be 8 bytes (2 x u32), PhantomData is zero-sized
            assert_eq!(std::mem::size_of::<WeakAssetHandle<TestTexture>>(), 8);
            assert_eq!(std::mem::align_of::<WeakAssetHandle<TestTexture>>(), 4);
        }

        #[test]
        fn test_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<WeakAssetHandle<TestTexture>>();
        }

        #[test]
        fn test_is_sync() {
            fn requires_sync<T: Sync>() {}
            requires_sync::<WeakAssetHandle<TestTexture>>();
        }
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_handle_lifecycle() {
            // Simulate asset lifecycle with handles
            let handle: AssetHandle<TestTexture> = AssetHandle::new(0, 1);

            // Initially loading
            let mut state = HandleLoadState::new(handle, AssetState::Loading { progress: 0.0 });
            assert!(state.is_loading());

            // Progress update
            state.set_state(AssetState::Loading { progress: 0.5 });
            assert_eq!(state.progress(), Some(0.5));

            // Loaded
            state.set_state(AssetState::Loaded);
            assert!(state.is_ready());
        }

        #[test]
        fn test_mixed_handle_collection() {
            // Store different asset types in single collection
            let tex_handle: AssetHandle<TestTexture> = AssetHandle::new(1, 1);
            let audio_handle: AssetHandle<TestAudio> = AssetHandle::new(2, 1);

            let handles: Vec<UntypedAssetHandle> =
                vec![tex_handle.untyped(), audio_handle.untyped()];

            // Can filter by type
            let textures: Vec<_> = handles
                .iter()
                .filter_map(|h| h.typed::<TestTexture>())
                .collect();
            assert_eq!(textures.len(), 1);
            assert_eq!(textures[0], tex_handle);
        }

        #[test]
        fn test_path_to_handle_workflow() {
            // Simulate path -> handle workflow
            let path = AssetPath::new("textures/player.png");
            assert_eq!(path.extension(), Some("png"));

            // Asset system would create handle
            let handle: AssetHandle<TestTexture> = AssetHandle::new(0, 1);

            // Check type matches extension
            assert_eq!(AssetHandle::<TestTexture>::asset_type(), AssetType::Texture);
        }

        #[test]
        fn test_weak_handle_usage() {
            // Strong handle
            let strong: AssetHandle<TestTexture> = AssetHandle::new(1, 1);

            // Create weak reference (for cache)
            let weak = WeakAssetHandle::from_handle(&strong);

            // Upgrade when needed
            let upgraded = weak.upgrade();
            assert_eq!(upgraded, strong);
        }
    }

    // =========================================================================
    // AssetHandleAllocator Tests
    // =========================================================================

    mod asset_handle_allocator {
        use super::*;

        #[test]
        fn test_new() {
            let allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();
            assert_eq!(allocator.len(), 0);
            assert!(allocator.is_empty());
            assert_eq!(allocator.capacity(), 0);
        }

        #[test]
        fn test_with_capacity() {
            let allocator: AssetHandleAllocator<TestTexture> =
                AssetHandleAllocator::with_capacity(100);
            assert!(allocator.is_empty());
        }

        #[test]
        fn test_allocate() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let h1 = allocator.allocate();
            assert!(h1.is_valid());
            assert_eq!(h1.index(), 0);
            assert_eq!(h1.generation(), 1);

            let h2 = allocator.allocate();
            assert!(h2.is_valid());
            assert_eq!(h2.index(), 1);
            assert_eq!(h2.generation(), 1);

            assert_eq!(allocator.len(), 2);
        }

        #[test]
        fn test_allocate_unique() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();

            // All handles should be unique
            for i in 0..handles.len() {
                for j in (i + 1)..handles.len() {
                    assert_ne!(handles[i], handles[j]);
                }
            }
        }

        #[test]
        fn test_deallocate() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let handle = allocator.allocate();
            assert!(allocator.is_alive(handle));
            assert_eq!(allocator.len(), 1);

            assert!(allocator.deallocate(handle));
            assert!(!allocator.is_alive(handle));
            assert_eq!(allocator.len(), 0);
        }

        #[test]
        fn test_deallocate_invalid() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            // Invalid handle
            assert!(!allocator.deallocate(AssetHandle::INVALID));

            // Handle that was never allocated
            let fake_handle = AssetHandle::new(100, 1);
            assert!(!allocator.deallocate(fake_handle));
        }

        #[test]
        fn test_deallocate_twice() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let handle = allocator.allocate();
            assert!(allocator.deallocate(handle));
            assert!(!allocator.deallocate(handle)); // Already deallocated
        }

        #[test]
        fn test_is_alive() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let handle = allocator.allocate();
            assert!(allocator.is_alive(handle));

            allocator.deallocate(handle);
            assert!(!allocator.is_alive(handle));

            // INVALID is never alive
            assert!(!allocator.is_alive(AssetHandle::INVALID));
        }

        #[test]
        fn test_slot_reuse() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let h1 = allocator.allocate();
            let original_index = h1.index();
            let original_gen = h1.generation();

            allocator.deallocate(h1);

            let h2 = allocator.allocate();

            // Same index, different generation
            assert_eq!(h2.index(), original_index);
            assert_eq!(h2.generation(), original_gen + 1);
            assert_ne!(h1, h2);
        }

        #[test]
        fn test_slot_reuse_multiple() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            // Allocate 10 handles
            let handles: Vec<_> = (0..10).map(|_| allocator.allocate()).collect();

            // Deallocate first 5
            for h in &handles[..5] {
                allocator.deallocate(*h);
            }

            assert_eq!(allocator.len(), 5);
            assert_eq!(allocator.capacity(), 10);

            // Allocate 5 more - should reuse slots
            for _ in 0..5 {
                let h = allocator.allocate();
                assert!(h.index() < 5); // Reused slot
                assert_eq!(h.generation(), 2); // Second generation
            }

            assert_eq!(allocator.len(), 10);
            assert_eq!(allocator.capacity(), 10); // No new slots needed
        }

        #[test]
        fn test_len_and_capacity() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            assert_eq!(allocator.len(), 0);
            assert_eq!(allocator.capacity(), 0);

            let h1 = allocator.allocate();
            let h2 = allocator.allocate();
            let h3 = allocator.allocate();

            assert_eq!(allocator.len(), 3);
            assert_eq!(allocator.capacity(), 3);

            allocator.deallocate(h2);

            assert_eq!(allocator.len(), 2);
            assert_eq!(allocator.capacity(), 3); // Capacity unchanged
        }

        #[test]
        fn test_clear() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            let h1 = allocator.allocate();
            let h2 = allocator.allocate();
            let h3 = allocator.allocate();

            assert_eq!(allocator.len(), 3);

            allocator.clear();

            assert_eq!(allocator.len(), 0);
            assert_eq!(allocator.capacity(), 3); // Capacity retained

            // Old handles are stale
            assert!(!allocator.is_alive(h1));
            assert!(!allocator.is_alive(h2));
            assert!(!allocator.is_alive(h3));

            // New handles have incremented generations
            let h4 = allocator.allocate();
            assert_eq!(h4.generation(), 2);
        }

        #[test]
        fn test_generation_at() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            assert_eq!(allocator.generation_at(0), None);

            let handle = allocator.allocate();
            assert_eq!(allocator.generation_at(0), Some(1));

            allocator.deallocate(handle);
            assert_eq!(allocator.generation_at(0), Some(2));
        }

        #[test]
        fn test_default() {
            let allocator: AssetHandleAllocator<TestTexture> = Default::default();
            assert!(allocator.is_empty());
        }

        #[test]
        fn test_debug() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();
            allocator.allocate();
            allocator.allocate();

            let debug_str = format!("{:?}", allocator);
            assert!(debug_str.contains("AssetHandleAllocator"));
            assert!(debug_str.contains("TestTexture"));
            assert!(debug_str.contains("len"));
            assert!(debug_str.contains("2"));
        }

        #[test]
        fn test_stress_allocate_deallocate() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            // Allocate many
            let handles: Vec<_> = (0..10000).map(|_| allocator.allocate()).collect();
            assert_eq!(allocator.len(), 10000);

            // Deallocate all
            for h in handles {
                assert!(allocator.deallocate(h));
            }
            assert_eq!(allocator.len(), 0);
            assert_eq!(allocator.capacity(), 10000);
        }

        #[test]
        fn test_stress_churn() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            // Simulate churn: allocate/deallocate repeatedly
            for _ in 0..100 {
                let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();
                for h in &handles[..50] {
                    allocator.deallocate(*h);
                }
                // Leave 50 alive
            }

            // Should have 5000 alive (100 iterations * 50 kept)
            assert_eq!(allocator.len(), 5000);
        }

        #[test]
        fn test_generation_wrap() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            // Manually test generation wrapping
            // This is mostly to verify the wrap logic; in practice u32 won't wrap
            let handle = allocator.allocate();

            // Deallocate original handle first - now it's stale
            allocator.deallocate(handle);
            assert!(!allocator.is_alive(handle));

            // Simulate many allocations/deallocations on the same slot
            for _ in 0..10 {
                let new_handle = allocator.allocate();
                allocator.deallocate(new_handle);
            }

            // Original handle should still be stale (gen is now 12, original was 1)
            assert!(!allocator.is_alive(handle));

            // Verify generation increased
            let gen = allocator.generation_at(0).unwrap();
            assert!(gen > handle.generation());
        }

        #[test]
        fn test_shrink_to_fit() {
            let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

            // Allocate then deallocate many
            let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();
            for h in handles {
                allocator.deallocate(h);
            }

            // Free list should be large
            allocator.shrink_to_fit();
            // Can't directly test internal capacity, but shouldn't panic
        }

        #[test]
        fn test_is_send() {
            fn requires_send<T: Send>() {}
            requires_send::<AssetHandleAllocator<TestTexture>>();
        }

        // Note: AssetHandleAllocator is NOT Sync (contains Vec which isn't Sync for &mut)
        // This is intentional - allocators should be accessed through synchronization primitives
    }
}
