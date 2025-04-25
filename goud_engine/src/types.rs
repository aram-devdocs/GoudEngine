use cgmath::Vector3;
use std::collections::{BTreeMap, HashMap as _HashMap};
use std::{ffi::c_uint, rc::Rc};

use tiled::{Loader, Map};

/// Grid rendering modes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridRenderMode {
    /// Grid blends with scene objects (drawn with proper depth testing)
    Blend,
    /// Grid is drawn on top of scene objects (drawn without depth testing)
    Overlap,
}

/// Configuration for the debug grid
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GridConfig {
    pub enabled: bool,
    pub size: f32,
    pub divisions: u32,
    pub xz_color: Vector3<f32>,      // Floor grid color (XZ plane)
    pub xy_color: Vector3<f32>,      // Vertical grid color (XY plane)
    pub yz_color: Vector3<f32>,      // Vertical grid color (YZ plane)
    pub x_axis_color: Vector3<f32>,  // X axis color
    pub y_axis_color: Vector3<f32>,  // Y axis color
    pub z_axis_color: Vector3<f32>,  // Z axis color
    pub line_width: f32,             // Width of grid lines
    pub axis_line_width: f32,        // Width of axis lines
    pub show_axes: bool,             // Whether to show coordinate axes
    pub show_xz_plane: bool,         // Show floor (XZ) plane
    pub show_xy_plane: bool,         // Show vertical (XY) plane
    pub show_yz_plane: bool,         // Show vertical (YZ) plane
    pub render_mode: GridRenderMode, // How to render the grid
}

impl Default for GridConfig {
    fn default() -> Self {
        GridConfig {
            enabled: true,
            size: 20.0,
            divisions: 20,
            xz_color: Vector3::new(0.7, 0.7, 0.7), // Light gray
            xy_color: Vector3::new(0.8, 0.6, 0.6), // Reddish
            yz_color: Vector3::new(0.6, 0.6, 0.8), // Bluish
            x_axis_color: Vector3::new(0.9, 0.2, 0.2), // Red
            y_axis_color: Vector3::new(0.2, 0.9, 0.2), // Green
            z_axis_color: Vector3::new(0.2, 0.2, 0.9), // Blue
            line_width: 1.5,
            axis_line_width: 2.5,
            show_axes: true,
            show_xz_plane: true,
            show_xy_plane: true,
            show_yz_plane: true,
            render_mode: GridRenderMode::Overlap, // Default to original behavior
        }
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub id: c_uint,
    pub width: u32,
    pub height: u32,
}

pub struct TextureManager {
    pub textures: _HashMap<c_uint, Rc<Texture>>,
    pub next_id: c_uint,
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

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
