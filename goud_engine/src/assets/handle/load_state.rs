//! Combined handle and load state type for convenient asset status checking.

use crate::assets::{Asset, AssetState};

use super::typed::AssetHandle;

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
