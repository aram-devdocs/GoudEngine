//! SDK 3D Rendering API: primitives, objects, lighting, camera, and scene configuration.
//! Requires the `native` feature (desktop platform).

use super::GoudGame;
#[cfg(feature = "native")]
use crate::libs::graphics::backend::native_backend::SharedNativeRenderBackend;
use crate::libs::graphics::renderer3d::{
    AntiAliasingMode, FogConfig, FogMode, GridConfig, InstanceTransform, Light, LightType,
    ParticleEmitterConfig, PrimitiveCreateInfo, PrimitiveType, Renderer3DStats, SkyboxConfig,
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

    /// Creates an instanced 3D primitive and returns its object ID.
    pub fn create_instanced_primitive(
        &mut self,
        info: PrimitiveCreateInfo,
        instances: &[InstanceTransform],
    ) -> u32 {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.create_instanced_primitive(info, instances),
            None => INVALID_OBJECT,
        }
    }

    /// Creates an instanced cube and returns its object ID.
    pub fn create_instanced_cube(
        &mut self,
        texture_id: u32,
        width: f32,
        height: f32,
        depth: f32,
        instances: &[InstanceTransform],
    ) -> u32 {
        self.create_instanced_primitive(
            PrimitiveCreateInfo {
                primitive_type: PrimitiveType::Cube,
                width,
                height,
                depth,
                segments: 1,
                texture_id,
            },
            instances,
        )
    }

    /// Replaces the instances stored by an instanced primitive.
    pub fn set_instanced_mesh_instances(
        &mut self,
        id: u32,
        instances: &[InstanceTransform],
    ) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.set_instanced_mesh_instances(id, instances),
            None => false,
        }
    }

    /// Removes an instanced primitive from the scene.
    pub fn destroy_instanced_mesh(&mut self, id: u32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.remove_instanced_mesh(id),
            None => false,
        }
    }

    /// Creates a particle emitter and returns its ID.
    pub fn create_particle_emitter(&mut self, config: ParticleEmitterConfig) -> u32 {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.create_particle_emitter(config),
            None => INVALID_OBJECT,
        }
    }

    /// Sets particle emitter origin.
    pub fn set_particle_emitter_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.set_particle_emitter_position(id, x, y, z),
            None => false,
        }
    }

    /// Removes a particle emitter.
    pub fn destroy_particle_emitter(&mut self, id: u32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => renderer.remove_particle_emitter(id),
            None => false,
        }
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

    /// Configures exponential fog settings.
    pub fn configure_fog(&mut self, enabled: bool, r: f32, g: f32, b: f32, density: f32) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.configure_fog(FogConfig {
                    enabled,
                    color: Vector3::new(r, g, b),
                    mode: FogMode::Exponential { density },
                });
                true
            }
            None => false,
        }
    }

    /// Configures linear fog with explicit start/end distances.
    pub fn configure_fog_linear(
        &mut self,
        enabled: bool,
        start_distance: f32,
        end_distance: f32,
        r: f32,
        g: f32,
        b: f32,
    ) -> bool {
        match &mut self.renderer_3d {
            Some(renderer) => {
                renderer.configure_fog(FogConfig {
                    enabled,
                    color: Vector3::new(r, g, b),
                    mode: FogMode::Linear {
                        start: start_distance,
                        end: end_distance,
                    },
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
        let viewport = self.render_viewport();
        match (&mut self.renderer_3d, &mut self.render_backend) {
            (Some(renderer), Some(backend)) => {
                let texture_bridge = BackendTextureBridge {
                    backend: backend.clone(),
                };
                renderer.update(self.context.delta_time());
                renderer.set_viewport(viewport.x, viewport.y, viewport.width, viewport.height);
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

    /// Returns last-frame 3D renderer stats when available.
    pub fn renderer_3d_stats(&self) -> Option<Renderer3DStats> {
        self.renderer_3d.as_ref().map(|renderer| renderer.stats())
    }

    /// Returns the active 3D anti-aliasing mode when a renderer exists.
    pub fn anti_aliasing_mode(&self) -> Option<AntiAliasingMode> {
        self.renderer_3d
            .as_ref()
            .map(|renderer| renderer.anti_aliasing_mode())
    }

    /// Updates the 3D anti-aliasing mode at runtime.
    pub fn set_anti_aliasing_mode(&mut self, mode: AntiAliasingMode) -> bool {
        self.config.anti_aliasing_mode = mode;
        match &mut self.renderer_3d {
            Some(renderer) => renderer.set_anti_aliasing_mode(mode).is_ok(),
            None => false,
        }
    }

    /// Returns the configured MSAA sample count.
    pub fn msaa_samples(&self) -> u32 {
        self.config.msaa_samples
    }

    /// Updates the stored MSAA sample count.
    pub fn set_msaa_samples(&mut self, samples: u32) {
        self.config.msaa_samples = match samples {
            2 | 4 | 8 => samples,
            _ => 1,
        };
        if let Some(renderer) = self.renderer_3d.as_mut() {
            renderer.set_msaa_samples(self.config.msaa_samples);
        }
    }

    /// Updates the shadow bias used for directional-light shadows.
    pub fn set_shadow_bias(&mut self, bias: f32) -> bool {
        match self.renderer_3d.as_mut() {
            Some(renderer) => {
                renderer.set_shadow_bias(bias);
                true
            }
            None => false,
        }
    }

    /// Returns the current directional shadow bias when available.
    pub fn shadow_bias(&self) -> Option<f32> {
        self.renderer_3d
            .as_ref()
            .map(|renderer| renderer.shadow_bias())
    }
}

#[cfg(test)]
mod tests;
