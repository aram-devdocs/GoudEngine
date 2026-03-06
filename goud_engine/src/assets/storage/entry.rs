//! [`AssetEntry`]: individual asset entry with metadata.

use crate::assets::{Asset, AssetPath, AssetState};

// =============================================================================
// AssetEntry
// =============================================================================

/// An individual asset entry in storage with metadata.
///
/// `AssetEntry` wraps an asset value together with its loading state,
/// optional path association, and other metadata.
///
/// # Example
///
/// ```
/// use goud_engine::assets::{Asset, AssetEntry, AssetState, AssetPath};
///
/// struct Texture { width: u32 }
/// impl Asset for Texture {}
///
/// // Create a loaded entry
/// let entry = AssetEntry::loaded(Texture { width: 256 });
/// assert!(entry.is_loaded());
/// assert_eq!(entry.asset().unwrap().width, 256);
///
/// // Create entry with path
/// let entry = AssetEntry::with_path(
///     Texture { width: 512 },
///     AssetPath::new("textures/player.png"),
/// );
/// assert_eq!(entry.path().map(|p| p.as_str()), Some("textures/player.png"));
/// ```
#[derive(Debug, Clone)]
pub struct AssetEntry<A: Asset> {
    /// The asset data, if loaded.
    asset: Option<A>,

    /// Current loading state.
    state: AssetState,

    /// Optional path this asset was loaded from.
    path: Option<AssetPath<'static>>,
}

impl<A: Asset> AssetEntry<A> {
    /// Creates a new entry in the `NotLoaded` state.
    ///
    /// Use this when you want to reserve a handle before the asset is loaded.
    #[inline]
    pub fn empty() -> Self {
        Self {
            asset: None,
            state: AssetState::NotLoaded,
            path: None,
        }
    }

    /// Creates a new entry with a loading state.
    ///
    /// # Arguments
    ///
    /// * `progress` - Initial loading progress (0.0 to 1.0)
    #[inline]
    pub fn loading(progress: f32) -> Self {
        Self {
            asset: None,
            state: AssetState::Loading { progress },
            path: None,
        }
    }

    /// Creates a new entry with a loaded asset.
    #[inline]
    pub fn loaded(asset: A) -> Self {
        Self {
            asset: Some(asset),
            state: AssetState::Loaded,
            path: None,
        }
    }

    /// Creates a new entry with a loaded asset and path.
    pub fn with_path(asset: A, path: AssetPath<'static>) -> Self {
        Self {
            asset: Some(asset),
            state: AssetState::Loaded,
            path: Some(path),
        }
    }

    /// Creates a new failed entry with an error message.
    #[inline]
    pub fn failed(error: impl Into<String>) -> Self {
        Self {
            asset: None,
            state: AssetState::Failed {
                error: error.into(),
            },
            path: None,
        }
    }

    /// Returns a reference to the asset if loaded.
    #[inline]
    pub fn asset(&self) -> Option<&A> {
        self.asset.as_ref()
    }

    /// Returns a mutable reference to the asset if loaded.
    #[inline]
    pub fn asset_mut(&mut self) -> Option<&mut A> {
        self.asset.as_mut()
    }

    /// Takes the asset out of this entry, leaving `None`.
    #[inline]
    pub fn take_asset(&mut self) -> Option<A> {
        let asset = self.asset.take();
        if asset.is_some() {
            self.state = AssetState::Unloaded;
        }
        asset
    }

    /// Returns a reference to the current state.
    #[inline]
    pub fn state(&self) -> &AssetState {
        &self.state
    }

    /// Returns the path this asset was loaded from, if any.
    #[inline]
    pub fn path(&self) -> Option<&AssetPath<'static>> {
        self.path.as_ref()
    }

    /// Sets the path for this entry.
    #[inline]
    pub fn set_path(&mut self, path: AssetPath<'static>) {
        self.path = Some(path);
    }

    /// Clears the path for this entry.
    #[inline]
    pub fn clear_path(&mut self) {
        self.path = None;
    }

    /// Returns `true` if the asset is fully loaded.
    #[inline]
    pub fn is_loaded(&self) -> bool {
        self.state.is_ready() && self.asset.is_some()
    }

    /// Returns `true` if the asset is currently loading.
    #[inline]
    pub fn is_loading(&self) -> bool {
        self.state.is_loading()
    }

    /// Returns `true` if loading failed.
    #[inline]
    pub fn is_failed(&self) -> bool {
        self.state.is_failed()
    }

    /// Sets the asset and marks as loaded.
    pub fn set_loaded(&mut self, asset: A) {
        self.asset = Some(asset);
        self.state = AssetState::Loaded;
    }

    /// Updates the loading progress.
    pub fn set_progress(&mut self, progress: f32) {
        self.state = AssetState::Loading { progress };
    }

    /// Marks the entry as failed.
    pub fn set_failed(&mut self, error: impl Into<String>) {
        self.asset = None;
        self.state = AssetState::Failed {
            error: error.into(),
        };
    }

    /// Marks the entry as unloaded and removes the asset.
    pub fn set_unloaded(&mut self) {
        self.asset = None;
        self.state = AssetState::Unloaded;
    }
}

impl<A: Asset> Default for AssetEntry<A> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}
