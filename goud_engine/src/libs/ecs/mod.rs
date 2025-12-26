use std::{
    collections::{BTreeMap, HashMap},
    ffi::c_uint,
};

use crate::types::{Sprite, SpriteCreateDto, SpriteUpdateDto};

pub struct Ecs {
    pub sprites: BTreeMap<i32, Vec<Sprite>>,
    /// Maps sprite ID to its z_layer for O(1) layer lookup
    sprite_layer_index: HashMap<c_uint, i32>,
    next_id: c_uint,        // Tracks the next unused ID
    free_list: Vec<c_uint>, // List of reusable indices
}

impl Ecs {
    pub fn new() -> Self {
        Ecs {
            sprites: BTreeMap::new(),
            sprite_layer_index: HashMap::new(),
            next_id: 0,
            free_list: Vec::new(),
        }
    }

    /// Adds a sprite to the ECS and returns its unique EntityId.
    pub fn add_sprite(&mut self, sprite_input: SpriteCreateDto) -> c_uint {
        let id = self.free_list.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        });

        let z_layer = sprite_input.z_layer;
        let sprite = Sprite::new(
            id,
            sprite_input.x,
            sprite_input.y,
            z_layer,
            sprite_input.scale_x,
            sprite_input.scale_y,
            sprite_input.dimension_x,
            sprite_input.dimension_y,
            sprite_input.rotation,
            sprite_input.source_rect,
            sprite_input.texture_id,
            sprite_input.debug,
            sprite_input.frame,
        );

        // Add to z-layer storage
        self.sprites.entry(z_layer).or_default().push(sprite);
        // Add to index for O(1) layer lookup
        self.sprite_layer_index.insert(id, z_layer);

        id
    }

    /// Updates a sprite in the ECS. Uses O(1) layer lookup.
    pub fn update_sprite(&mut self, sprite_input: SpriteUpdateDto) -> Result<(), String> {
        let id = sprite_input.id;
        let new_z_layer = sprite_input.z_layer;

        // O(1) lookup of current z_layer
        let current_z_layer = self
            .sprite_layer_index
            .get(&id)
            .copied()
            .ok_or_else(|| "EntityId not found".to_string())?;

        let new_sprite = Sprite::new(
            id,
            sprite_input.x,
            sprite_input.y,
            new_z_layer,
            sprite_input.scale_x,
            sprite_input.scale_y,
            sprite_input.dimension_x,
            sprite_input.dimension_y,
            sprite_input.rotation,
            sprite_input.source_rect,
            sprite_input.texture_id,
            sprite_input.debug,
            sprite_input.frame,
        );

        // If z_layer changed, move sprite between layers
        if current_z_layer != new_z_layer {
            // Remove from old layer
            if let Some(sprites) = self.sprites.get_mut(&current_z_layer) {
                if let Some(index) = sprites.iter().position(|s| s.id == id) {
                    sprites.remove(index);
                    // Clean up empty layers
                    if sprites.is_empty() {
                        self.sprites.remove(&current_z_layer);
                    }
                }
            }
            // Add to new layer
            self.sprites
                .entry(new_z_layer)
                .or_default()
                .push(new_sprite);
            // Update index
            self.sprite_layer_index.insert(id, new_z_layer);
        } else {
            // Same layer, just update in place
            if let Some(sprites) = self.sprites.get_mut(&current_z_layer) {
                if let Some(existing_sprite) = sprites.iter_mut().find(|s| s.id == id) {
                    *existing_sprite = new_sprite;
                }
            }
        }

        Ok(())
    }

    /// Gets a sprite by ID. Uses O(1) layer lookup, then searches within the layer.
    pub fn get_sprite(&self, sprite_id: c_uint) -> Option<&Sprite> {
        // O(1) lookup of z_layer
        let z_layer = self.sprite_layer_index.get(&sprite_id)?;
        // Search within the specific layer only (much smaller than all sprites)
        self.sprites
            .get(z_layer)?
            .iter()
            .find(|s| s.id == sprite_id)
    }

    /// Removes a sprite by ID. Uses O(1) layer lookup.
    pub fn remove_sprite(&mut self, sprite_id: c_uint) -> Result<Sprite, String> {
        // O(1) lookup of z_layer
        let z_layer = self
            .sprite_layer_index
            .remove(&sprite_id)
            .ok_or_else(|| "EntityId not found".to_string())?;

        // Find and remove from the layer
        let sprites = self
            .sprites
            .get_mut(&z_layer)
            .ok_or_else(|| "EntityId not found".to_string())?;

        let index = sprites
            .iter()
            .position(|s| s.id == sprite_id)
            .ok_or_else(|| "EntityId not found".to_string())?;

        let sprite = sprites.remove(index);

        // Clean up empty layers
        if sprites.is_empty() {
            self.sprites.remove(&z_layer);
        }

        // Add ID to free list for reuse
        self.free_list.push(sprite_id);
        Ok(sprite)
    }

    /// Checks collision between two sprites. Uses O(1) layer lookup via get_sprite.
    pub fn check_collision_between_sprites(&self, sprite_id1: c_uint, sprite_id2: c_uint) -> bool {
        match (self.get_sprite(sprite_id1), self.get_sprite(sprite_id2)) {
            (Some(sprite1), Some(sprite2)) => sprite1.check_collision(sprite2),
            _ => false,
        }
    }

    /// Clears all sprites and resets the ECS state.
    pub fn terminate(&mut self) {
        self.sprites.clear();
        self.sprite_layer_index.clear();
        self.free_list.clear();
        self.next_id = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rectangle;

    fn create_test_rectangle(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    fn create_test_sprite_dto(x: f32, y: f32, z_layer: i32) -> SpriteCreateDto {
        SpriteCreateDto {
            x,
            y,
            z_layer,
            scale_x: 1.0,
            scale_y: 1.0,
            dimension_x: 32.0,
            dimension_y: 32.0,
            rotation: 0.0,
            source_rect: create_test_rectangle(0.0, 0.0, 32.0, 32.0),
            texture_id: 1,
            debug: false,
            frame: create_test_rectangle(0.0, 0.0, 32.0, 32.0),
        }
    }

    fn create_test_sprite_update_dto(id: c_uint, x: f32, y: f32, z_layer: i32) -> SpriteUpdateDto {
        SpriteUpdateDto {
            id,
            x,
            y,
            z_layer,
            scale_x: 1.0,
            scale_y: 1.0,
            dimension_x: 32.0,
            dimension_y: 32.0,
            rotation: 0.0,
            source_rect: create_test_rectangle(0.0, 0.0, 32.0, 32.0),
            texture_id: 1,
            debug: false,
            frame: create_test_rectangle(0.0, 0.0, 32.0, 32.0),
        }
    }

    #[test]
    fn test_ecs_new() {
        let ecs = Ecs::new();
        assert!(ecs.sprites.is_empty());
        assert_eq!(ecs.next_id, 0);
        assert!(ecs.free_list.is_empty());
    }

    #[test]
    fn test_ecs_add_sprite() {
        let mut ecs = Ecs::new();
        let sprite_dto = create_test_sprite_dto(10.0, 20.0, 0);

        let id = ecs.add_sprite(sprite_dto);
        assert_eq!(id, 0);
        assert_eq!(ecs.next_id, 1);

        let sprite = ecs.get_sprite(id).unwrap();
        assert_eq!(sprite.id, id);
        assert_eq!(sprite.x, 10.0);
        assert_eq!(sprite.y, 20.0);
        assert_eq!(sprite.z_layer, 0);
    }

    #[test]
    fn test_ecs_update_sprite() {
        let mut ecs = Ecs::new();
        let sprite_dto = create_test_sprite_dto(10.0, 20.0, 0);
        let id = ecs.add_sprite(sprite_dto);

        let update_dto = create_test_sprite_update_dto(id, 30.0, 40.0, 1);
        let result = ecs.update_sprite(update_dto);
        assert!(result.is_ok());

        let sprite = ecs.get_sprite(id).unwrap();
        assert_eq!(sprite.x, 30.0);
        assert_eq!(sprite.y, 40.0);
        assert_eq!(sprite.z_layer, 1);
    }

    #[test]
    fn test_ecs_get_sprite() {
        let mut ecs = Ecs::new();
        let sprite_dto = create_test_sprite_dto(10.0, 20.0, 0);
        let id = ecs.add_sprite(sprite_dto);

        assert!(ecs.get_sprite(id).is_some());
        assert!(ecs.get_sprite(999).is_none());
    }

    #[test]
    fn test_ecs_remove_sprite() {
        let mut ecs = Ecs::new();
        let sprite_dto = create_test_sprite_dto(10.0, 20.0, 0);
        let id = ecs.add_sprite(sprite_dto);

        let result = ecs.remove_sprite(id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, id);

        assert!(ecs.get_sprite(id).is_none());
    }

    #[test]
    fn test_ecs_id_reuse() {
        let mut ecs = Ecs::new();

        let sprite_dto1 = create_test_sprite_dto(10.0, 20.0, 0);
        let id1 = ecs.add_sprite(sprite_dto1);
        assert_eq!(id1, 0);

        let sprite_dto2 = create_test_sprite_dto(30.0, 40.0, 0);
        let id2 = ecs.add_sprite(sprite_dto2);
        assert_eq!(id2, 1);

        ecs.remove_sprite(id1).unwrap();
        assert_eq!(ecs.free_list.len(), 1);

        let sprite_dto3 = create_test_sprite_dto(50.0, 60.0, 0);
        let id3 = ecs.add_sprite(sprite_dto3);
        assert_eq!(id3, id1);
        assert!(ecs.free_list.is_empty());
    }

    #[test]
    fn test_ecs_sequential_id_generation() {
        let mut ecs = Ecs::new();

        for i in 0..5 {
            let sprite_dto = create_test_sprite_dto(i as f32, i as f32, 0);
            let id = ecs.add_sprite(sprite_dto);
            assert_eq!(id, i);
        }
        assert_eq!(ecs.next_id, 5);
    }

    #[test]
    fn test_ecs_id_uniqueness() {
        let mut ecs = Ecs::new();
        let mut ids = Vec::new();

        for i in 0..10 {
            let sprite_dto = create_test_sprite_dto(i as f32, i as f32, 0);
            ids.push(ecs.add_sprite(sprite_dto));
        }

        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 10);
    }

    #[test]
    fn test_ecs_z_layer_organization() {
        let mut ecs = Ecs::new();

        ecs.add_sprite(create_test_sprite_dto(0.0, 0.0, 2));
        ecs.add_sprite(create_test_sprite_dto(1.0, 1.0, 0));
        ecs.add_sprite(create_test_sprite_dto(2.0, 2.0, 1));
        ecs.add_sprite(create_test_sprite_dto(3.0, 3.0, 0));

        assert_eq!(ecs.sprites.len(), 3);
        assert_eq!(ecs.sprites[&0].len(), 2);
        assert_eq!(ecs.sprites[&1].len(), 1);
        assert_eq!(ecs.sprites[&2].len(), 1);
    }

    #[test]
    fn test_ecs_multiple_sprites_same_layer() {
        let mut ecs = Ecs::new();

        for i in 0..5 {
            ecs.add_sprite(create_test_sprite_dto(i as f32, i as f32, 0));
        }

        assert_eq!(ecs.sprites[&0].len(), 5);
    }

    #[test]
    fn test_ecs_update_sprite_z_layer_change() {
        let mut ecs = Ecs::new();
        let id = ecs.add_sprite(create_test_sprite_dto(10.0, 20.0, 0));

        assert_eq!(ecs.sprites[&0].len(), 1);
        assert!(!ecs.sprites.contains_key(&5));

        let update_dto = create_test_sprite_update_dto(id, 10.0, 20.0, 5);
        ecs.update_sprite(update_dto).unwrap();

        // Sprite should have moved from layer 0 to layer 5
        assert!(!ecs.sprites.contains_key(&0)); // Layer 0 should be removed (was empty)
        assert_eq!(ecs.sprites[&5].len(), 1); // Sprite should now be in layer 5

        let sprite = ecs.get_sprite(id).unwrap();
        assert_eq!(sprite.z_layer, 5);
    }

    #[test]
    fn test_ecs_update_nonexistent_sprite() {
        let mut ecs = Ecs::new();
        let update_dto = create_test_sprite_update_dto(999, 10.0, 20.0, 0);

        let result = ecs.update_sprite(update_dto);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "EntityId not found");
    }

    #[test]
    fn test_ecs_remove_nonexistent_sprite() {
        let mut ecs = Ecs::new();

        let result = ecs.remove_sprite(999);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "EntityId not found");
    }

    #[test]
    fn test_ecs_get_nonexistent_sprite() {
        let ecs = Ecs::new();
        assert!(ecs.get_sprite(999).is_none());
    }

    #[test]
    fn test_ecs_collision_with_nonexistent_sprites() {
        let mut ecs = Ecs::new();
        let id = ecs.add_sprite(create_test_sprite_dto(10.0, 20.0, 0));

        assert!(!ecs.check_collision_between_sprites(id, 999));
        assert!(!ecs.check_collision_between_sprites(999, id));
        assert!(!ecs.check_collision_between_sprites(998, 999));
    }

    #[test]
    fn test_ecs_collision_between_sprites() {
        let mut ecs = Ecs::new();

        let id1 = ecs.add_sprite(create_test_sprite_dto(0.0, 0.0, 0));
        let id2 = ecs.add_sprite(create_test_sprite_dto(10.0, 10.0, 0));

        assert!(ecs.check_collision_between_sprites(id1, id2));
        assert!(ecs.check_collision_between_sprites(id2, id1));
    }

    #[test]
    fn test_ecs_no_collision() {
        let mut ecs = Ecs::new();

        let id1 = ecs.add_sprite(create_test_sprite_dto(0.0, 0.0, 0));
        let id2 = ecs.add_sprite(create_test_sprite_dto(100.0, 100.0, 0));

        assert!(!ecs.check_collision_between_sprites(id1, id2));
        assert!(!ecs.check_collision_between_sprites(id2, id1));
    }

    #[test]
    fn test_ecs_collision_edge_cases() {
        let mut ecs = Ecs::new();

        let id1 = ecs.add_sprite(create_test_sprite_dto(0.0, 0.0, 0));
        let id2 = ecs.add_sprite(create_test_sprite_dto(32.0, 0.0, 0));
        let id3 = ecs.add_sprite(create_test_sprite_dto(0.0, 32.0, 0));
        let id4 = ecs.add_sprite(create_test_sprite_dto(31.9, 31.9, 0));

        assert!(!ecs.check_collision_between_sprites(id1, id2));
        assert!(!ecs.check_collision_between_sprites(id1, id3));
        assert!(ecs.check_collision_between_sprites(id1, id4));
    }

    #[test]
    fn test_ecs_terminate() {
        let mut ecs = Ecs::new();

        for i in 0..5 {
            ecs.add_sprite(create_test_sprite_dto(i as f32, i as f32, i));
        }
        ecs.remove_sprite(2).unwrap();

        assert!(!ecs.sprites.is_empty());
        assert!(!ecs.free_list.is_empty());
        assert_ne!(ecs.next_id, 0);

        ecs.terminate();

        assert!(ecs.sprites.is_empty());
        assert!(ecs.free_list.is_empty());
        assert_eq!(ecs.next_id, 0);
    }

    #[test]
    fn test_ecs_large_scale_operations() {
        let mut ecs = Ecs::new();

        for i in 0..100 {
            let z_layer = i % 10;
            ecs.add_sprite(create_test_sprite_dto(i as f32, i as f32, z_layer));
        }

        assert_eq!(ecs.sprites.len(), 10);
        for z in 0..10 {
            assert_eq!(ecs.sprites[&z].len(), 10);
        }

        for i in (0..100).step_by(2) {
            ecs.remove_sprite(i).unwrap();
        }

        let total_sprites: usize = ecs.sprites.values().map(|v| v.len()).sum();
        assert_eq!(total_sprites, 50);
        assert_eq!(ecs.free_list.len(), 50);
    }

    #[test]
    fn test_ecs_stress_test_id_reuse() {
        let mut ecs = Ecs::new();
        let mut all_ids = Vec::new();

        for _iteration in 0..10 {
            let mut current_ids = Vec::new();
            for i in 0..20 {
                let id = ecs.add_sprite(create_test_sprite_dto(i as f32, i as f32, 0));
                current_ids.push(id);
            }
            all_ids.extend(&current_ids);

            // Remove the last 10 sprites added in this iteration
            for &id in current_ids.iter().skip(10) {
                ecs.remove_sprite(id).unwrap();
            }
        }

        all_ids.sort_unstable();
        all_ids.dedup();

        // Analysis:
        // - Each iteration adds 20 sprites and removes 10
        // - We accumulate 10 sprites per iteration
        // - After 10 iterations, we have 100 active sprites
        // - The last iteration removes 10 sprites that go to free list
        let max_id: c_uint = *all_ids.iter().max().unwrap();
        let active_count = ecs.sprites.values().map(|v| v.len()).sum::<usize>();
        assert_eq!(active_count, 100, "Should have exactly 100 active sprites");
        assert_eq!(
            ecs.free_list.len(),
            10,
            "Free list should have 10 IDs from last removal"
        );

        // The important thing is that ID reuse is working
        let unique_count = all_ids.len();
        assert_eq!(
            unique_count, 110,
            "Should have seen exactly 110 unique IDs total"
        );
        assert_eq!(max_id, 109, "Max ID should be 109 (0-indexed)");
    }
}
