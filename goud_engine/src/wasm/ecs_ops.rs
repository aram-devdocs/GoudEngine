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
    pub fn spawn_empty(&mut self) -> u64 {
        self.world.spawn_empty().to_bits()
    }

    pub fn spawn_batch(&mut self, count: u32) -> Vec<u64> {
        self.world
            .spawn_batch(count as usize)
            .into_iter()
            .map(|e| e.to_bits())
            .collect()
    }

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

    pub fn entity_count(&self) -> u32 {
        self.world.entity_count() as u32
    }

    pub fn is_alive(&self, entity_bits: u64) -> bool {
        self.world.is_alive(Entity::from_bits(entity_bits))
    }

    // ======================================================================
    // Transform2D component
    // ======================================================================

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

    pub fn has_transform2d(&self, entity_bits: u64) -> bool {
        self.world
            .has::<Transform2D>(Entity::from_bits(entity_bits))
    }

    pub fn remove_transform2d(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Transform2D>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Sprite component
    // ======================================================================

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
        anchor_x: f32,
        anchor_y: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        let handle: AssetHandle<TextureAsset> = AssetHandle::new(texture_handle as u32, 1);
        let sprite = Sprite::new(handle)
            .with_color(Color::rgba(r, g, b, a))
            .with_flip(flip_x, flip_y)
            .with_anchor(anchor_x, anchor_y);
        self.world.insert(entity, sprite);
    }

    pub fn get_sprite(&self, entity_bits: u64) -> Option<WasmSprite> {
        let entity = Entity::from_bits(entity_bits);
        self.world.get::<Sprite>(entity).map(|s| WasmSprite {
            texture_handle: s.texture.index() as u32,
            r: s.color.r,
            g: s.color.g,
            b: s.color.b,
            a: s.color.a,
            flip_x: s.flip_x,
            flip_y: s.flip_y,
            anchor_x: s.anchor.x,
            anchor_y: s.anchor.y,
        })
    }

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
        anchor_x: f32,
        anchor_y: f32,
    ) {
        let entity = Entity::from_bits(entity_bits);
        if let Some(s) = self.world.get_mut::<Sprite>(entity) {
            s.texture = AssetHandle::new(texture_handle as u32, 1);
            s.color = Color::rgba(r, g, b, a);
            s.flip_x = flip_x;
            s.flip_y = flip_y;
            s.anchor = Vec2::new(anchor_x, anchor_y);
        }
    }

    pub fn has_sprite(&self, entity_bits: u64) -> bool {
        self.world.has::<Sprite>(Entity::from_bits(entity_bits))
    }

    pub fn remove_sprite(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Sprite>(Entity::from_bits(entity_bits))
            .is_some()
    }

    // ======================================================================
    // Name component
    // ======================================================================

    pub fn add_name(&mut self, entity_bits: u64, name: &str) {
        let entity = Entity::from_bits(entity_bits);
        self.world.insert(entity, Name::new(name));
    }

    pub fn get_name(&self, entity_bits: u64) -> Option<String> {
        let entity = Entity::from_bits(entity_bits);
        self.world
            .get::<Name>(entity)
            .map(|n| n.as_str().to_string())
    }

    pub fn has_name(&self, entity_bits: u64) -> bool {
        self.world.has::<Name>(Entity::from_bits(entity_bits))
    }

    pub fn remove_name(&mut self, entity_bits: u64) -> bool {
        self.world
            .remove::<Name>(Entity::from_bits(entity_bits))
            .is_some()
    }
}
