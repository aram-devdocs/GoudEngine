// src/sprite.rs

use crate::{
    types::Rectangle,
    types::{Sprite, SpriteCreateDto, SpriteUpdateDto},
};
use std::ffi::c_uint;

impl Sprite {
    /// Creates a new Sprite.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: c_uint,
        x: f32,
        y: f32,
        z_layer: i32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        texture_id: c_uint,
        debug: bool,
        frame: Rectangle,
    ) -> Sprite {
        Sprite {
            id,
            x,
            y,
            z_layer,
            scale_x,
            scale_y,
            dimension_x,
            dimension_y,
            rotation,
            texture_id,
            source_rect,
            debug,
            frame,
        }
    }

    // TODO: This should be moved
    pub fn check_collision(&self, other: &Sprite) -> bool {
        let self_left = self.x;
        let self_right = self.x + self.dimension_x;
        let self_top = self.y;
        let self_bottom = self.y + self.dimension_y;

        let other_left = other.x;
        let other_right = other.x + other.dimension_x;
        let other_top = other.y;
        let other_bottom = other.y + other.dimension_y;

        self_left < other_right
            && self_right > other_left
            && self_top < other_bottom
            && self_bottom > other_top
    }
}

impl SpriteCreateDto {
    /// Creates a new SpriteCreateDto.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x: f32,
        y: f32,
        z_layer: i32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        texture_id: c_uint,
        debug: bool,
        frame: Rectangle,
    ) -> SpriteCreateDto {
        SpriteCreateDto {
            x,
            y,
            z_layer,
            scale_x,
            scale_y,
            dimension_x,
            dimension_y,
            rotation,
            source_rect,
            texture_id,
            debug,
            frame,
        }
    }
}

impl SpriteUpdateDto {
    /// Creates a new SpriteUpdateDto.
    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub fn new(
        id: c_uint,
        x: f32,
        y: f32,
        z_layer: i32,
        scale_x: f32,
        scale_y: f32,
        dimension_x: f32,
        dimension_y: f32,
        rotation: f32,
        source_rect: Rectangle,
        texture_id: c_uint,
        debug: bool,
        frame: Rectangle,
    ) -> SpriteUpdateDto {
        SpriteUpdateDto {
            id,
            x,
            y,
            z_layer,
            scale_x,
            scale_y,
            dimension_x,
            dimension_y,
            rotation,
            source_rect,
            texture_id,
            debug,
            frame,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Rectangle;

    #[test]
    fn test_sprite_new() {
        let source_rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };
        let frame = Rectangle {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 100.0,
        };

        let sprite = Sprite::new(
            1,
            50.0,
            100.0,
            2,
            1.5,
            1.5,
            64.0,
            64.0,
            45.0,
            source_rect,
            10,
            true,
            frame,
        );

        assert_eq!(sprite.id, 1);
        assert_eq!(sprite.x, 50.0);
        assert_eq!(sprite.y, 100.0);
        assert_eq!(sprite.z_layer, 2);
        assert_eq!(sprite.scale_x, 1.5);
        assert_eq!(sprite.scale_y, 1.5);
        assert_eq!(sprite.dimension_x, 64.0);
        assert_eq!(sprite.dimension_y, 64.0);
        assert_eq!(sprite.rotation, 45.0);
        assert_eq!(sprite.source_rect.x, source_rect.x);
        assert_eq!(sprite.source_rect.y, source_rect.y);
        assert_eq!(sprite.source_rect.width, source_rect.width);
        assert_eq!(sprite.source_rect.height, source_rect.height);
        assert_eq!(sprite.texture_id, 10);
        assert!(sprite.debug);
        assert_eq!(sprite.frame.x, frame.x);
        assert_eq!(sprite.frame.y, frame.y);
    }

    #[test]
    fn test_sprite_new_with_defaults() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 16.0,
            height: 16.0,
        };

        let sprite = Sprite::new(
            0, 0.0, 0.0, 0, 1.0, 1.0, 16.0, 16.0, 0.0, rect, 0, false, rect,
        );

        assert_eq!(sprite.id, 0);
        assert_eq!(sprite.x, 0.0);
        assert_eq!(sprite.y, 0.0);
        assert_eq!(sprite.z_layer, 0);
        assert_eq!(sprite.scale_x, 1.0);
        assert_eq!(sprite.scale_y, 1.0);
        assert_eq!(sprite.rotation, 0.0);
        assert!(!sprite.debug);
    }

    #[test]
    fn test_check_collision_overlapping() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };

        let sprite1 = Sprite::new(
            1, 0.0, 0.0, 0, 1.0, 1.0, 50.0, 50.0, 0.0, rect, 0, false, rect,
        );
        let sprite2 = Sprite::new(
            2, 25.0, 25.0, 0, 1.0, 1.0, 50.0, 50.0, 0.0, rect, 0, false, rect,
        );

        assert!(sprite1.check_collision(&sprite2));
        assert!(sprite2.check_collision(&sprite1));
    }

    #[test]
    fn test_check_collision_non_overlapping() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };

        let sprite1 = Sprite::new(
            1, 0.0, 0.0, 0, 1.0, 1.0, 50.0, 50.0, 0.0, rect, 0, false, rect,
        );
        let sprite2 = Sprite::new(
            2, 100.0, 100.0, 0, 1.0, 1.0, 50.0, 50.0, 0.0, rect, 0, false, rect,
        );

        assert!(!sprite1.check_collision(&sprite2));
        assert!(!sprite2.check_collision(&sprite1));
    }

    #[test]
    fn test_check_collision_edge_touching() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };

        let sprite1 = Sprite::new(
            1, 0.0, 0.0, 0, 1.0, 1.0, 50.0, 50.0, 0.0, rect, 0, false, rect,
        );
        let sprite2 = Sprite::new(
            2, 50.0, 0.0, 0, 1.0, 1.0, 50.0, 50.0, 0.0, rect, 0, false, rect,
        );

        assert!(!sprite1.check_collision(&sprite2));
        assert!(!sprite2.check_collision(&sprite1));
    }

    #[test]
    fn test_check_collision_same_position() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };

        let sprite1 = Sprite::new(
            1, 10.0, 10.0, 0, 1.0, 1.0, 30.0, 30.0, 0.0, rect, 0, false, rect,
        );
        let sprite2 = Sprite::new(
            2, 10.0, 10.0, 0, 1.0, 1.0, 30.0, 30.0, 0.0, rect, 0, false, rect,
        );

        assert!(sprite1.check_collision(&sprite2));
        assert!(sprite2.check_collision(&sprite1));
    }

    #[test]
    fn test_check_collision_partial_overlap() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };

        let sprite1 = Sprite::new(
            1, 0.0, 0.0, 0, 1.0, 1.0, 40.0, 40.0, 0.0, rect, 0, false, rect,
        );
        let sprite2 = Sprite::new(
            2, 30.0, 30.0, 0, 1.0, 1.0, 40.0, 40.0, 0.0, rect, 0, false, rect,
        );

        assert!(sprite1.check_collision(&sprite2));
        assert!(sprite2.check_collision(&sprite1));
    }

    #[test]
    fn test_sprite_create_dto_new() {
        let source_rect = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 64.0,
            height: 64.0,
        };
        let frame = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };

        let dto = SpriteCreateDto::new(
            100.0,
            200.0,
            5,
            2.0,
            2.0,
            128.0,
            128.0,
            90.0,
            source_rect,
            42,
            true,
            frame,
        );

        assert_eq!(dto.x, 100.0);
        assert_eq!(dto.y, 200.0);
        assert_eq!(dto.z_layer, 5);
        assert_eq!(dto.scale_x, 2.0);
        assert_eq!(dto.scale_y, 2.0);
        assert_eq!(dto.dimension_x, 128.0);
        assert_eq!(dto.dimension_y, 128.0);
        assert_eq!(dto.rotation, 90.0);
        assert_eq!(dto.source_rect.x, source_rect.x);
        assert_eq!(dto.source_rect.width, source_rect.width);
        assert_eq!(dto.texture_id, 42);
        assert!(dto.debug);
    }

    #[test]
    fn test_sprite_update_dto_new() {
        let source_rect = Rectangle {
            x: 5.0,
            y: 10.0,
            width: 32.0,
            height: 32.0,
        };
        let frame = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1024.0,
            height: 768.0,
        };

        let dto = SpriteUpdateDto::new(
            99,
            150.0,
            250.0,
            3,
            0.5,
            0.5,
            16.0,
            16.0,
            180.0,
            source_rect,
            7,
            false,
            frame,
        );

        assert_eq!(dto.id, 99);
        assert_eq!(dto.x, 150.0);
        assert_eq!(dto.y, 250.0);
        assert_eq!(dto.z_layer, 3);
        assert_eq!(dto.scale_x, 0.5);
        assert_eq!(dto.scale_y, 0.5);
        assert_eq!(dto.dimension_x, 16.0);
        assert_eq!(dto.dimension_y, 16.0);
        assert_eq!(dto.rotation, 180.0);
        assert_eq!(dto.texture_id, 7);
        assert!(!dto.debug);
    }

    #[test]
    fn test_sprite_negative_dimensions() {
        let rect = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };

        let sprite1 = Sprite::new(
            1, 10.0, 10.0, 0, 1.0, 1.0, -30.0, -30.0, 0.0, rect, 0, false, rect,
        );
        let sprite2 = Sprite::new(
            2, 5.0, 5.0, 0, 1.0, 1.0, 20.0, 20.0, 0.0, rect, 0, false, rect,
        );

        assert!(!sprite1.check_collision(&sprite2));
    }
}
