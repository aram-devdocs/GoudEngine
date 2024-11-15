use std::collections::{BTreeMap, HashMap as _HashMap};
use std::ffi::c_char;
use std::{ffi::c_uint, rc::Rc};

use gl::types::GLuint as _GLuint;
// pub type EntityId = u32;

pub type FontId = u32;

pub struct Character {
    pub texture_id: _GLuint,
    pub size: (u32, u32),
    pub bearing: (i32, i32),
    pub advance: u32,
}

pub struct Font {
    pub characters: _HashMap<char, Character>,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub id: c_uint,
    pub content: String,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub color: (f32, f32, f32),
    pub font_id: FontId,
    pub z_layer: i32,
}

pub type TextMap = BTreeMap<i32, Vec<Text>>;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct ColorInput {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct TextCreateDto {
    pub content: *const c_char,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub color: ColorInput,
    pub font_id: u32,
    pub z_layer: i32,
}
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TextUpdateDto {
    pub id: c_uint,
    pub content: *const c_char,
    pub x: f32,
    pub y: f32,
    pub scale: f32,
    pub color: ColorInput,
    pub font_id: FontId,
    pub z_layer: i32,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: c_uint, // TODO: Right now this needs to be user generated. We should generate this in the future.
    pub width: u32,
    pub height: u32,
}

pub struct TextureManager {
    pub textures: _HashMap<c_uint, Rc<Texture>>, // property 1 is the texture id, property 2 is the texture itself.
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

// Opaque pointers for additional structures
#[repr(C)]
#[allow(dead_code)]
pub struct Glfw {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct Receiver {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct HashSet {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct ShaderProgram {
    _private: [u8; 0],
}

// vao vec

#[repr(C)]
#[allow(dead_code)]

pub struct Vao {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct Duration {
    secs: u64,
    nanos: u32, // Duration is a struct with two fields: secs and nanos. Nanos is nanoseconds coming from std::time::Duration.
}

#[repr(C)]
#[allow(dead_code)]

pub struct Instant {
    secs: u64,
    nanos: u32, // Instant is a struct with two fields: secs and nanos. Nanos is nanoseconds coming from std::time::Instant.
}

#[repr(C)]
#[allow(dead_code)]

pub struct ECS {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct HashMap {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct FontManager {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct GLuint {
    _private: [u8; 0],
}
