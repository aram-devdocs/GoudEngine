#[repr(C)]

// Data Transfer Objects
pub struct SpriteData {
    pub x: f32,
    pub y: f32,
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub dimmension_x: Option<f32>,
    pub dimmension_y: Option<f32>,
    pub rotation: f32,
}

// Shared types
// Types
// TODO: https://github.com/aram-devdocs/GoudEngine/issues/5

// Opaque pointers for additional structures
#[repr(C)]
pub struct Glfw {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Receiver {
    _private: [u8; 0],
}

#[repr(C)]
pub struct HashSet {
    _private: [u8; 0],
}

#[repr(C)]
pub struct ShaderProgram {
    _private: [u8; 0],
}

// vao vec

#[repr(C)]
pub struct Vao {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Vec {
    _private: [u8; 0],
}
