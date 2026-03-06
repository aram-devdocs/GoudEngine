//! Bulk entity allocation and deallocation operations.
//!
//! These methods extend [`EntityAllocator`] with batch operations that are
//! more efficient than repeated single-entity calls when operating on many
//! entities at once.

use super::allocator::EntityAllocator;
use super::types::Entity;

impl EntityAllocator {
    /// Allocates multiple entities at once.
    ///
    /// This is more efficient than calling [`allocate`](Self::allocate) in a loop
    /// because it pre-allocates the result vector and minimizes reallocations.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of entities to allocate
    ///
    /// # Returns
    ///
    /// A vector containing `count` newly allocated entities. All entities are
    /// guaranteed to be valid and unique.
    ///
    /// # Panics
    ///
    /// Panics if allocating `count` entities would exceed `u32::MAX - 1` total slots.
    ///
    /// # Performance
    ///
    /// For large batch allocations, this method:
    /// - Pre-allocates the result vector with exact capacity
    /// - Reuses free slots first (LIFO order)
    /// - Bulk-extends the generations vector for remaining slots
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    ///
    /// // Allocate 1000 entities in one call
    /// let entities = allocator.allocate_batch(1000);
    /// assert_eq!(entities.len(), 1000);
    /// assert_eq!(allocator.len(), 1000);
    ///
    /// // All entities are unique and alive
    /// for entity in &entities {
    ///     assert!(allocator.is_alive(*entity));
    /// }
    /// ```
    pub fn allocate_batch(&mut self, count: usize) -> Vec<Entity> {
        if count == 0 {
            return Vec::new();
        }

        let mut entities = Vec::with_capacity(count);

        // First, use up free slots
        let from_free_list = count.min(self.free_list.len());
        for _ in 0..from_free_list {
            if let Some(index) = self.free_list.pop() {
                let generation = self.generations[index as usize];
                entities.push(Entity::new(index, generation));
            }
        }

        // Then, allocate new slots for remaining count
        let remaining = count - from_free_list;
        if remaining > 0 {
            let start_index = self.generations.len();
            let end_index = start_index + remaining;

            // Ensure we don't exceed maximum capacity
            assert!(
                end_index <= u32::MAX as usize,
                "EntityAllocator would exceed maximum capacity"
            );

            // Bulk-extend generations vector with initial generation of 1
            self.generations.resize(end_index, 1);

            // Create entities for new slots
            for index in start_index..end_index {
                entities.push(Entity::new(index as u32, 1));
            }
        }

        entities
    }

    /// Deallocates multiple entities at once.
    ///
    /// This method attempts to deallocate each entity in the slice. Invalid
    /// entities (already deallocated, wrong generation, out of bounds, or
    /// PLACEHOLDER) are skipped without error.
    ///
    /// # Arguments
    ///
    /// * `entities` - A slice of entities to deallocate
    ///
    /// # Returns
    ///
    /// The number of entities that were successfully deallocated.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::entity::EntityAllocator;
    ///
    /// let mut allocator = EntityAllocator::new();
    /// let entities = allocator.allocate_batch(100);
    /// assert_eq!(allocator.len(), 100);
    ///
    /// // Deallocate all at once
    /// let deallocated = allocator.deallocate_batch(&entities);
    /// assert_eq!(deallocated, 100);
    /// assert_eq!(allocator.len(), 0);
    ///
    /// // Second deallocation returns 0 (already dead)
    /// let deallocated_again = allocator.deallocate_batch(&entities);
    /// assert_eq!(deallocated_again, 0);
    /// ```
    pub fn deallocate_batch(&mut self, entities: &[Entity]) -> usize {
        let mut success_count = 0;

        for entity in entities {
            if self.deallocate(*entity) {
                success_count += 1;
            }
        }

        success_count
    }
}
