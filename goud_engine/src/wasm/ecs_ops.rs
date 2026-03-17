//! ECS entity and component operations for WasmGame.
//!
//! Covers entity lifecycle, Transform2D CRUD, Sprite CRUD, and Name CRUD.

use wasm_bindgen::prelude::*;

use crate::assets::loaders::TextureAsset;
use crate::assets::AssetHandle;
use crate::core::math::{Color, Vec2};
use crate::ecs::components::{Name, Sprite, Transform2D};
use crate::ecs::Entity;

use super::{WasmGame, WasmSprite, WasmTransform2D};

// ---------------------------------------------------------------------------
// Entity operations
// ---------------------------------------------------------------------------

#[wasm_bindgen]
impl WasmGame {
    /// Spawns a single empty entity and returns its packed entity bits.
    pub fn spawn_empty(&mut self) -> u64 {
        self.world.spawn_empty().to_bits()
    }

    /// Spawns `count` empty entities and returns their packed entity bits.
    pub fn spawn_batch(&mut self, count: u32) -> Vec<u64> {
        self.world
            .spawn_batch(count as usize)
            .into_iter()
            .map(|e| e.to_bits())
            .collect()
    }

    /// Despawns an entity by packed entity bits.
    pub fn despawn(&mut self, entity_bits: u64) -> bool {
        self.world.despawn(Entity::from_bits(entity_bits))
    }

    /// Despawns multiple entities at once. Returns the number of entities
    /// successfully despawned.
    pub fn despawn_batch(&mut self, entity_bits: Vec<u64>) -> u32 {
        let mut count = 0u32;
        for bits in entity_bits {
            if self.world.despawn(Entity::from_bits(bits)) {
                count += 1;
            }
        }
        count
    }

    /// Returns the number of currently alive entities.
    pub fn entity_count(&self) -> u32 {
        self.world.entity_count() as u32
    }

    /// Returns whether the entity bits identify a live entity.
    pub fn is_alive(&self, entity_bits: u64) -> bool {
        self.world.is_alive(Entity::from_bits(entity_bits))
    }

    // ======================================================================
    // Transform2D component
    // ======================================================================

    /// Inserts or replaces `Transform2D` on the entity.
    pub fn add_transform2d(
        &mut self,
        entity_bits: u64,
        px: f32,
        py: f32,
        rotation: f32,
        sx: f32,
        sy: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        self.world.insert(
            entity,
            Transform2D {
                position: Vec2::new(px, py),
                rotation,
                scale: Vec2::new(sx, sy),
            },
        );
    }

    /// Gets the entity's `Transform2D` as a wasm-safe DTO.
    pub fn get_transform2d(&self, entity_bits: u64) -> Option<WasmTransform2D> {
        let entity = Entity::from_bits(entity_bits);
        self.world
            .get::<Transform2D>(entity)
            .map(|t| WasmTransform2D {
                position_x: t.position.x,
                position_y: t.position.y,
                rotation: t.rotation,
                scale_x: t.scale.x,
                scale_y: t.scale.y,
            })
    }

    /// Updates an existing `Transform2D` if present.
    pub fn set_transform2d(
        &mut self,
        entity_bits: u64,
        px: f32,
        py: f32,
        rotation: f32,
        sx: f32,
        sy: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        if let Some(t) = self.world.get_mut::<Transform2D>(entity) {
            t.position = Vec2::new(px, py);
            t.rotation = rotation;
            t.scale = Vec2::new(sx, sy);
        }
    }

    /// Returns whether the entity has a `Transform2D` component.
    pub fn has_transform2d(&self, entity_bits: u64) -> bool {
        self.world
            .has::<Transform2D>(Entity::from_bits(entity_bits))
    }

    /// Removes `Transform2D` from the entity.
    pub fn remove_transform2d(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Transform2D>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Sprite component
    // ======================================================================

    /// Inserts or replaces `Sprite` on the entity.
    pub fn add_sprite(
        &mut self,
        entity_bits: u64,
        texture_handle: u32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        flip_x: bool,
        flip_y: bool,
        z_layer: i32,
        anchor_x: f32,
        anchor_y: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        let handle: AssetHandle<TextureAsset> = AssetHandle::new(texture_handle, 1);
        let sprite = Sprite::new(handle)
            .with_color(Color::rgba(r, g, b, a))
            .with_flip(flip_x, flip_y)
            .with_z_layer(z_layer)
            .with_anchor(anchor_x, anchor_y);
        self.world.insert(entity, sprite);
    }

    /// Gets the entity's `Sprite` as a wasm-safe DTO.
    pub fn get_sprite(&self, entity_bits: u64) -> Option<WasmSprite> {
        let entity = Entity::from_bits(entity_bits);
        self.world.get::<Sprite>(entity).map(|s| WasmSprite {
            texture_handle: s.texture.index(),
            r: s.color.r,
            g: s.color.g,
            b: s.color.b,
            a: s.color.a,
            flip_x: s.flip_x,
            flip_y: s.flip_y,
            z_layer: s.z_layer,
            anchor_x: s.anchor.x,
            anchor_y: s.anchor.y,
        })
    }

    /// Updates an existing `Sprite` if present.
    pub fn set_sprite(
        &mut self,
        entity_bits: u64,
        texture_handle: u32,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
        flip_x: bool,
        flip_y: bool,
        z_layer: i32,
        anchor_x: f32,
        anchor_y: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        if let Some(s) = self.world.get_mut::<Sprite>(entity) {
            s.texture = AssetHandle::new(texture_handle, 1);
            s.color = Color::rgba(r, g, b, a);
            s.flip_x = flip_x;
            s.flip_y = flip_y;
            s.z_layer = z_layer;
            s.anchor = Vec2::new(anchor_x, anchor_y);
        }
    }

    /// Returns whether the entity has a `Sprite` component.
    pub fn has_sprite(&self, entity_bits: u64) -> bool {
        self.world.has::<Sprite>(Entity::from_bits(entity_bits))
    }

    /// Removes `Sprite` from the entity.
    pub fn remove_sprite(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Sprite>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Name component
    // ======================================================================

    /// Inserts or replaces `Name` on the entity.
    pub fn add_name(&mut self, entity_bits: u64, name: &str) {
        let entity = Entity::from_bits(entity_bits);
        self.world.insert(entity, Name::new(name));
    }

    /// Gets the entity's `Name`.
    pub fn get_name(&self, entity_bits: u64) -> Option<String> {
        let entity = Entity::from_bits(entity_bits);
        self.world
            .get::<Name>(entity)
            .map(|n| n.as_str().to_string())
    }

    /// Returns whether the entity has a `Name` component.
    pub fn has_name(&self, entity_bits: u64) -> bool {
        self.world.has::<Name>(Entity::from_bits(entity_bits))
    }

    /// Removes `Name` from the entity.
    pub fn remove_name(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Name>(Entity::from_bits(entity_bits))
            .is_some()
    }
}
