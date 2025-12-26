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

/// Configuration for the skybox
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SkyboxConfig {
    pub enabled: bool,
    pub size: f32,                      // Scale of the skybox
    pub texture_size: u32,              // Size of each face texture
    pub face_colors: [Vector3<f32>; 6], // Colors for each face [right, left, top, bottom, front, back]
    pub blend_factor: f32,              // How much to blend between faces (0.0 - 1.0)
    pub min_color: Vector3<f32>,        // Minimum color values to prevent pure black
    pub use_custom_textures: bool,      // Whether to use custom textures or generated gradients
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        SkyboxConfig {
            enabled: true,
            size: 100.0,
            texture_size: 128,
            face_colors: [
                Vector3::new(0.7, 0.8, 0.9), // Right face
                Vector3::new(0.7, 0.8, 0.9), // Left face
                Vector3::new(0.6, 0.7, 0.9), // Top face
                Vector3::new(0.3, 0.3, 0.4), // Bottom face
                Vector3::new(0.7, 0.8, 0.9), // Front face
                Vector3::new(0.7, 0.8, 0.9), // Back face
            ],
            blend_factor: 0.5,
            min_color: Vector3::new(0.1, 0.1, 0.2),
            use_custom_textures: false,
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

/// Base 2D camera struct
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Camera2D {
    pub position: Vector3<f32>,
    pub zoom: f32,
}

/// 3D camera struct with additional properties for 3D rendering
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Camera3D {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,
    pub zoom: f32,
    pub rotation: Vector3<f32>, // Euler angles (pitch, yaw, roll) in degrees
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::mem;

    #[test]
    fn test_grid_render_mode_enum() {
        let blend = GridRenderMode::Blend;
        let overlap = GridRenderMode::Overlap;

        assert_eq!(blend, GridRenderMode::Blend);
        assert_eq!(overlap, GridRenderMode::Overlap);
        assert_ne!(blend, overlap);

        // Test Debug trait
        assert_eq!(format!("{:?}", blend), "Blend");
        assert_eq!(format!("{:?}", overlap), "Overlap");

        // Test Copy trait
        let blend_copy = blend;
        assert_eq!(blend, blend_copy);
    }

    #[test]
    fn test_grid_config_default() {
        let config = GridConfig::default();

        assert!(config.enabled);
        assert_eq!(config.size, 20.0);
        assert_eq!(config.divisions, 20);
        assert_eq!(config.xz_color, Vector3::new(0.7, 0.7, 0.7));
        assert_eq!(config.xy_color, Vector3::new(0.8, 0.6, 0.6));
        assert_eq!(config.yz_color, Vector3::new(0.6, 0.6, 0.8));
        assert_eq!(config.x_axis_color, Vector3::new(0.9, 0.2, 0.2));
        assert_eq!(config.y_axis_color, Vector3::new(0.2, 0.9, 0.2));
        assert_eq!(config.z_axis_color, Vector3::new(0.2, 0.2, 0.9));
        assert_eq!(config.line_width, 1.5);
        assert_eq!(config.axis_line_width, 2.5);
        assert!(config.show_axes);
        assert!(config.show_xz_plane);
        assert!(config.show_xy_plane);
        assert!(config.show_yz_plane);
        assert_eq!(config.render_mode, GridRenderMode::Overlap);
    }

    #[test]
    fn test_grid_config_modification() {
        let config = GridConfig {
            enabled: false,
            size: 50.0,
            divisions: 40,
            render_mode: GridRenderMode::Blend,
            ..Default::default()
        };

        assert!(!config.enabled);
        assert_eq!(config.size, 50.0);
        assert_eq!(config.divisions, 40);
        assert_eq!(config.render_mode, GridRenderMode::Blend);
    }

    #[test]
    fn test_grid_config_clone() {
        let config = GridConfig::default();
        let cloned = config.clone();

        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.size, cloned.size);
        assert_eq!(config.render_mode, cloned.render_mode);
    }

    #[test]
    fn test_skybox_config_default() {
        let config = SkyboxConfig::default();

        assert!(config.enabled);
        assert_eq!(config.size, 100.0);
        assert_eq!(config.texture_size, 128);
        assert_eq!(config.blend_factor, 0.5);
        assert_eq!(config.min_color, Vector3::new(0.1, 0.1, 0.2));
        assert!(!config.use_custom_textures);

        // Test face colors array
        assert_eq!(config.face_colors.len(), 6);
        assert_eq!(config.face_colors[0], Vector3::new(0.7, 0.8, 0.9)); // Right
        assert_eq!(config.face_colors[3], Vector3::new(0.3, 0.3, 0.4)); // Bottom
    }

    #[test]
    fn test_texture_struct_fields() {
        // We can't create actual Texture instances in unit tests because
        // the Drop implementation requires OpenGL context.
        // Instead, we test that the struct has the expected fields and traits.
        use std::ffi::c_uint;

        // Test that fields are accessible (compile-time test)
        let _ = |texture: &Texture| {
            let _id: c_uint = texture.id;
            let _width: u32 = texture.width;
            let _height: u32 = texture.height;
        };

        // Test that Texture implements expected traits
        fn assert_traits<T: Clone + Debug>() {}
        assert_traits::<Texture>();

        // Test memory layout
        assert_eq!(
            mem::size_of::<Texture>(),
            mem::size_of::<c_uint>() + 2 * mem::size_of::<u32>()
        );
    }

    #[test]
    fn test_rectangle_struct() {
        let rect = Rectangle {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 200.0,
        };

        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 200.0);

        // Test Copy trait
        let rect_copy = rect;
        assert_eq!(rect.x, rect_copy.x);
        assert_eq!(rect.y, rect_copy.y);
    }

    #[test]
    fn test_sprite_struct() {
        let sprite = Sprite {
            id: 1,
            x: 100.0,
            y: 200.0,
            z_layer: 5,
            scale_x: 1.5,
            scale_y: 2.0,
            dimension_x: 64.0,
            dimension_y: 64.0,
            rotation: 45.0,
            source_rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            texture_id: 10,
            debug: false,
            frame: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 64.0,
                height: 64.0,
            },
        };

        assert_eq!(sprite.id, 1);
        assert_eq!(sprite.x, 100.0);
        assert_eq!(sprite.z_layer, 5);
        assert_eq!(sprite.rotation, 45.0);
        assert_eq!(sprite.texture_id, 10);
        assert!(!sprite.debug);

        // Test Clone
        let cloned = sprite.clone();
        assert_eq!(sprite.id, cloned.id);
        assert_eq!(sprite.x, cloned.x);
        assert_eq!(sprite.source_rect.width, cloned.source_rect.width);
    }

    #[test]
    fn test_sprite_create_dto() {
        let dto = SpriteCreateDto {
            x: 50.0,
            y: 75.0,
            z_layer: 2,
            scale_x: 1.0,
            scale_y: 1.0,
            dimension_x: 32.0,
            dimension_y: 32.0,
            rotation: 0.0,
            source_rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 16.0,
                height: 16.0,
            },
            texture_id: 5,
            debug: true,
            frame: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
        };

        assert_eq!(dto.x, 50.0);
        assert_eq!(dto.y, 75.0);
        assert_eq!(dto.z_layer, 2);
        assert!(dto.debug);
    }

    #[test]
    fn test_sprite_update_dto() {
        let dto = SpriteUpdateDto {
            id: 99,
            x: 150.0,
            y: 250.0,
            z_layer: 3,
            scale_x: 2.0,
            scale_y: 2.0,
            dimension_x: 128.0,
            dimension_y: 128.0,
            rotation: 90.0,
            source_rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 64.0,
                height: 64.0,
            },
            texture_id: 20,
            debug: false,
            frame: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 128.0,
                height: 128.0,
            },
        };

        assert_eq!(dto.id, 99);
        assert_eq!(dto.x, 150.0);
        assert_eq!(dto.rotation, 90.0);
    }

    #[test]
    fn test_update_response_data() {
        let data = UpdateResponseData {
            delta_time: 0.016666,
        };

        assert_eq!(data.delta_time, 0.016666);
    }

    #[test]
    fn test_mouse_position() {
        let pos = MousePosition {
            x: 400.5,
            y: 300.25,
        };

        assert_eq!(pos.x, 400.5);
        assert_eq!(pos.y, 300.25);

        // Test Copy trait
        let pos_copy = pos;
        assert_eq!(pos.x, pos_copy.x);
        assert_eq!(pos.y, pos_copy.y);
    }

    #[test]
    fn test_camera2d() {
        let camera = Camera2D {
            position: Vector3::new(100.0, 200.0, 0.0),
            zoom: 2.0,
        };

        assert_eq!(camera.position.x, 100.0);
        assert_eq!(camera.position.y, 200.0);
        assert_eq!(camera.position.z, 0.0);
        assert_eq!(camera.zoom, 2.0);

        // Test Clone
        let cloned = camera.clone();
        assert_eq!(camera.position.x, cloned.position.x);
        assert_eq!(camera.zoom, cloned.zoom);
    }

    #[test]
    fn test_camera3d() {
        let camera = Camera3D {
            position: Vector3::new(10.0, 20.0, 30.0),
            target: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            zoom: 1.5,
            rotation: Vector3::new(45.0, 90.0, 0.0),
        };

        assert_eq!(camera.position.x, 10.0);
        assert_eq!(camera.target.x, 0.0);
        assert_eq!(camera.up.y, 1.0);
        assert_eq!(camera.zoom, 1.5);
        assert_eq!(camera.rotation.x, 45.0);

        // Test Clone
        let cloned = camera.clone();
        assert_eq!(camera.position.x, cloned.position.x);
        assert_eq!(camera.rotation.y, cloned.rotation.y);
    }

    #[test]
    fn test_ffi_repr_c_sizes() {
        // Ensure FFI types have expected memory layouts
        // These assertions help validate FFI compatibility

        // GridRenderMode should be size of C enum (usually 4 bytes)
        assert!(mem::size_of::<GridRenderMode>() <= mem::size_of::<c_uint>());

        // Rectangle should be tightly packed (4 floats)
        assert_eq!(mem::size_of::<Rectangle>(), 4 * mem::size_of::<f32>());

        // MousePosition should be 2 doubles
        assert_eq!(mem::size_of::<MousePosition>(), 2 * mem::size_of::<f64>());

        // UpdateResponseData should be 1 float
        assert_eq!(mem::size_of::<UpdateResponseData>(), mem::size_of::<f32>());
    }

    #[test]
    fn test_texture_manager_creation() {
        let manager = TextureManager {
            textures: _HashMap::new(),
            next_id: 1,
        };

        assert_eq!(manager.next_id, 1);
        assert_eq!(manager.textures.len(), 0);
    }

    #[test]
    fn test_tiled_struct() {
        // We can only test the fields we control directly
        // Map requires loading from a file, so we skip testing with actual Map
        use std::ffi::c_uint;

        // Test that we can access the fields (compile-time test)
        let _ = |tiled: &Tiled| {
            let _id: c_uint = tiled.id;
            let _texture_ids: &Vec<c_uint> = &tiled.texture_ids;
            let _map: &Rc<Map> = &tiled.map;
        };
    }

    #[test]
    fn test_tiled_manager_creation() {
        let manager = TiledManager {
            selected_map_id: Some(5),
            loader: Loader::new(),
            maps: _HashMap::new(),
        };

        assert_eq!(manager.selected_map_id, Some(5));
        assert_eq!(manager.maps.len(), 0);
    }

    #[test]
    fn test_sprite_map_type() {
        let mut sprite_map: SpriteMap = BTreeMap::new();

        let sprite = Sprite {
            id: 1,
            x: 0.0,
            y: 0.0,
            z_layer: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            dimension_x: 32.0,
            dimension_y: 32.0,
            rotation: 0.0,
            source_rect: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
            texture_id: 1,
            debug: false,
            frame: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 32.0,
                height: 32.0,
            },
        };

        sprite_map.entry(0).or_default().push(sprite);

        assert_eq!(sprite_map.len(), 1);
        assert_eq!(sprite_map.get(&0).unwrap().len(), 1);
    }
}
