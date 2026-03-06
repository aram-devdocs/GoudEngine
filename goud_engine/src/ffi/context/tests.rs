//! # Context FFI Tests
//!
//! Tests for context ID, registry, handle, and integration scenarios.

#[cfg(test)]
mod tests {
    use crate::core::context_registry::{
        GoudContext, GoudContextHandle, GoudContextId, GoudContextRegistry, GOUD_INVALID_CONTEXT_ID,
    };

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
        assert_eq!(format!("{}", id), "GoudContextId(10:3)");

        let invalid = GOUD_INVALID_CONTEXT_ID;
        assert_eq!(format!("{}", invalid), "GoudContextId(INVALID)");
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
        let _id1 = registry.create().unwrap();
        let id2 = registry.create().unwrap();
        let _id3 = registry.create().unwrap();

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

        let debug = format!("{:?}", registry);
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
        let debug = format!("{:?}", context);
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

        let debug = format!("{:?}", handle);
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
