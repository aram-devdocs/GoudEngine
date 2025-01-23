use std::collections::{BTreeMap, HashMap as _HashMap};
use std::{ffi::c_uint, rc::Rc};

use tiled::{Loader, Map};

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: c_uint, // TODO: Right now this needs to be user generated. We should generate this in the future.
    pub width: u32,
    pub height: u32,
}

pub struct TextureManager {
    pub textures: _HashMap<c_uint, Rc<Texture>>, // property 1 is the texture id, property 2 is the texture itself.
}

pub struct Tiled {
    pub id: c_uint,
    pub map: Rc<Map>,
    pub texture_ids: Vec<c_uint>,
}

pub struct TiledManager {
    pub selected_map_id: Option<c_uint>,
    pub loader: Loader,
    pub maps: _HashMap<String, Tiled>,
}

// Data Transfer Objects
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Sprite {
    pub id: c_uint,
    pub x: f32,
    pub y: f32,
    pub z_layer: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub dimension_x: f32,
    pub dimension_y: f32,
    pub rotation: f32,
    pub source_rect: Rectangle,
    pub texture_id: c_uint,
    pub debug: bool,
    pub frame: Rectangle,
}

pub type SpriteMap = BTreeMap<i32, Vec<Sprite>>;
// Data Transfer Objects
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SpriteCreateDto {
    pub x: f32,
    pub y: f32,
    pub z_layer: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub dimension_x: f32,
    pub dimension_y: f32,
    pub rotation: f32,
    pub source_rect: Rectangle,
    pub texture_id: c_uint,
    pub debug: bool,
    pub frame: Rectangle,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct SpriteUpdateDto {
    pub id: c_uint,
    pub x: f32,
    pub y: f32,
    pub z_layer: i32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub dimension_x: f32,
    pub dimension_y: f32,
    pub rotation: f32,
    pub source_rect: Rectangle,
    pub texture_id: c_uint,
    pub debug: bool,
    pub frame: Rectangle,
}

// Build a struct for Rc

/// Rectangle
///
/// Represents a rectangle, typically used for spritesheet source rectangles.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Sprite
///
/// Represents a 2D sprite with position, scale, rotation, and optional source rectangle.

#[repr(C)]
pub struct UpdateResponseData {
    pub delta_time: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MousePosition {
    pub x: f64,
    pub y: f64,
}
// Shared types
// Types
// TODO: https://github.com/aram-devdocs/GoudEngine/issues/5
