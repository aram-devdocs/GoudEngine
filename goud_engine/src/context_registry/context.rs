//! GoudContext -- a single engine context with its own World.
//!
//! Each context is isolated: it has its own entities, components, resources,
//! and assets.  Multiple contexts can exist simultaneously (e.g. for multiple
//! game instances or editor viewports).

use std::collections::HashSet;

use crate::context_registry::scene::{SceneId, SceneManager};
use crate::ecs::World;

/// A single engine context containing scene management and associated state.
///
/// Each context is isolated - it has its own scenes (each with its own World),
/// components, resources, and assets. Multiple contexts can exist simultaneously
/// (e.g., for multiple game instances or editor viewports).
///
/// # Thread Safety
///
/// Contexts are NOT Send or Sync - they must be used from a single thread.
/// The registry that holds contexts IS thread-safe.
pub struct GoudContext {
    /// Scene manager holding one or more isolated worlds.
    scene_manager: SceneManager,

    /// The scene currently targeted by `world()` / `world_mut()`.
    current_scene: SceneId,

    /// Generation counter for this context slot.
    ///
    /// When a context is destroyed, the generation increments. This detects
    /// use-after-free when old IDs are used.
    generation: u32,

    /// Runtime plugin registry tracking registered plugin IDs by name.
    ///
    /// This is a string-based registry for FFI consumers to register and
    /// query plugins at runtime. It is separate from the Rust-level `Plugin`
    /// trait system which operates at compile time.
    registered_plugins: HashSet<String>,

    /// Thread ID that created this context (for validation in test builds).
    #[cfg(test)]
    owner_thread: std::thread::ThreadId,
}

impl GoudContext {
    /// Creates a new context with the given generation.
    pub(crate) fn new(generation: u32) -> Self {
        let scene_manager = SceneManager::new();
        let current_scene = scene_manager.default_scene();
        Self {
            scene_manager,
            current_scene,
            generation,
            registered_plugins: HashSet::new(),
            #[cfg(test)]
            owner_thread: std::thread::current().id(),
        }
    }

    /// Returns a reference to the current scene's world.
    pub fn world(&self) -> &World {
        self.scene_manager
            .get_scene(self.current_scene)
            .expect("current scene must exist")
    }

    /// Returns a mutable reference to the current scene's world.
    pub fn world_mut(&mut self) -> &mut World {
        self.scene_manager
            .get_scene_mut(self.current_scene)
            .expect("current scene must exist")
    }

    /// Returns a reference to the scene manager.
    pub fn scene_manager(&self) -> &SceneManager {
        &self.scene_manager
    }

    /// Returns a mutable reference to the scene manager.
    pub fn scene_manager_mut(&mut self) -> &mut SceneManager {
        &mut self.scene_manager
    }

    /// Returns the currently targeted scene ID.
    pub fn current_scene(&self) -> SceneId {
        self.current_scene
    }

    /// Sets the scene targeted by `world()` / `world_mut()`.
    ///
    /// # Errors
    ///
    /// Returns `GoudError::ResourceNotFound` if the scene does not exist.
    pub fn set_current_scene(&mut self, id: SceneId) -> Result<(), crate::core::error::GoudError> {
        if self.scene_manager.get_scene(id).is_none() {
            return Err(crate::core::error::GoudError::ResourceNotFound(format!(
                "Scene id {} not found",
                id
            )));
        }
        self.current_scene = id;
        Ok(())
    }

    /// Destroys a scene and resets the current scene to default if needed.
    ///
    /// If the destroyed scene was the current scene, the current scene is automatically
    /// reset to the default scene to prevent dangling references.
    ///
    /// # Errors
    ///
    /// Returns an error if the scene does not exist or if attempting to destroy
    /// the default scene.
    pub fn destroy_scene(&mut self, id: SceneId) -> Result<(), crate::core::error::GoudError> {
        self.scene_manager.destroy_scene(id)?;
        // If the destroyed scene was current, reset to default
        if self.current_scene == id {
            self.current_scene = self.scene_manager.default_scene();
        }
        Ok(())
    }

    // =========================================================================
    // Plugin Registry
    // =========================================================================

    /// Registers a plugin by ID. Returns `true` if newly registered,
    /// `false` if already present.
    pub fn register_plugin(&mut self, plugin_id: &str) -> bool {
        self.registered_plugins.insert(plugin_id.to_string())
    }

    /// Unregisters a plugin by ID. Returns `true` if it was registered,
    /// `false` if it was not present.
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> bool {
        self.registered_plugins.remove(plugin_id)
    }

    /// Returns whether a plugin with the given ID is registered.
    pub fn is_plugin_registered(&self, plugin_id: &str) -> bool {
        self.registered_plugins.contains(plugin_id)
    }

    /// Returns a sorted list of registered plugin IDs.
    pub fn registered_plugins(&self) -> Vec<&str> {
        let mut plugins: Vec<&str> = self.registered_plugins.iter().map(|s| s.as_str()).collect();
        plugins.sort_unstable();
        plugins
    }

    /// Returns the generation of this context.
    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }

    /// Validates that this context is being accessed from the correct thread.
    ///
    /// Panics if called from a different thread than the one that created the context.
    /// Available in test builds only; use this in tests to verify thread safety invariants.
    #[cfg(test)]
    pub(crate) fn validate_thread(&self) {
        let current = std::thread::current().id();
        if current != self.owner_thread {
            panic!(
                "GoudContext accessed from wrong thread! Created on {:?}, accessed from {:?}",
                self.owner_thread, current
            );
        }
    }
}

impl std::fmt::Debug for GoudContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoudContext")
            .field("scene_manager", &self.scene_manager)
            .field("current_scene", &self.current_scene)
            .field("generation", &self.generation)
            .finish()
    }
}
