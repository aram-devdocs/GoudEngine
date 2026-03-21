//! Entity pool operations on the World.
//!
//! Provides methods to create, acquire from, release to, and destroy
//! entity pools. Pooled entities are pre-spawned and recycled to avoid
//! the overhead of repeated spawn/despawn cycles.

use crate::core::pool::{EntityPool, PoolStats};
use crate::ecs::components::PoolMember;
use crate::ecs::entity::Entity;
use std::collections::HashMap;

use super::World;

/// Pool registry stored as a World resource.
///
/// Manages all entity pools created in the world. Each pool is identified
/// by a unique `u32` handle.
#[derive(Debug, Default)]
pub struct EntityPoolRegistry {
    pools: HashMap<u32, EntityPool>,
    /// Maps entity ID (u64 bits) to (pool_handle, slot_index) for fast release.
    entity_to_slot: HashMap<u64, (u32, usize)>,
    next_handle: u32,
}

impl World {
    /// Creates an entity pool with pre-spawned entities.
    ///
    /// Pre-spawns `capacity` entities, marks each with a [`PoolMember`]
    /// component, and stores them in the pool's free list. All entities
    /// start in the available (free) state.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of entities to pre-spawn
    ///
    /// # Returns
    ///
    /// A pool handle that can be used with [`acquire_from_pool`](Self::acquire_from_pool),
    /// [`release_to_pool`](Self::release_to_pool), and other pool methods.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    /// let pool = world.create_entity_pool(10);
    ///
    /// let stats = world.pool_stats(pool).unwrap();
    /// assert_eq!(stats.capacity, 10);
    /// assert_eq!(stats.available, 10);
    /// assert_eq!(stats.active, 0);
    /// ```
    pub fn create_entity_pool(&mut self, capacity: usize) -> u32 {
        // Ensure registry resource exists
        if !self.contains_resource::<EntityPoolRegistry>() {
            self.insert_resource(EntityPoolRegistry::default());
        }

        // Allocate a handle
        let handle = {
            let registry = self
                .get_resource_mut::<EntityPoolRegistry>()
                .expect("Registry just inserted");
            let h = registry.next_handle;
            registry.next_handle = h + 1;
            h
        };

        // Pre-spawn entities
        let entities = self.spawn_batch(capacity);

        // Mark each entity with PoolMember
        for &entity in &entities {
            self.insert(entity, PoolMember::new(handle));
        }

        // Create pool from entity IDs (u64 bits)
        let entity_ids: Vec<u64> = entities.iter().map(|e| e.to_bits()).collect();
        let pool = EntityPool::from_entity_ids(entity_ids);

        // Build entity-to-slot reverse lookup
        let registry = self
            .get_resource_mut::<EntityPoolRegistry>()
            .expect("Registry exists");
        for (slot_idx, &entity) in entities.iter().enumerate() {
            registry
                .entity_to_slot
                .insert(entity.to_bits(), (handle, slot_idx));
        }
        registry.pools.insert(handle, pool);

        handle
    }

    /// Acquires an entity from a pool.
    ///
    /// The entity is already alive in the world. The caller should add
    /// components to configure the entity for its intended use.
    ///
    /// # Arguments
    ///
    /// * `pool_handle` - The pool to acquire from
    ///
    /// # Returns
    ///
    /// `Some(entity)` if an entity was available, `None` if the pool is
    /// exhausted or the handle is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    /// let pool = world.create_entity_pool(5);
    ///
    /// let entity = world.acquire_from_pool(pool).unwrap();
    /// assert!(world.is_alive(entity));
    /// ```
    pub fn acquire_from_pool(&mut self, pool_handle: u32) -> Option<Entity> {
        let registry = self.get_resource_mut::<EntityPoolRegistry>()?;
        let pool = registry.pools.get_mut(&pool_handle)?;
        let (_slot, entity_bits) = pool.acquire()?;
        let entity = Entity::from_bits(entity_bits);

        // Re-add PoolMember marker (reset_entity removes all components)
        self.insert(entity, PoolMember::new(pool_handle));

        Some(entity)
    }

    /// Releases an entity back to its pool.
    ///
    /// Strips all components from the entity (via [`reset_entity`](Self::reset_entity))
    /// and returns it to the pool's free list. The entity remains alive
    /// in the world but has no components.
    ///
    /// # Arguments
    ///
    /// * `pool_handle` - The pool to release to
    /// * `entity` - The entity to release
    ///
    /// # Returns
    ///
    /// `true` if the entity was successfully released, `false` if the
    /// entity is not alive, does not belong to this pool, or the handle
    /// is invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    /// let pool = world.create_entity_pool(3);
    ///
    /// let entity = world.acquire_from_pool(pool).unwrap();
    /// assert!(world.release_to_pool(pool, entity));
    ///
    /// let stats = world.pool_stats(pool).unwrap();
    /// assert_eq!(stats.available, 3);
    /// assert_eq!(stats.active, 0);
    /// ```
    pub fn release_to_pool(&mut self, pool_handle: u32, entity: Entity) -> bool {
        // Verify entity is alive
        if !self.is_alive(entity) {
            return false;
        }

        // Verify entity belongs to this pool
        match self.get::<PoolMember>(entity) {
            Some(member) if member.pool_id == pool_handle => {}
            _ => return false,
        }

        // Look up the slot index via the reverse map
        let entity_bits = entity.to_bits();
        let slot_index = {
            let registry = match self.get_resource::<EntityPoolRegistry>() {
                Some(r) => r,
                None => return false,
            };
            match registry.entity_to_slot.get(&entity_bits) {
                Some(&(ph, slot)) if ph == pool_handle => slot,
                _ => return false,
            }
        };

        // Reset the entity (remove all components including PoolMember)
        if !self.reset_entity(entity) {
            return false;
        }

        // Release slot back to pool (O(1))
        let registry = match self.get_resource_mut::<EntityPoolRegistry>() {
            Some(r) => r,
            None => return false,
        };
        let pool = match registry.pools.get_mut(&pool_handle) {
            Some(p) => p,
            None => return false,
        };

        pool.release(slot_index)
    }

    /// Strips all components from an entity.
    ///
    /// The entity remains alive and moves to the empty archetype.
    /// This is used by the pool system to recycle entities.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to reset
    ///
    /// # Returns
    ///
    /// `true` if the entity was alive and reset, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World, Component};
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// struct Health(i32);
    /// impl Component for Health {}
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn_empty();
    /// world.insert(entity, Health(100));
    /// assert!(world.has::<Health>(entity));
    ///
    /// world.reset_entity(entity);
    /// assert!(!world.has::<Health>(entity));
    /// assert!(world.is_alive(entity));
    /// ```
    pub fn reset_entity(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        // Get current archetype
        let archetype_id = match self.entity_archetypes.get(&entity).copied() {
            Some(id) => id,
            None => return false,
        };

        // Get component IDs from the entity's archetype
        let component_ids: Vec<_> = self
            .archetypes
            .get(archetype_id)
            .map(|arch| arch.components().iter().copied().collect())
            .unwrap_or_default();

        // If the entity already has no components, nothing to do
        if component_ids.is_empty() {
            return true;
        }

        // Remove each component from storage
        for component_id in &component_ids {
            if let Some(storage_entry) = self.storages.get_mut(component_id) {
                storage_entry.remove_entity(entity);
            }
        }

        // Move entity from current archetype to EMPTY archetype
        if let Some(old_arch) = self.archetypes.get_mut(archetype_id) {
            old_arch.remove_entity(entity);
        }

        use crate::ecs::archetype::ArchetypeId;
        if let Some(empty_arch) = self.archetypes.get_mut(ArchetypeId::EMPTY) {
            empty_arch.add_entity(entity);
        }

        self.entity_archetypes.insert(entity, ArchetypeId::EMPTY);

        true
    }

    /// Destroys a pool and optionally despawns its entities.
    ///
    /// # Arguments
    ///
    /// * `pool_handle` - The pool to destroy
    /// * `despawn_entities` - If `true`, despawn all entities in the pool.
    ///   If `false`, entities remain alive but are no longer tracked by a pool.
    ///
    /// # Returns
    ///
    /// `true` if the pool existed and was destroyed, `false` if the handle
    /// was invalid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    /// let pool = world.create_entity_pool(5);
    /// assert_eq!(world.entity_count(), 5);
    ///
    /// world.destroy_entity_pool(pool, true);
    /// assert_eq!(world.entity_count(), 0);
    /// ```
    pub fn destroy_entity_pool(&mut self, pool_handle: u32, despawn_entities: bool) -> bool {
        // Extract the pool from the registry
        let (pool, entity_ids_to_clean) = {
            let registry = match self.get_resource_mut::<EntityPoolRegistry>() {
                Some(r) => r,
                None => return false,
            };
            let pool = match registry.pools.remove(&pool_handle) {
                Some(p) => p,
                None => return false,
            };
            // Collect entity IDs to remove from reverse map
            let ids = pool.all_entity_ids();
            for &id in &ids {
                registry.entity_to_slot.remove(&id);
            }
            (pool, ids)
        };

        if despawn_entities {
            for &entity_bits in &entity_ids_to_clean {
                let entity = Entity::from_bits(entity_bits);
                self.despawn(entity);
            }
        }

        drop(pool);
        true
    }

    /// Gets pool statistics.
    ///
    /// # Arguments
    ///
    /// * `pool_handle` - The pool to query
    ///
    /// # Returns
    ///
    /// `Some(PoolStats)` if the pool exists, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::World;
    ///
    /// let mut world = World::new();
    /// let pool = world.create_entity_pool(10);
    ///
    /// let stats = world.pool_stats(pool).unwrap();
    /// assert_eq!(stats.capacity, 10);
    /// assert_eq!(stats.available, 10);
    /// ```
    pub fn pool_stats(&self, pool_handle: u32) -> Option<PoolStats> {
        let registry = self.get_resource::<EntityPoolRegistry>()?;
        let pool = registry.pools.get(&pool_handle)?;
        Some(*pool.stats())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::Component;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Health(i32);
    impl Component for Health {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Speed(f32);
    impl Component for Speed {}

    #[test]
    fn test_create_pool() {
        let mut world = World::new();
        let pool = world.create_entity_pool(10);

        let stats = world.pool_stats(pool).unwrap();
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.available, 10);
        assert_eq!(stats.active, 0);
        assert_eq!(world.entity_count(), 10);
    }

    #[test]
    fn test_create_pool_zero_capacity() {
        let mut world = World::new();
        let pool = world.create_entity_pool(0);

        let stats = world.pool_stats(pool).unwrap();
        assert_eq!(stats.capacity, 0);
        assert_eq!(stats.available, 0);
        assert_eq!(stats.active, 0);
    }

    #[test]
    fn test_acquire_from_pool() {
        let mut world = World::new();
        let pool = world.create_entity_pool(5);

        let entity = world.acquire_from_pool(pool).unwrap();
        assert!(world.is_alive(entity));
        assert!(world.has::<PoolMember>(entity));

        let stats = world.pool_stats(pool).unwrap();
        assert_eq!(stats.active, 1);
        assert_eq!(stats.available, 4);
    }

    #[test]
    fn test_acquire_then_add_components() {
        let mut world = World::new();
        let pool = world.create_entity_pool(3);

        let entity = world.acquire_from_pool(pool).unwrap();
        world.insert(entity, Health(100));
        world.insert(entity, Speed(5.0));

        assert_eq!(world.get::<Health>(entity), Some(&Health(100)));
        assert_eq!(world.get::<Speed>(entity), Some(&Speed(5.0)));
    }

    #[test]
    fn test_release_to_pool() {
        let mut world = World::new();
        let pool = world.create_entity_pool(3);

        let entity = world.acquire_from_pool(pool).unwrap();
        world.insert(entity, Health(100));
        world.insert(entity, Speed(5.0));

        assert!(world.release_to_pool(pool, entity));

        // Entity should be alive but have no components
        assert!(world.is_alive(entity));
        assert!(!world.has::<Health>(entity));
        assert!(!world.has::<Speed>(entity));
        assert!(!world.has::<PoolMember>(entity));

        let stats = world.pool_stats(pool).unwrap();
        assert_eq!(stats.active, 0);
        assert_eq!(stats.available, 3);
    }

    #[test]
    fn test_acquire_release_reacquire() {
        let mut world = World::new();
        let pool = world.create_entity_pool(1);

        let entity1 = world.acquire_from_pool(pool).unwrap();
        world.insert(entity1, Health(100));

        // Pool should be exhausted
        assert!(world.acquire_from_pool(pool).is_none());

        // Release it
        assert!(world.release_to_pool(pool, entity1));

        // Should be able to acquire again
        let entity2 = world.acquire_from_pool(pool).unwrap();
        assert_eq!(entity1, entity2);
        assert!(!world.has::<Health>(entity2));
        assert!(world.has::<PoolMember>(entity2));
    }

    #[test]
    fn test_pool_exhaustion() {
        let mut world = World::new();
        let pool = world.create_entity_pool(2);

        let _e1 = world.acquire_from_pool(pool).unwrap();
        let _e2 = world.acquire_from_pool(pool).unwrap();

        assert!(world.acquire_from_pool(pool).is_none());

        let stats = world.pool_stats(pool).unwrap();
        assert_eq!(stats.active, 2);
        assert_eq!(stats.available, 0);
    }

    #[test]
    fn test_reset_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        world.insert(entity, Health(50));
        world.insert(entity, Speed(3.0));

        assert!(world.has::<Health>(entity));
        assert!(world.has::<Speed>(entity));

        assert!(world.reset_entity(entity));

        assert!(world.is_alive(entity));
        assert!(!world.has::<Health>(entity));
        assert!(!world.has::<Speed>(entity));
    }

    #[test]
    fn test_reset_entity_already_empty() {
        let mut world = World::new();
        let entity = world.spawn_empty();

        assert!(world.reset_entity(entity));
        assert!(world.is_alive(entity));
    }

    #[test]
    fn test_reset_entity_dead() {
        let mut world = World::new();
        let entity = world.spawn_empty();
        world.despawn(entity);

        assert!(!world.reset_entity(entity));
    }

    #[test]
    fn test_destroy_pool_with_despawn() {
        let mut world = World::new();
        let pool = world.create_entity_pool(5);
        assert_eq!(world.entity_count(), 5);

        assert!(world.destroy_entity_pool(pool, true));
        assert_eq!(world.entity_count(), 0);
        assert!(world.pool_stats(pool).is_none());
    }

    #[test]
    fn test_destroy_pool_without_despawn() {
        let mut world = World::new();
        let pool = world.create_entity_pool(5);

        assert!(world.destroy_entity_pool(pool, false));
        assert_eq!(world.entity_count(), 5);
        assert!(world.pool_stats(pool).is_none());
    }

    #[test]
    fn test_destroy_invalid_pool() {
        let mut world = World::new();
        assert!(!world.destroy_entity_pool(999, true));
    }

    #[test]
    fn test_pool_stats_invalid_handle() {
        let world = World::new();
        assert!(world.pool_stats(42).is_none());
    }

    #[test]
    fn test_release_wrong_pool() {
        let mut world = World::new();
        let pool_a = world.create_entity_pool(3);
        let pool_b = world.create_entity_pool(3);

        let entity = world.acquire_from_pool(pool_a).unwrap();

        assert!(!world.release_to_pool(pool_b, entity));
    }

    #[test]
    fn test_release_non_pooled_entity() {
        let mut world = World::new();
        let pool = world.create_entity_pool(3);
        let free_entity = world.spawn_empty();

        assert!(!world.release_to_pool(pool, free_entity));
    }

    #[test]
    fn test_multiple_pools() {
        let mut world = World::new();
        let pool_a = world.create_entity_pool(3);
        let pool_b = world.create_entity_pool(5);

        let stats_a = world.pool_stats(pool_a).unwrap();
        let stats_b = world.pool_stats(pool_b).unwrap();
        assert_eq!(stats_a.capacity, 3);
        assert_eq!(stats_b.capacity, 5);

        let ea = world.acquire_from_pool(pool_a).unwrap();
        let eb = world.acquire_from_pool(pool_b).unwrap();

        assert_ne!(ea, eb);

        let stats_a = world.pool_stats(pool_a).unwrap();
        let stats_b = world.pool_stats(pool_b).unwrap();
        assert_eq!(stats_a.active, 1);
        assert_eq!(stats_b.active, 1);
    }

    #[test]
    fn test_acquire_from_invalid_pool() {
        let mut world = World::new();
        assert!(world.acquire_from_pool(999).is_none());
    }
}
