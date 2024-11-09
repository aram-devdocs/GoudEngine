use std::os::raw::c_char;
use std::{ffi::c_uint, rc::Rc};
pub type EntityId = u32;

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: c_uint,
    pub width: u32,
    pub height: u32,
}

// Data Transfer Objects
#[derive(Debug, Clone)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub dimension_x: f32,
    pub dimension_y: f32,
    pub rotation: f32,
    pub source_rect: Rectangle,
    pub texture: Rc<Texture>,
}

pub type SpriteMap = Vec<Option<Sprite>>;
// Data Transfer Objects
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SpriteDto {
    pub x: f32,
    pub y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub dimension_x: f32,
    pub dimension_y: f32,
    pub rotation: f32,
    pub source_rect: Rectangle,
    pub texture_path: *const c_char,
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

// pub struct Vec {
//     _private: [u8; 0],
// }
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
