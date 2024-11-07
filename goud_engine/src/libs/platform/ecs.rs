// ecs.rs

use std::collections::HashMap;
use std::rc::Rc;
use crate::types::{SpriteData, EntityId};
use crate::libs::platform::graphics::rendering::{Rectangle, Sprite, Texture};
use crate::game::cgmath::Vector2;

pub struct ECS {
    entities: HashMap<EntityId, Entity>,
    next_entity_id: EntityId,
}

pub struct Entity {
    pub id: EntityId,
    pub components: Components,
}

pub struct Components {
    pub sprite: Option<SpriteComponent>,
    // You can add more components like Position, Velocity, etc.
}

pub struct SpriteComponent {
    pub texture: Rc<Texture>,
    pub sprite: Sprite,
}

impl ECS {
    pub fn new() -> Self {
        ECS {
            entities: HashMap::new(),
            next_entity_id: 0,
        }
    }

    pub fn create_entity(&mut self) -> EntityId {
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.insert(entity_id, Entity {
            id: entity_id,
            components: Components { sprite: None },
        });
        entity_id
    }

    pub fn add_sprite_component(&mut self, entity_id: EntityId, texture_path: &str, data: &SpriteData) {
        let entity = self.entities.get_mut(&entity_id).expect("Entity not found");
        let texture = Texture::new(texture_path).expect("Failed to load texture");

        let sprite = Sprite::new(
            texture.clone(),
            Vector2::new(data.x, data.y),
            Vector2::new(
                data.scale_x.unwrap_or(1.0),
                data.scale_y.unwrap_or(1.0),
            ),
            Vector2::new(
                data.dimension_x.unwrap_or(texture.width() as f32),
                data.dimension_y.unwrap_or(texture.height() as f32),
            ),
            data.rotation,
            Some(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            }),
        );

        entity.components.sprite = Some(SpriteComponent { texture, sprite });
    }

    pub fn update_sprite_component(&mut self, entity_id: EntityId, data: &SpriteData) {
        if let Some(entity) = self.entities.get_mut(&entity_id) {
            if let Some(sprite_component) = &mut entity.components.sprite {
                sprite_component.sprite.position = Vector2::new(data.x, data.y);
                sprite_component.sprite.scale = Vector2::new(
                    data.scale_x.unwrap_or(1.0),
                    data.scale_y.unwrap_or(1.0),
                );
                sprite_component.sprite.dimensions = Vector2::new(
                    data.dimension_x.unwrap_or(sprite_component.texture.width() as f32),
                    data.dimension_y.unwrap_or(sprite_component.texture.height() as f32),
                );
                sprite_component.sprite.rotation = data.rotation;
            }
        }
    }

    pub fn get_all_sprites(&self) -> Vec<Sprite> {
        self.entities
            .values()
            .filter_map(|entity| entity.components.sprite.as_ref().map(|s| s.sprite.clone()))
            .collect()
    }
}