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
        let handle = match self.get_resource_mut::<EntityPoolRegistry>() {
            Some(registry) => {
                let h = registry.next_handle;
                registry.next_handle = h + 1;
                h
            }
            None => return u32::MAX,
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
        let registry = match self.get_resource_mut::<EntityPoolRegistry>() {
            Some(r) => r,
            None => return u32::MAX,
        };
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
