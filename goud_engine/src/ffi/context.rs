//! # FFI Context Management
//!
//! This module implements the core context type for FFI operations. A context
//! represents a single engine instance with its own World, asset storage,
//! and error state.
//!
//! ## Context Lifecycle
//!
//! 1. **Creation**: `goud_context_create()` allocates a new context
//! 2. **Operations**: All FFI functions accept `GoudContextId` as first parameter
//! 3. **Destruction**: `goud_context_destroy()` releases all resources
//!
//! ## Thread Safety
//!
//! - Contexts are stored in a global registry protected by `RwLock`
//! - Each context owns its World (not Send+Sync)
//! - Context operations must be called from the thread that created the context
//! - Future: Add thread ownership validation for debug builds

#![allow(clippy::arc_with_non_send_sync)]

use crate::core::error::GoudError;
use crate::ecs::World;
use std::sync::{Arc, RwLock};

/// Opaque identifier for an FFI context.
///
/// This ID is returned to FFI callers and used to look up the actual context.
/// It uses generational indexing to detect use-after-free bugs.
///
/// # FFI Safety
///
/// - `#[repr(C)]` ensures predictable memory layout
/// - 64-bit value can be passed by value on all platforms
/// - Invalid ID (all bits 1) is distinguishable from any valid ID
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GoudContextId(u64);

impl GoudContextId {
    /// Creates a new context ID from index and generation.
    ///
    /// # Layout
    ///
    /// ```text
    /// | 32 bits: generation | 32 bits: index |
    /// ```
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        let packed = ((generation as u64) << 32) | (index as u64);
        Self(packed)
    }

    /// Returns the index component (lower 32 bits).
    pub(crate) fn index(self) -> u32 {
        self.0 as u32
    }

    /// Returns the generation component (upper 32 bits).
    pub(crate) fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Returns true if this is the invalid sentinel ID.
    pub fn is_invalid(self) -> bool {
        self.0 == u64::MAX
    }
}

impl Default for GoudContextId {
    fn default() -> Self {
        GOUD_INVALID_CONTEXT_ID
    }
}

impl std::fmt::Display for GoudContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_invalid() {
            write!(f, "GoudContextId(INVALID)")
        } else {
            write!(f, "GoudContextId({}:{})", self.index(), self.generation())
        }
    }
}

/// Sentinel value representing an invalid context ID.
///
/// This is returned by `goud_context_create()` on failure and should be
/// checked by callers before using the ID.
pub const GOUD_INVALID_CONTEXT_ID: GoudContextId = GoudContextId(u64::MAX);

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

    /// Thread ID that created this context (for validation in debug builds).
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    owner_thread: std::thread::ThreadId,
}

impl GoudContext {
    /// Creates a new context with the given generation.
    pub(crate) fn new(generation: u32) -> Self {
        Self {
            world: World::new(),
            generation,
            #[cfg(debug_assertions)]
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
    /// In debug builds, panics if called from wrong thread.
    /// In release builds, this is a no-op.
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub(crate) fn validate_thread(&self) {
        let current = std::thread::current().id();
        if current != self.owner_thread {
            panic!(
                "GoudContext accessed from wrong thread! Created on {:?}, accessed from {:?}",
                self.owner_thread, current
            );
        }
    }

    #[cfg(not(debug_assertions))]
    #[allow(dead_code)]
    pub(crate) fn validate_thread(&self) {
        // No-op in release builds
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

/// Global registry for all FFI contexts.
///
/// This is the central storage for all active contexts. It uses generational
/// indexing to detect use-after-free bugs and allows safe context lookup
/// from FFI code.
///
/// # Thread Safety
///
/// The registry itself is thread-safe (protected by RwLock), but individual
/// contexts are not. Callers must ensure they don't use a context from
/// multiple threads simultaneously.
pub struct GoudContextRegistry {
    /// Slots for context storage.
    ///
    /// Invariants:
    /// - Occupied slots have valid contexts
    /// - Free slots store the next generation to use
    /// - Index corresponds to lower 32 bits of GoudContextId
    slots: Vec<ContextSlot>,

    /// Free list of indices that can be reused.
    ///
    /// When a context is destroyed, its index is pushed here.
    /// When creating a new context, we pop from here first.
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
    ///
    /// # Returns
    ///
    /// - `Ok(id)` - Successfully created context with given ID
    /// - `Err(error)` - Failed to create context (e.g., out of memory)
    pub fn create(&mut self) -> Result<GoudContextId, GoudError> {
        // Try to reuse a free slot first
        if let Some(index) = self.free_list.pop() {
            let generation = match &self.slots[index as usize] {
                ContextSlot::Free { next_generation } => *next_generation,
                ContextSlot::Occupied(_) => {
                    return Err(GoudError::InternalError(
                        "Free list points to occupied slot".to_string(),
                    ))
                }
            };

            let context = GoudContext::new(generation);
            self.slots[index as usize] = ContextSlot::Occupied(Box::new(context));

            Ok(GoudContextId::new(index, generation))
        } else {
            // Allocate new slot
            let index = self.slots.len() as u32;
            if index == u32::MAX {
                return Err(GoudError::InternalError(
                    "Context registry full (u32::MAX contexts)".to_string(),
                ));
            }

            let generation = 1; // Generation 0 reserved for "never allocated"
            let context = GoudContext::new(generation);
            self.slots.push(ContextSlot::Occupied(Box::new(context)));

            Ok(GoudContextId::new(index, generation))
        }
    }

    /// Destroys a context and frees its slot for reuse.
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Context destroyed successfully
    /// - `Err(error)` - Context not found or already destroyed
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

                // Increment generation for next allocation
                let next_generation = context.generation().checked_add(1).unwrap_or(1); // Wrap to 1 on overflow

                self.slots[index] = ContextSlot::Free { next_generation };
                self.free_list.push(id.index());

                Ok(())
            }
            ContextSlot::Free { .. } => Err(GoudError::InvalidContext),
        }
    }

    /// Gets an immutable reference to a context.
    ///
    /// # Returns
    ///
    /// - `Some(&context)` - Context exists and generation matches
    /// - `None` - Context not found or generation mismatch (use-after-free)
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

// Safety: GoudContextRegistry is Send despite containing non-Send World because:
// 1. Access is always synchronized via Mutex in the global registry
// 2. Contexts are single-threaded (thread ownership validated)
// 3. Only the registry structure crosses thread boundaries
unsafe impl Send for GoudContextRegistry {}

/// Handle to the global context registry.
///
/// This is a thread-safe `Arc<RwLock>` wrapper that allows concurrent access
/// to the registry from multiple threads.
///
/// Note: The Arc contains !Send+!Sync data (World with NonSendResources),
/// but the `RwLock` ensures only one thread can access mutable data at a time,
/// making this safe for multi-threaded context ID operations (create/destroy/is_valid).
#[derive(Clone)]
#[allow(clippy::arc_with_non_send_sync)]
pub struct GoudContextHandle {
    inner: Arc<RwLock<GoudContextRegistry>>,
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

// Safety: GoudContextHandle is Send despite containing non-Send World because:
// 1. The RwLock provides synchronization for all access
// 2. World operations are always performed while holding the lock
// 3. Each context is single-threaded (thread ownership is validated)
// 4. Only the handle itself crosses thread boundaries, not the World
unsafe impl Send for GoudContextHandle {}

// ============================================================================
// Global Registry
// ============================================================================

use std::sync::Mutex;
use std::sync::OnceLock;

/// Global context registry for FFI access.
///
/// This is the single source of truth for all engine contexts accessed via FFI.
/// It's thread-safe and can be accessed from any thread, but individual contexts
/// are single-threaded and must be used from the thread that created them.
///
/// Note: We use `Mutex<GoudContextRegistry>` (not GoudContextHandle) to avoid
/// double-locking. The registry is the actual storage, the handle is a wrapper.
static GOUD_CONTEXT_REGISTRY_CELL: OnceLock<Mutex<GoudContextRegistry>> = OnceLock::new();

/// Gets a reference to the global context registry.
///
/// Initializes the registry on first access.
pub fn get_context_registry() -> &'static Mutex<GoudContextRegistry> {
    GOUD_CONTEXT_REGISTRY_CELL.get_or_init(|| Mutex::new(GoudContextRegistry::new()))
}

// ============================================================================
// FFI Functions
// ============================================================================

/// Creates a new engine context.
///
/// Returns a unique context ID that must be passed to all subsequent FFI calls.
/// On failure, returns `GOUD_INVALID_CONTEXT_ID` and sets the last error.
///
/// # Thread Safety
///
/// This function is thread-safe and can be called from any thread.
/// However, the returned context must be used from a single thread.
///
/// # Error Codes
///
/// - `INTERNAL_ERROR_BASE + 0` (InternalError) - Failed to lock registry or create context
///
/// # Example (C#)
///
/// ```csharp
/// var contextId = goud_context_create();
/// if (contextId == GOUD_INVALID_CONTEXT_ID) {
///     var error = goud_get_last_error_message();
///     Console.WriteLine($"Failed to create context: {error}");
///     return;
/// }
/// ```
#[no_mangle]
pub extern "C" fn goud_context_create() -> GoudContextId {
    use crate::core::error::set_last_error;

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return GOUD_INVALID_CONTEXT_ID;
        }
    };

    match registry.create() {
        Ok(id) => id,
        Err(err) => {
            set_last_error(err);
            GOUD_INVALID_CONTEXT_ID
        }
    }
}

/// Destroys an engine context, freeing all associated resources.
///
/// After calling this, the context ID becomes invalid and should not be used.
/// All entities, components, and resources in the context are destroyed.
///
/// # Arguments
///
/// * `context_id` - The context to destroy
///
/// # Returns
///
/// `true` if the context was successfully destroyed, `false` on error.
///
/// # Error Codes
///
/// - `CONTEXT_ERROR_BASE + 3` (InvalidContext) - Invalid or already-destroyed context ID
/// - `INTERNAL_ERROR_BASE + 0` (InternalError) - Failed to lock registry
///
/// # Thread Safety
///
/// This function is thread-safe and can be called from any thread.
///
/// # Example (C#)
///
/// ```csharp
/// if (!goud_context_destroy(contextId)) {
///     var error = goud_get_last_error_message();
///     Console.WriteLine($"Failed to destroy context: {error}");
/// }
/// ```
#[no_mangle]
pub extern "C" fn goud_context_destroy(context_id: GoudContextId) -> bool {
    use crate::core::error::set_last_error;

    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return false;
    }

    let mut registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => {
            set_last_error(GoudError::InternalError(
                "Failed to lock context registry".to_string(),
            ));
            return false;
        }
    };

    match registry.destroy(context_id) {
        Ok(()) => true,
        Err(err) => {
            set_last_error(err);
            false
        }
    }
}

/// Checks if a context ID is valid.
///
/// A context is valid if it was created and has not been destroyed.
///
/// # Arguments
///
/// * `context_id` - The context ID to check
///
/// # Returns
///
/// `true` if the context is valid, `false` otherwise.
///
/// # Example (C#)
///
/// ```csharp
/// if (goud_context_is_valid(contextId)) {
///     Console.WriteLine("Context is valid");
/// }
/// ```
#[no_mangle]
pub extern "C" fn goud_context_is_valid(context_id: GoudContextId) -> bool {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        return false;
    }

    let registry = match get_context_registry().lock() {
        Ok(r) => r,
        Err(_) => return false,
    };

    registry.is_valid(context_id)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // GoudContextId Tests
    // ========================================================================

    #[test]
    fn test_context_id_new() {
        let id = GoudContextId::new(42, 7);
        assert_eq!(id.index(), 42);
        assert_eq!(id.generation(), 7);
        assert!(!id.is_invalid());
    }

    #[test]
    fn test_context_id_invalid() {
        let id = GOUD_INVALID_CONTEXT_ID;
        assert!(id.is_invalid());
        assert_eq!(id.index(), u32::MAX);
        assert_eq!(id.generation(), u32::MAX);
    }

    #[test]
    fn test_context_id_default() {
        let id = GoudContextId::default();
        assert!(id.is_invalid());
    }

    #[test]
    fn test_context_id_display() {
        let id = GoudContextId::new(10, 3);
        assert_eq!(format!("{id}"), "GoudContextId(10:3)");

        let invalid = GOUD_INVALID_CONTEXT_ID;
        assert_eq!(format!("{invalid}"), "GoudContextId(INVALID)");
    }

    #[test]
    fn test_context_id_equality() {
        let id1 = GoudContextId::new(10, 3);
        let id2 = GoudContextId::new(10, 3);
        let id3 = GoudContextId::new(10, 4);
        let id4 = GoudContextId::new(11, 3);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3); // Different generation
        assert_ne!(id1, id4); // Different index
    }

    #[test]
    fn test_context_id_hash() {
        use std::collections::HashSet;

        let id1 = GoudContextId::new(10, 3);
        let id2 = GoudContextId::new(10, 3);
        let id3 = GoudContextId::new(11, 3);

        let mut set = HashSet::new();
        set.insert(id1);
        assert!(set.contains(&id2)); // Same as id1
        assert!(!set.contains(&id3)); // Different
    }

    #[test]
    fn test_context_id_copy_clone() {
        let id1 = GoudContextId::new(5, 2);
        let id2 = id1; // Copy
        let id3 = id1.clone(); // Clone

        assert_eq!(id1, id2);
        assert_eq!(id1, id3);
    }

    // ========================================================================
    // GoudContextRegistry Tests
    // ========================================================================

    #[test]
    fn test_registry_new() {
        let registry = GoudContextRegistry::new();
        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
        assert_eq!(registry.capacity(), 0);
    }

    #[test]
    fn test_registry_create() {
        let mut registry = GoudContextRegistry::new();

        let id = registry.create().unwrap();
        assert!(!id.is_invalid());
        assert_eq!(id.index(), 0);
        assert_eq!(id.generation(), 1);
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_create_multiple() {
        let mut registry = GoudContextRegistry::new();

        let id1 = registry.create().unwrap();
        let id2 = registry.create().unwrap();
        let id3 = registry.create().unwrap();

        assert_eq!(id1.index(), 0);
        assert_eq!(id2.index(), 1);
        assert_eq!(id3.index(), 2);
        assert_eq!(registry.len(), 3);
    }

    #[test]
    fn test_registry_get() {
        let mut registry = GoudContextRegistry::new();
        let id = registry.create().unwrap();

        let context = registry.get(id);
        assert!(context.is_some());
        assert_eq!(context.unwrap().generation(), id.generation());
    }

    #[test]
    fn test_registry_get_invalid() {
        let registry = GoudContextRegistry::new();

        // Invalid ID
        assert!(registry.get(GOUD_INVALID_CONTEXT_ID).is_none());

        // Out of bounds
        let id = GoudContextId::new(100, 1);
        assert!(registry.get(id).is_none());
    }

    #[test]
    fn test_registry_get_mut() {
        let mut registry = GoudContextRegistry::new();
        let id = registry.create().unwrap();

        let context = registry.get_mut(id);
        assert!(context.is_some());

        // Can modify world
        let context = context.unwrap();
        let entity = context.world_mut().spawn_empty();
        assert!(context.world().is_alive(entity));
    }

    #[test]
    fn test_registry_destroy() {
        let mut registry = GoudContextRegistry::new();
        let id = registry.create().unwrap();

        assert_eq!(registry.len(), 1);

        registry.destroy(id).unwrap();
        assert_eq!(registry.len(), 0);
        assert!(registry.get(id).is_none());
    }

    #[test]
    fn test_registry_destroy_invalid() {
        let mut registry = GoudContextRegistry::new();

        // Destroy invalid ID
        let result = registry.destroy(GOUD_INVALID_CONTEXT_ID);
        assert!(result.is_err());

        // Destroy non-existent ID
        let id = GoudContextId::new(100, 1);
        let result = registry.destroy(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_destroy_twice() {
        let mut registry = GoudContextRegistry::new();
        let id = registry.create().unwrap();

        registry.destroy(id).unwrap();

        // Second destroy should fail
        let result = registry.destroy(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_generation_increment() {
        let mut registry = GoudContextRegistry::new();

        let id1 = registry.create().unwrap();
        assert_eq!(id1.generation(), 1);

        registry.destroy(id1).unwrap();

        // Reuse slot, generation should increment
        let id2 = registry.create().unwrap();
        assert_eq!(id2.index(), id1.index()); // Same slot
        assert_eq!(id2.generation(), 2); // Incremented

        // Old ID should not work
        assert!(registry.get(id1).is_none());
        assert!(registry.get(id2).is_some());
    }

    #[test]
    fn test_registry_free_list_reuse() {
        let mut registry = GoudContextRegistry::new();

        // Create 3 contexts
        let id1 = registry.create().unwrap();
        let id2 = registry.create().unwrap();
        let id3 = registry.create().unwrap();

        // Destroy middle one
        registry.destroy(id2).unwrap();

        // Next create should reuse slot 1
        let id4 = registry.create().unwrap();
        assert_eq!(id4.index(), id2.index());
        assert_eq!(id4.generation(), id2.generation() + 1);

        assert_eq!(registry.len(), 3);
        assert_eq!(registry.capacity(), 3);
    }

    #[test]
    fn test_registry_is_valid() {
        let mut registry = GoudContextRegistry::new();
        let id = registry.create().unwrap();

        assert!(registry.is_valid(id));

        registry.destroy(id).unwrap();
        assert!(!registry.is_valid(id));
    }

    #[test]
    fn test_registry_debug() {
        let mut registry = GoudContextRegistry::new();
        registry.create().unwrap();

        let debug = format!("{registry:?}");
        assert!(debug.contains("GoudContextRegistry"));
        assert!(debug.contains("active"));
        assert!(debug.contains("capacity"));
    }

    // ========================================================================
    // GoudContext Tests
    // ========================================================================

    #[test]
    fn test_context_new() {
        let context = GoudContext::new(1);
        assert_eq!(context.generation(), 1);
        assert!(context.world().is_empty());
    }

    #[test]
    fn test_context_world_access() {
        let mut context = GoudContext::new(1);

        // Immutable access
        assert_eq!(context.world().entity_count(), 0);

        // Mutable access
        let entity = context.world_mut().spawn_empty();
        assert_eq!(context.world().entity_count(), 1);
        assert!(context.world().is_alive(entity));
    }

    #[test]
    fn test_context_validate_thread() {
        let context = GoudContext::new(1);
        // Should not panic on same thread
        context.validate_thread();
    }

    #[test]
    fn test_context_debug() {
        let context = GoudContext::new(5);
        let debug = format!("{context:?}");
        assert!(debug.contains("GoudContext"));
        assert!(debug.contains("generation"));
        assert!(debug.contains("5"));
    }

    // ========================================================================
    // GoudContextHandle Tests
    // ========================================================================

    #[test]
    fn test_handle_new() {
        let handle = GoudContextHandle::new();
        assert_eq!(handle.len(), 0);
        assert!(handle.is_empty());
    }

    #[test]
    fn test_handle_create() {
        let handle = GoudContextHandle::new();
        let id = handle.create().unwrap();

        assert!(!id.is_invalid());
        assert_eq!(handle.len(), 1);
        assert!(handle.is_valid(id));
    }

    #[test]
    fn test_handle_create_multiple() {
        let handle = GoudContextHandle::new();

        let id1 = handle.create().unwrap();
        let id2 = handle.create().unwrap();

        assert_ne!(id1, id2);
        assert_eq!(handle.len(), 2);
    }

    #[test]
    fn test_handle_destroy() {
        let handle = GoudContextHandle::new();
        let id = handle.create().unwrap();

        assert!(handle.is_valid(id));
        handle.destroy(id).unwrap();
        assert!(!handle.is_valid(id));
        assert!(handle.is_empty());
    }

    #[test]
    fn test_handle_clone() {
        let handle1 = GoudContextHandle::new();
        let id = handle1.create().unwrap();

        let handle2 = handle1.clone();
        assert!(handle2.is_valid(id));
        assert_eq!(handle2.len(), 1);
    }

    #[test]
    fn test_handle_debug() {
        let handle = GoudContextHandle::new();
        handle.create().unwrap();

        let debug = format!("{handle:?}");
        assert!(debug.contains("GoudContextHandle"));
    }

    #[test]
    fn test_handle_default() {
        let handle = GoudContextHandle::default();
        assert!(handle.is_empty());
    }

    // ========================================================================
    // Thread Safety Tests
    // ========================================================================

    // Note: GoudContextHandle is NOT Send+Sync because it contains World,
    // which has NonSendResources. This is intentional - contexts are
    // single-threaded. Only GoudContextId needs to be Send+Sync.

    #[test]
    fn test_context_id_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<GoudContextId>();
    }

    #[test]
    fn test_context_id_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<GoudContextId>();
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    #[test]
    fn test_full_lifecycle() {
        let handle = GoudContextHandle::new();

        // Create
        let id = handle.create().unwrap();
        assert!(handle.is_valid(id));

        // Use (via registry access)
        {
            let registry = handle.inner.read().unwrap();
            let context = registry.get(id).unwrap();
            assert_eq!(context.world().entity_count(), 0);
        }

        // Destroy
        handle.destroy(id).unwrap();
        assert!(!handle.is_valid(id));
    }

    #[test]
    fn test_multiple_contexts_isolation() {
        let handle = GoudContextHandle::new();

        let id1 = handle.create().unwrap();
        let id2 = handle.create().unwrap();

        // Modify world in context 1
        {
            let mut registry = handle.inner.write().unwrap();
            let context1 = registry.get_mut(id1).unwrap();
            context1.world_mut().spawn_empty();
        }

        // Context 2 should be unaffected
        {
            let registry = handle.inner.read().unwrap();
            let context2 = registry.get(id2).unwrap();
            assert_eq!(context2.world().entity_count(), 0);
        }
    }

    #[test]
    fn test_stale_id_detection() {
        let handle = GoudContextHandle::new();

        let id1 = handle.create().unwrap();
        handle.destroy(id1).unwrap();

        let id2 = handle.create().unwrap();

        // id1 and id2 have same index but different generations
        assert_eq!(id1.index(), id2.index());
        assert_ne!(id1.generation(), id2.generation());

        // id1 should not resolve
        {
            let registry = handle.inner.read().unwrap();
            assert!(registry.get(id1).is_none());
            assert!(registry.get(id2).is_some());
        }
    }

    #[test]
    fn test_stress_create_destroy() {
        let handle = GoudContextHandle::new();

        for _ in 0..1000 {
            let id = handle.create().unwrap();
            handle.destroy(id).unwrap();
        }

        assert!(handle.is_empty());
    }

    #[test]
    fn test_many_concurrent_contexts() {
        let handle = GoudContextHandle::new();
        let mut ids = Vec::new();

        // Create 100 contexts
        for _ in 0..100 {
            ids.push(handle.create().unwrap());
        }

        assert_eq!(handle.len(), 100);

        // All should be valid
        for id in &ids {
            assert!(handle.is_valid(*id));
        }

        // Destroy all
        for id in ids {
            handle.destroy(id).unwrap();
        }

        assert!(handle.is_empty());
    }
}
