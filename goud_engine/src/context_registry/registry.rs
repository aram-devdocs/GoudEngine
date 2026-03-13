//! Context registry, context handle, and global accessor.
//!
//! The registry is the central storage for all active contexts. It uses
//! generational indexing to detect use-after-free bugs. The global accessor
//! (`get_context_registry`) provides thread-safe access to the singleton
//! registry instance.

#![allow(clippy::arc_with_non_send_sync)]

use crate::core::debugger::{self, ContextConfig, RuntimeSurfaceKind};
use crate::core::error::GoudError;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use super::context::GoudContext;
use super::context_id::GoudContextId;

// =============================================================================
// Context Slot
// =============================================================================

/// Entry in the context registry, tracking allocation state.
#[derive(Debug)]
enum ContextSlot {
    /// Slot is occupied by a live context.
    ///
    /// Boxed to avoid large enum size difference (World is large).
    Occupied(Box<GoudContext>),

    /// Slot was freed and can be reused (stores next generation).
    Free { next_generation: u32 },
}

// =============================================================================
// Context Registry
// =============================================================================

/// Global registry for all engine contexts.
///
/// This is the central storage for all active contexts. It uses generational
/// indexing to detect use-after-free bugs and allows safe context lookup.
///
/// # Thread Safety
///
/// The registry itself is thread-safe (protected by Mutex), but individual
/// contexts are not. Callers must ensure they don't use a context from
/// multiple threads simultaneously.
pub struct GoudContextRegistry {
    /// Slots for context storage.
    slots: Vec<ContextSlot>,

    /// Free list of indices that can be reused.
    free_list: Vec<u32>,
}

impl GoudContextRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    /// Allocates a new context and returns its ID.
    pub fn create(&mut self) -> Result<GoudContextId, GoudError> {
        self.create_with_config(
            ContextConfig::default(),
            RuntimeSurfaceKind::HeadlessContext,
        )
    }

    /// Allocates a new context and applies the provided debugger configuration.
    pub fn create_with_config(
        &mut self,
        config: ContextConfig,
        surface_kind: RuntimeSurfaceKind,
    ) -> Result<GoudContextId, GoudError> {
        // Try to reuse a free slot first
        let id = if let Some(index) = self.free_list.pop() {
            let generation = match &self.slots[index as usize] {
                ContextSlot::Free { next_generation } => *next_generation,
                ContextSlot::Occupied(_) => {
                    return Err(GoudError::InternalError(
                        "Free list points to occupied slot".to_string(),
                    ))
                }
            };

            let context = GoudContext::new_with_config(generation, config.clone());
            self.slots[index as usize] = ContextSlot::Occupied(Box::new(context));

            GoudContextId::new(index, generation)
        } else {
            // Allocate new slot
            let index = self.slots.len() as u32;
            if index == u32::MAX {
                return Err(GoudError::InternalError(
                    "Context registry full (u32::MAX contexts)".to_string(),
                ));
            }

            let generation = 1; // Generation 0 reserved for "never allocated"
            let context = GoudContext::new_with_config(generation, config.clone());
            self.slots.push(ContextSlot::Occupied(Box::new(context)));

            GoudContextId::new(index, generation)
        };

        if config.debugger.enabled {
            let route = debugger::register_context(id, surface_kind, &config.debugger);
            let context_id_for_hook = id;
            debugger::register_snapshot_refresh_hook_for_route(route.clone(), move |_route_id| {
                let _ = crate::ffi::debug::debugger_runtime::refresh_debugger_snapshot(
                    context_id_for_hook,
                );
            });
            if let Some(context) = self.get_mut(id) {
                context.set_debugger_route(Some(route));
            }
        }

        Ok(id)
    }

    /// Destroys a context and frees its slot for reuse.
    pub fn destroy(&mut self, id: GoudContextId) -> Result<(), GoudError> {
        if id.is_invalid() {
            return Err(GoudError::InvalidContext);
        }

        let index = id.index() as usize;
        if index >= self.slots.len() {
            return Err(GoudError::InvalidContext);
        }

        match &self.slots[index] {
            ContextSlot::Occupied(context) => {
                if context.generation() != id.generation() {
                    return Err(GoudError::InvalidContext);
                }

                let route_to_remove = context.debugger_route().cloned();

                // Increment generation for next allocation
                let next_generation = context.generation().checked_add(1).unwrap_or(1);

                self.slots[index] = ContextSlot::Free { next_generation };
                self.free_list.push(id.index());

                if let Some(ref route_id) = route_to_remove {
                    debugger::unregister_snapshot_refresh_hook_for_route(route_id);
                    debugger::unregister_context(id);
                }

                Ok(())
            }
            ContextSlot::Free { .. } => Err(GoudError::InvalidContext),
        }
    }

    /// Gets an immutable reference to a context.
    pub fn get(&self, id: GoudContextId) -> Option<&GoudContext> {
        if id.is_invalid() {
            return None;
        }

        let index = id.index() as usize;
        if index >= self.slots.len() {
            return None;
        }

        match &self.slots[index] {
            ContextSlot::Occupied(context) => {
                if context.generation() == id.generation() {
                    Some(context)
                } else {
                    None // Stale ID
                }
            }
            ContextSlot::Free { .. } => None,
        }
    }

    /// Gets a mutable reference to a context.
    pub fn get_mut(&mut self, id: GoudContextId) -> Option<&mut GoudContext> {
        if id.is_invalid() {
            return None;
        }

        let index = id.index() as usize;
        if index >= self.slots.len() {
            return None;
        }

        match &mut self.slots[index] {
            ContextSlot::Occupied(context) => {
                if context.generation() == id.generation() {
                    Some(context)
                } else {
                    None // Stale ID
                }
            }
            ContextSlot::Free { .. } => None,
        }
    }

    /// Returns true if the context exists and is valid.
    pub fn is_valid(&self, id: GoudContextId) -> bool {
        self.get(id).is_some()
    }

    /// Returns the number of active contexts.
    pub fn len(&self) -> usize {
        self.slots.len() - self.free_list.len()
    }

    /// Returns true if there are no active contexts.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total capacity (including free slots).
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}

impl Default for GoudContextRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for GoudContextRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GoudContextRegistry")
            .field("active", &self.len())
            .field("capacity", &self.capacity())
            .field("free", &self.free_list.len())
            .finish()
    }
}

// SAFETY: GoudContextRegistry is Send despite containing non-Send World because:
// 1. Access is always synchronized via Mutex in the global registry
// 2. Contexts are single-threaded (thread ownership validated)
// 3. Only the registry structure crosses thread boundaries
unsafe impl Send for GoudContextRegistry {}

// =============================================================================
// Context Handle
// =============================================================================

/// Handle to a context registry.
///
/// This is a thread-safe `Arc<RwLock>` wrapper that allows concurrent access
/// to the registry from multiple threads.
#[derive(Clone)]
#[allow(clippy::arc_with_non_send_sync)]
pub struct GoudContextHandle {
    /// The inner registry.
    pub inner: Arc<RwLock<GoudContextRegistry>>,
}

impl GoudContextHandle {
    /// Creates a new context handle with an empty registry.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(GoudContextRegistry::new())),
        }
    }

    /// Creates a new context and returns its ID.
    pub fn create(&self) -> Result<GoudContextId, GoudError> {
        self.inner
            .write()
            .map_err(|_| GoudError::InternalError("Failed to acquire registry lock".to_string()))?
            .create()
    }

    /// Destroys a context.
    pub fn destroy(&self, id: GoudContextId) -> Result<(), GoudError> {
        self.inner
            .write()
            .map_err(|_| GoudError::InternalError("Failed to acquire registry lock".to_string()))?
            .destroy(id)
    }

    /// Validates that a context exists.
    pub fn is_valid(&self, id: GoudContextId) -> bool {
        self.inner
            .read()
            .map(|registry| registry.is_valid(id))
            .unwrap_or(false)
    }

    /// Returns the number of active contexts.
    pub fn len(&self) -> usize {
        self.inner.read().map(|r| r.len()).unwrap_or(0)
    }

    /// Returns true if there are no active contexts.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for GoudContextHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for GoudContextHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(registry) = self.inner.read() {
            write!(f, "GoudContextHandle({:?})", *registry)
        } else {
            write!(f, "GoudContextHandle(<locked>)")
        }
    }
}

// SAFETY: GoudContextHandle is Send despite containing non-Send World because:
// 1. The RwLock provides synchronization for all access
// 2. World operations are always performed while holding the lock
// 3. Each context is single-threaded (thread ownership is validated)
// 4. Only the handle itself crosses thread boundaries, not the World
unsafe impl Send for GoudContextHandle {}

// =============================================================================
// Global Registry
// =============================================================================

/// Global context registry for engine access.
///
/// This is the single source of truth for all engine contexts.
/// It's thread-safe and can be accessed from any thread, but individual
/// contexts are single-threaded.
static GOUD_CONTEXT_REGISTRY_CELL: OnceLock<Mutex<GoudContextRegistry>> = OnceLock::new();

/// Gets a reference to the global context registry.
///
/// Initializes the registry on first access.
pub fn get_context_registry() -> &'static Mutex<GoudContextRegistry> {
    GOUD_CONTEXT_REGISTRY_CELL.get_or_init(|| Mutex::new(GoudContextRegistry::new()))
}
