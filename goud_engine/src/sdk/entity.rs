//! # SDK Entity Operations API
//!
//! Provides methods on [`GoudGame`](super::GoudGame) for entity lifecycle
//! management: spawning, despawning, and querying entity state.
//!
//! These methods are annotated with `#[goud_api]` to auto-generate FFI
//! wrappers that replace the hand-written functions in `ffi/entity.rs`.

use super::GoudGame;
use crate::ecs::Entity;

// NOTE: FFI wrappers are hand-written in ffi/entity.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl GoudGame {
    /// Spawns a new empty entity with no components.
    ///
    /// Returns the entity as a u64 bit representation for FFI compatibility.
    pub fn ffi_entity_spawn_empty(&mut self) -> u64 {
        self.world.spawn_empty().to_bits()
    }

    /// Spawns multiple empty entities in a batch and writes their IDs to
    /// the provided output buffer.
    ///
    /// Returns the number of entities spawned.
    ///
    /// # Safety
    ///
    /// `out_entities` must point to valid memory with capacity for at
    /// least `count` u64 values.
    pub unsafe fn ffi_entity_spawn_batch(&mut self, count: u32, out_entities: *mut u64) -> u32 {
        if out_entities.is_null() || count == 0 {
            return 0;
        }
        let entities = self.world.spawn_batch(count as usize);
        // SAFETY: Caller guarantees out_entities has capacity for count u64s.
        let out_slice = std::slice::from_raw_parts_mut(out_entities, count as usize);
        for (i, entity) in entities.iter().enumerate() {
            out_slice[i] = entity.to_bits();
        }
        entities.len() as u32
    }

    /// Despawns an entity and all its components from the world.
    ///
    /// Returns `true` if the entity was despawned, `false` otherwise.
    pub fn ffi_entity_despawn(&mut self, entity_id: u64) -> bool {
        let entity = Entity::from_bits(entity_id);
        self.world.despawn(entity)
    }

    /// Despawns multiple entities in a batch.
    ///
    /// Returns the number of entities successfully despawned.
    ///
    /// # Safety
    ///
    /// `entity_ids` must point to valid memory with at least `count` u64
    /// values.
    pub unsafe fn ffi_entity_despawn_batch(&mut self, entity_ids: *const u64, count: u32) -> u32 {
        if entity_ids.is_null() || count == 0 {
            return 0;
        }
        // SAFETY: Caller guarantees entity_ids has count u64 values.
        let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
        let entities: Vec<Entity> = entity_slice
            .iter()
            .map(|&bits| Entity::from_bits(bits))
            .collect();
        self.world.despawn_batch(&entities) as u32
    }

    /// Checks if an entity is currently alive in the world.
    pub fn ffi_entity_is_alive(&self, entity_id: u64) -> bool {
        let entity = Entity::from_bits(entity_id);
        self.world.is_alive(entity)
    }

    /// Returns the total number of alive entities in the world.
    pub fn ffi_entity_count(&self) -> u32 {
        self.world.entity_count() as u32
    }

    /// Checks if multiple entities are alive in the world.
    ///
    /// Results are written to `out_results` where 1 = alive, 0 = dead.
    /// Returns the number of results written.
    ///
    /// # Safety
    ///
    /// - `entity_ids` must point to valid memory with at least `count` u64
    ///   values.
    /// - `out_results` must point to valid memory with at least `count` u8
    ///   values.
    pub unsafe fn ffi_entity_is_alive_batch(
        &self,
        entity_ids: *const u64,
        count: u32,
        out_results: *mut u8,
    ) -> u32 {
        if entity_ids.is_null() || out_results.is_null() || count == 0 {
            return 0;
        }
        // SAFETY: Caller guarantees pointers have sufficient capacity.
        let entity_slice = std::slice::from_raw_parts(entity_ids, count as usize);
        let results_slice = std::slice::from_raw_parts_mut(out_results, count as usize);
        for (i, &entity_bits) in entity_slice.iter().enumerate() {
            let entity = Entity::from_bits(entity_bits);
            results_slice[i] = if self.world.is_alive(entity) { 1 } else { 0 };
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::GameConfig;

    #[test]
    fn test_ffi_entity_spawn_empty() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let bits = game.ffi_entity_spawn_empty();
        assert_ne!(bits, u64::MAX);
        assert!(game.ffi_entity_is_alive(bits));
    }

    #[test]
    fn test_ffi_entity_despawn() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let bits = game.ffi_entity_spawn_empty();
        assert!(game.ffi_entity_despawn(bits));
        assert!(!game.ffi_entity_is_alive(bits));
    }

    #[test]
    fn test_ffi_entity_count() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert_eq!(game.ffi_entity_count(), 0);
        game.ffi_entity_spawn_empty();
        assert_eq!(game.ffi_entity_count(), 1);
        game.ffi_entity_spawn_empty();
        assert_eq!(game.ffi_entity_count(), 2);
    }

    #[test]
    fn test_ffi_entity_spawn_batch() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let mut out = vec![0u64; 5];
        let count = unsafe { game.ffi_entity_spawn_batch(5, out.as_mut_ptr()) };
        assert_eq!(count, 5);
        for &bits in &out {
            assert!(game.ffi_entity_is_alive(bits));
        }
    }

    #[test]
    fn test_ffi_entity_despawn_batch() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let mut out = vec![0u64; 3];
        unsafe { game.ffi_entity_spawn_batch(3, out.as_mut_ptr()) };
        assert_eq!(game.ffi_entity_count(), 3);

        let count = unsafe { game.ffi_entity_despawn_batch(out.as_ptr(), 3) };
        assert_eq!(count, 3);
        assert_eq!(game.ffi_entity_count(), 0);
    }

    #[test]
    fn test_ffi_entity_is_alive_batch() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let mut entities = vec![0u64; 3];
        unsafe { game.ffi_entity_spawn_batch(3, entities.as_mut_ptr()) };

        // Despawn middle one
        game.ffi_entity_despawn(entities[1]);

        let mut results = vec![0u8; 3];
        let count =
            unsafe { game.ffi_entity_is_alive_batch(entities.as_ptr(), 3, results.as_mut_ptr()) };
        assert_eq!(count, 3);
        assert_eq!(results[0], 1);
        assert_eq!(results[1], 0);
        assert_eq!(results[2], 1);
    }
}
