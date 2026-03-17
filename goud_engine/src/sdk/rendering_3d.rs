//! # SDK 3D Rendering API
//!
//! Provides methods on [`GoudGame`] for 3D rendering operations
//! including primitive creation, object manipulation, lighting, camera control,
//! and scene configuration.
//!
//! # Availability
//!
//! This module requires the `native` feature (desktop platform with OpenGL).

use super::GoudGame;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::native_backend::SharedNativeRenderBackend;
use crate::libs::graphics::renderer3d::{
    FogConfig, GridConfig, Light, LightType, PrimitiveCreateInfo, PrimitiveType, SkyboxConfig,
    TextureManagerTrait,
};
use cgmath::{Vector3, Vector4};

/// Sentinel value for invalid object handles.
const INVALID_OBJECT: u32 = u32::MAX;

#[cfg(feature = "native")]
struct BackendTextureBridge {
    backend: SharedNativeRenderBackend,
}

#[cfg(feature = "native")]
impl TextureManagerTrait for BackendTextureBridge {
    fn bind_texture(&self, texture_id: u32, slot: u32) {
        let _ = self.backend.bind_texture_by_index(texture_id, slot);
    }
}

// =============================================================================
// 3D Rendering (annotated for FFI generation)
// =============================================================================

// NOTE: FFI wrappers are hand-written in ffi/renderer.rs. The `#[goud_api]`
// attribute is omitted here to avoid duplicate `#[no_mangle]` symbol conflicts.
impl GoudGame {
    /// Creates a 3D primitive and returns its object ID.
    pub fn create_primitive(&mut self, info: PrimitiveCreateInfo) -> u32 {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.create_primitive(info),
            None => INVALID_OBJECT,
        }
    }

    /// Creates a 3D cube and returns its object ID.
    pub fn create_cube(&mut self, texture_id: u32, width: f32, height: f32, depth: f32) -> u32 {
        self.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Cube,
            width,
            height,
            depth,
            segments: 1,
            texture_id,
        })
    }

    /// Creates a 3D plane and returns its object ID.
    pub fn create_plane(&mut self, texture_id: u32, width: f32, depth: f32) -> u32 {
        self.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Plane,
            width,
            height: 0.0,
            depth,
            segments: 1,
            texture_id,
        })
    }

    /// Creates a 3D sphere and returns its object ID.
    pub fn create_sphere(&mut self, texture_id: u32, diameter: f32, segments: u32) -> u32 {
        self.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Sphere,
            width: diameter,
            height: diameter,
            depth: diameter,
            segments,
            texture_id,
        })
    }

    /// Creates a 3D cylinder and returns its object ID.
    pub fn create_cylinder(
        &mut self,
        texture_id: u32,
        radius: f32,
        height: f32,
        segments: u32,
    ) -> u32 {
        self.create_primitive(PrimitiveCreateInfo {
            primitive_type: PrimitiveType::Cylinder,
            width: radius * 2.0,
            height,
            depth: radius * 2.0,
            segments,
            texture_id,
        })
    }

    /// Sets the position of a 3D object.
    pub fn set_object_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.set_object_position(id, x, y, z),
            _ => false,
        }
    }

    /// Sets the rotation of a 3D object (Euler angles in degrees).
    pub fn set_object_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.set_object_rotation(id, x, y, z),
            _ => false,
        }
    }

    /// Sets the scale of a 3D object.
    pub fn set_object_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.set_object_scale(id, x, y, z),
            None => false,
        }
    }

    /// Removes a 3D object from the scene.
    pub fn destroy_object(&mut self, object_id: u32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.remove_object(object_id),
            None => false,
        }
    }

    /// Adds a light to the 3D scene with flattened parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn add_light(
        &mut self,
        light_type: i32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        dir_x: f32,
        dir_y: f32,
        dir_z: f32,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
        range: f32,
        spot_angle: f32,
    ) -> u32 {
        let lt = match light_type {
            1 => LightType::Directional,
            2 => LightType::Spot,
            _ => LightType::Point,
        };
        match &mut self.renderer_3d {
            Some(renderer) => renderer.add_light(Light {
                light_type: lt,
                position: Vector3::new(pos_x, pos_y, pos_z),
                direction: Vector3::new(dir_x, dir_y, dir_z),
                color: Vector3::new(r, g, b),
                intensity,
                range,
                spot_angle,
                enabled: true,
            }),
            None => u32::MAX,
        }
    }

    /// Updates a light's properties.
    #[allow(clippy::too_many_arguments)]
    pub fn update_light(
        &mut self,
        light_id: u32,
        light_type: i32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
        dir_x: f32,
        dir_y: f32,
        dir_z: f32,
        r: f32,
        g: f32,
        b: f32,
        intensity: f32,
        range: f32,
        spot_angle: f32,
    ) -> bool {
        let lt = match light_type {
            1 => LightType::Directional,
            2 => LightType::Spot,
            _ => LightType::Point,
        };
        match &mut self.renderer_3d {
            Some(renderer) => renderer.update_light(
                light_id,
                Light {
                    light_type: lt,
                    position: Vector3::new(pos_x, pos_y, pos_z),
                    direction: Vector3::new(dir_x, dir_y, dir_z),
                    color: Vector3::new(r, g, b),
                    intensity,
                    range,
                    spot_angle,
                    enabled: true,
                },
            ),
            None => false,
        }
    }

    /// Removes a light from the 3D scene.
    pub fn remove_light(&mut self, light_id: u32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.remove_light(light_id),
            None => false,
        }
    }

    /// Sets the 3D camera position.
    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.set_camera_position(x, y, z);
                true
            }
            None => false,
        }
    }

    /// Sets the 3D camera rotation (pitch, yaw, roll in degrees).
    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.set_camera_rotation(pitch, yaw, roll);
                true
            }
            None => false,
        }
    }

    /// Configures the ground grid.
    pub fn configure_grid(&mut self, enabled: bool, size: f32, divisions: u32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.configure_grid(GridConfig {
                    enabled,
                    size,
                    divisions,
                    ..Default::default()
                });
                true
            }
            None => false,
        }
    }

    /// Sets grid enabled state.
    pub fn set_grid_enabled(&mut self, enabled: bool) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.set_grid_enabled(enabled);
                true
            }
            None => false,
        }
    }

    /// Configures the skybox/background color.
    pub fn configure_skybox(&mut self, enabled: bool, r: f32, g: f32, b: f32, a: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.configure_skybox(SkyboxConfig {
                    enabled,
                    color: Vector4::new(r, g, b, a),
                });
                true
            }
            None => false,
        }
    }

    /// Configures fog settings.
    pub fn configure_fog(&mut self, enabled: bool, r: f32, g: f32, b: f32, density: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.configure_fog(FogConfig {
                    enabled,
                    color: Vector3::new(r, g, b),
                    density,
                });
                true
            }
            None => false,
        }
    }

    /// Sets fog enabled state.
    pub fn set_fog_enabled(&mut self, enabled: bool) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.set_fog_enabled(enabled);
                true
            }
            None => false,
        }
    }

    /// Renders all 3D objects in the scene.
    pub fn render(&mut self) -> bool {
        match (&mut self.renderer_3d, &mut self.render_backend) {
            (Some(renderer), Some(backend)) => {
                let texture_bridge = BackendTextureBridge {
                    backend: backend.clone(),
                };
                renderer.render(Some(&texture_bridge));
                true
            }
            _ => false,
        }
    }

    /// Renders all 3D objects (alias for render).
    pub fn render_all(&mut self) -> bool {
        self.render()
    }

    /// Returns `true` if a 3D renderer is initialized.
    #[inline]
    pub fn has_3d_renderer(&self) -> bool {
        self.renderer_3d.is_some()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::GameConfig;

    #[test]
    fn test_create_cube_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert_eq!(game.create_cube(0, 1.0, 1.0, 1.0), u32::MAX);
    }

    #[test]
    fn test_set_object_position_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.set_object_position(0, 1.0, 2.0, 3.0));
    }

    #[test]
    fn test_set_camera_position_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.set_camera_position(0.0, 5.0, -10.0));
    }

    #[test]
    fn test_render_3d_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.render());
    }

    #[test]
    fn test_has_3d_renderer_headless() {
        let game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.has_3d_renderer());
    }

    #[test]
    fn test_add_light_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        let id = game.add_light(
            0, 0.0, 5.0, 0.0, 0.0, -1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 10.0, 0.0,
        );
        assert_eq!(id, u32::MAX);
    }

    #[test]
    fn test_render_all_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.render_all());
    }

    #[test]
    fn test_configure_grid_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.configure_grid(true, 10.0, 10));
    }

    #[test]
    fn test_configure_skybox_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.configure_skybox(true, 0.5, 0.5, 0.8, 1.0));
    }

    #[test]
    fn test_configure_fog_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.configure_fog(true, 0.5, 0.5, 0.5, 0.01));
    }

    #[test]
    fn test_set_fog_enabled_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.set_fog_enabled(true));
    }

    #[test]
    fn test_destroy_object_headless() {
        let mut game = GoudGame::new(GameConfig::default()).unwrap();
        assert!(!game.destroy_object(0));
    }
}
