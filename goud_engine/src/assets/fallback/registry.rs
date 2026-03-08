//! [`FallbackRegistry`]: stores default asset values for load-failure substitution.

use crate::assets::loaders::{TextureAsset, TextureFormat};
use crate::assets::{Asset, AssetId};
use std::any::Any;
use std::collections::HashMap;

// =============================================================================
// FallbackEntry
// =============================================================================

/// Internal entry that stores a clone factory for a registered fallback asset.
///
/// The `clone_fn` closure captures the original asset value and produces a new
/// boxed clone each time it is called. This allows `FallbackRegistry` to hand
/// out clones without requiring the `Clone` bound on the `Asset` trait itself
/// or on the `AssetServer::load()` signature.
struct FallbackEntry {
    /// Produces a boxed clone of the stored fallback asset.
    clone_fn: Box<dyn Fn() -> Box<dyn Any + Send + Sync> + Send + Sync>,
}

// =============================================================================
// FallbackRegistry
// =============================================================================

/// Registry of default assets used as fallbacks when loading fails.
///
/// Each asset type can have at most one registered fallback. When the asset
/// server encounters a load error for a type that has a registered fallback,
/// it clones the fallback into the entry instead of leaving it empty.
///
/// # Example
///
/// ```
/// use goud_engine::assets::fallback::FallbackRegistry;
/// use goud_engine::assets::loaders::TextureAsset;
///
/// let registry = FallbackRegistry::with_defaults();
///
/// // TextureAsset fallback is registered by default (1x1 magenta pixel)
/// let fallback: Option<TextureAsset> = registry.get_cloned();
/// assert!(fallback.is_some());
/// ```
pub struct FallbackRegistry {
    entries: HashMap<AssetId, FallbackEntry>,
}

impl FallbackRegistry {
    /// Creates an empty registry with no fallbacks registered.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Creates a registry pre-populated with built-in default fallbacks.
    ///
    /// Currently registers:
    /// - [`TextureAsset`]: 1x1 magenta pixel (`[255, 0, 255, 255]`)
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Registers a custom fallback for asset type `A`.
    ///
    /// Overwrites any previously registered fallback for the same type.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Must implement both [`Asset`] and [`Clone`].
    pub fn register<A: Asset + Clone>(&mut self, asset: A) {
        let entry = FallbackEntry {
            clone_fn: Box::new(move || Box::new(asset.clone())),
        };
        self.entries.insert(AssetId::of::<A>(), entry);
    }

    /// Returns a cloned instance of the fallback for asset type `A`, if one
    /// is registered.
    ///
    /// This performs a `downcast` from `Box<dyn Any>` to `A`. Returns `None`
    /// if no fallback is registered for `A` or if the downcast fails
    /// (which should not happen with correct registration).
    pub fn get_cloned<A: Asset>(&self) -> Option<A> {
        let entry = self.entries.get(&AssetId::of::<A>())?;
        let boxed = (entry.clone_fn)();
        boxed.downcast::<A>().ok().map(|b| *b)
    }

    /// Returns `true` if a fallback is registered for asset type `A`.
    pub fn has<A: Asset>(&self) -> bool {
        self.entries.contains_key(&AssetId::of::<A>())
    }

    /// Returns the number of registered fallback types.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if no fallbacks are registered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Registers the built-in default fallbacks.
    ///
    /// Called automatically by [`with_defaults`](Self::with_defaults).
    fn register_defaults(&mut self) {
        // 1x1 magenta texture -- a classic "missing texture" indicator.
        self.register(TextureAsset {
            data: vec![255, 0, 255, 255],
            width: 1,
            height: 1,
            format: TextureFormat::Png,
        });
    }
}

impl Default for FallbackRegistry {
    /// Default creates a registry **with** built-in defaults.
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl std::fmt::Debug for FallbackRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FallbackRegistry")
            .field("registered_types", &self.entries.len())
            .finish()
    }
}
