//! GoudContext -- a single engine context with its own World.
//!
//! Each context is isolated: it has its own entities, components, resources,
//! and assets.  Multiple contexts can exist simultaneously (e.g. for multiple
//! game instances or editor viewports).

use crate::ecs::World;

/// A single engine context containing a World and associated state.
///
/// Each context is isolated - it has its own entities, components, resources,
/// and assets. Multiple contexts can exist simultaneously (e.g., for multiple
/// game instances or editor viewports).
///
/// # Thread Safety
///
/// Contexts are NOT Send or Sync - they must be used from a single thread.
/// The registry that holds contexts IS thread-safe.
pub struct GoudContext {
    /// The ECS world for this context.
    world: World,

    /// Generation counter for this context slot.
    ///
    /// When a context is destroyed, the generation increments. This detects
    /// use-after-free when old IDs are used.
    generation: u32,

    /// Thread ID that created this context (for validation in test builds).
    #[cfg(test)]
    owner_thread: std::thread::ThreadId,
}

impl GoudContext {
    /// Creates a new context with the given generation.
    pub(crate) fn new(generation: u32) -> Self {
        Self {
            world: World::new(),
            generation,
            #[cfg(test)]
            owner_thread: std::thread::current().id(),
        }
    }

    /// Returns a reference to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Returns a mutable reference to the world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
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
            .field("world", &self.world)
            .field("generation", &self.generation)
            .finish()
    }
}
