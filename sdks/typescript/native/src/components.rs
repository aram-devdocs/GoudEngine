use crate::types::{Color, Vec2};
use goud_engine::core::math::{Color as EngineColor, Vec2 as EngineVec2};
use goud_engine::ecs::components::{Sprite, Transform2D};
use napi_derive::napi;

// =============================================================================
// Transform2D
// =============================================================================

#[napi(object)]
#[derive(Clone, Debug)]
pub struct Transform2DData {
    pub position_x: f64,
    pub position_y: f64,
    pub rotation: f64,
    pub scale_x: f64,
    pub scale_y: f64,
}

impl Default for Transform2DData {
    fn default() -> Self {
        Self {
            position_x: 0.0,
            position_y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

impl From<&Transform2D> for Transform2DData {
    fn from(t: &Transform2D) -> Self {
        Self {
            position_x: t.position.x as f64,
            position_y: t.position.y as f64,
            rotation: t.rotation as f64,
            scale_x: t.scale.x as f64,
            scale_y: t.scale.y as f64,
        }
    }
}

impl From<&Transform2DData> for Transform2D {
    fn from(data: &Transform2DData) -> Self {
        Transform2D {
            position: EngineVec2::new(data.position_x as f32, data.position_y as f32),
            rotation: data.rotation as f32,
            scale: EngineVec2::new(data.scale_x as f32, data.scale_y as f32),
        }
    }
}

// =============================================================================
// Sprite
// =============================================================================

#[napi(object)]
#[derive(Clone, Debug)]
pub struct SpriteData {
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub anchor_x: f64,
    pub anchor_y: f64,
    pub custom_width: Option<f64>,
    pub custom_height: Option<f64>,
    pub source_rect_x: Option<f64>,
    pub source_rect_y: Option<f64>,
    pub source_rect_width: Option<f64>,
    pub source_rect_height: Option<f64>,
}

impl From<&Sprite> for SpriteData {
    fn from(s: &Sprite) -> Self {
        let (custom_width, custom_height) = match s.custom_size {
            Some(size) => (Some(size.x as f64), Some(size.y as f64)),
            None => (None, None),
        };
        let (src_x, src_y, src_w, src_h) = match s.source_rect {
            Some(rect) => (
                Some(rect.x as f64),
                Some(rect.y as f64),
                Some(rect.width as f64),
                Some(rect.height as f64),
            ),
            None => (None, None, None, None),
        };
        Self {
            color: s.color.into(),
            flip_x: s.flip_x,
            flip_y: s.flip_y,
            anchor_x: s.anchor.x as f64,
            anchor_y: s.anchor.y as f64,
            custom_width,
            custom_height,
            source_rect_x: src_x,
            source_rect_y: src_y,
            source_rect_width: src_w,
            source_rect_height: src_h,
        }
    }
}

impl From<&SpriteData> for Sprite {
    fn from(data: &SpriteData) -> Self {
        Sprite {
            color: EngineColor::from(&data.color),
            flip_x: data.flip_x,
            flip_y: data.flip_y,
            anchor: EngineVec2::new(data.anchor_x as f32, data.anchor_y as f32),
            custom_size: match (data.custom_width, data.custom_height) {
                (Some(w), Some(h)) => Some(EngineVec2::new(w as f32, h as f32)),
                _ => None,
            },
            source_rect: match (
                data.source_rect_x,
                data.source_rect_y,
                data.source_rect_width,
                data.source_rect_height,
            ) {
                (Some(x), Some(y), Some(w), Some(h)) => Some(goud_engine::core::math::Rect::new(
                    x as f32, y as f32, w as f32, h as f32,
                )),
                _ => None,
            },
            ..Default::default()
        }
    }
}

// =============================================================================
// Factory functions for Transform2D
// =============================================================================

#[napi]
pub fn transform2d_default() -> Transform2DData {
    Transform2DData::default()
}

#[napi]
pub fn transform2d_from_position(x: f64, y: f64) -> Transform2DData {
    Transform2DData {
        position_x: x,
        position_y: y,
        ..Transform2DData::default()
    }
}

#[napi]
pub fn transform2d_from_scale(x: f64, y: f64) -> Transform2DData {
    Transform2DData {
        scale_x: x,
        scale_y: y,
        ..Transform2DData::default()
    }
}

#[napi]
pub fn transform2d_from_rotation(radians: f64) -> Transform2DData {
    Transform2DData {
        rotation: radians,
        ..Transform2DData::default()
    }
}

// Factory functions for Sprite
#[napi]
pub fn sprite_default() -> SpriteData {
    let sprite = Sprite::default();
    SpriteData::from(&sprite)
}

// Factory functions for Vec2 convenience
#[napi]
pub fn vec2(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

#[napi]
pub fn vec2_zero() -> Vec2 {
    Vec2 { x: 0.0, y: 0.0 }
}

#[napi]
pub fn vec2_one() -> Vec2 {
    Vec2 { x: 1.0, y: 1.0 }
}
