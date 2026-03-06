//! Loading state tracking for assets.

use std::fmt;

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
            AssetState::Failed { error } => write!(f, "Failed({})", error),
            AssetState::Unloaded => write!(f, "Unloaded"),
        }
    }
}
