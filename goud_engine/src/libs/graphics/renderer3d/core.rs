//! Core [`Renderer3D`] struct, constructor, and object/light/camera manipulation.

use std::collections::HashMap;

use super::scene::Scene3D;
use crate::core::providers::types::DebugShape3D;
use crate::libs::graphics::backend::BufferHandle;
use crate::libs::graphics::backend::BufferType;
use crate::libs::graphics::backend::BufferUsage;
use crate::libs::graphics::backend::{RenderBackend, ShaderLanguage, VertexLayout};

use super::animation::AnimationPlayer;
use super::debug_draw::build_debug_draw_vertices;
use super::mesh::generate_plane_vertices;
use super::mesh::{
    create_axis_mesh, create_grid_mesh, create_postprocess_quad, grid_vertex_layout,
    instance_vertex_layout, object_vertex_layout, postprocess_vertex_layout, skinned_vertex_layout,
    upload_buffer,
};
use super::model::{Model3D, ModelInstance3D};
use super::shaders::{
    resolve_grid_uniforms, resolve_main_uniforms, resolve_skinned_uniforms, GridUniforms,
    MainUniforms, SkinnedUniforms, FRAGMENT_SHADER_3D, FRAGMENT_SHADER_3D_WGSL,
    GRID_FRAGMENT_SHADER, GRID_FRAGMENT_SHADER_WGSL, GRID_VERTEX_SHADER, GRID_VERTEX_SHADER_WGSL,
    INSTANCED_FRAGMENT_SHADER_3D, INSTANCED_FRAGMENT_SHADER_3D_WGSL, INSTANCED_VERTEX_SHADER_3D,
    INSTANCED_VERTEX_SHADER_3D_WGSL, POSTPROCESS_FRAGMENT_SHADER, POSTPROCESS_FRAGMENT_SHADER_WGSL,
    POSTPROCESS_VERTEX_SHADER, POSTPROCESS_VERTEX_SHADER_WGSL, SKINNED_VERTEX_SHADER,
    SKINNED_VERTEX_SHADER_WGSL, VERTEX_SHADER_3D, VERTEX_SHADER_3D_WGSL,
};
use super::types::{
    AntiAliasingMode, Camera3D, FogConfig, GridConfig, InstancedMesh, Light, Material3D, Object3D,
    ParticleEmitter, PostProcessPipeline, Renderer3DStats, SkinnedMesh3D, SkyboxConfig,
};

use crate::libs::graphics::backend::ShaderHandle;
use cgmath::Vector3;

// ============================================================================
// Renderer3D
// ============================================================================

/// The main 3D renderer.
///
/// Owns a [`RenderBackend`] and manages all GPU resources (shaders, buffers)
/// through it. No direct graphics API calls are made outside the backend.
pub struct Renderer3D {
    pub(super) backend: Box<dyn RenderBackend>,
    pub(super) shader_handle: ShaderHandle,
    pub(super) instanced_shader_handle: ShaderHandle,
    pub(super) grid_shader_handle: ShaderHandle,
    pub(super) postprocess_shader_handle: ShaderHandle,
    pub(super) grid_buffer: BufferHandle,
    pub(super) grid_vertex_count: i32,
    pub(super) axis_buffer: BufferHandle,
    pub(super) axis_vertex_count: i32,
    pub(super) debug_draw_buffer: BufferHandle,
    pub(super) debug_draw_buffer_capacity_bytes: usize,
    pub(super) debug_draw_vertex_count: i32,
    pub(super) objects: HashMap<u32, Object3D>,
    pub(super) instanced_meshes: HashMap<u32, InstancedMesh>,
    pub(super) particle_emitters: HashMap<u32, ParticleEmitter>,
    pub(super) lights: HashMap<u32, Light>,
    pub(super) next_object_id: u32,
    pub(super) next_instanced_mesh_id: u32,
    pub(super) next_light_id: u32,
    pub(super) next_particle_emitter_id: u32,
    pub(super) camera: Camera3D,
    pub(super) window_width: u32,
    pub(super) window_height: u32,
    pub(super) viewport: (i32, i32, u32, u32),
    pub(super) grid_config: GridConfig,
    pub(super) skybox_config: SkyboxConfig,
    pub(super) fog_config: FogConfig,
    pub(super) uniforms: MainUniforms,
    pub(super) instanced_uniforms: MainUniforms,
    pub(super) grid_uniforms: GridUniforms,
    pub(super) object_layout: VertexLayout,
    pub(super) instance_layout: VertexLayout,
    pub(super) grid_layout: VertexLayout,
    pub(super) particle_quad_buffer: BufferHandle,
    pub(super) particle_quad_vertex_count: u32,
    pub(super) postprocess_quad_buffer: BufferHandle,
    pub(super) postprocess_layout: VertexLayout,
    pub(super) postprocess_texture: Option<crate::libs::graphics::backend::TextureHandle>,
    pub(super) postprocess_texture_size: (u32, u32),
    pub(super) shadow_texture: Option<crate::libs::graphics::backend::TextureHandle>,
    pub(super) shadow_map_size: u32,
    pub(super) shadow_bias: f32,
    pub(super) materials: HashMap<u32, Material3D>,
    pub(super) object_materials: HashMap<u32, u32>,
    pub(super) next_material_id: u32,
    pub(super) skinned_meshes: HashMap<u32, SkinnedMesh3D>,
    pub(super) next_skinned_mesh_id: u32,
    pub(super) skinned_shader_handle: ShaderHandle,
    pub(super) skinned_uniforms: SkinnedUniforms,
    pub(super) skinned_layout: VertexLayout,
    pub(super) models: HashMap<u32, Model3D>,
    pub(super) model_instances: HashMap<u32, ModelInstance3D>,
    pub(super) next_model_id: u32,
    pub(super) animation_players: HashMap<u32, AnimationPlayer>,
    pub(super) postprocess_pipeline: PostProcessPipeline,
    pub(super) stats: Renderer3DStats,
    pub(super) anti_aliasing_mode: AntiAliasingMode,
    pub(super) msaa_samples: u32,
    pub(super) scenes: HashMap<u32, Scene3D>,
    pub(super) next_scene_id: u32,
    pub(super) current_scene: Option<u32>,
}

impl Renderer3D {
    /// Create a new 3D renderer with the given backend.
    pub fn new(
        mut backend: Box<dyn RenderBackend>,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        let (vertex_shader, fragment_shader) = match backend.shader_language() {
            ShaderLanguage::Wgsl => (VERTEX_SHADER_3D_WGSL, FRAGMENT_SHADER_3D_WGSL),
            ShaderLanguage::Glsl => (VERTEX_SHADER_3D, FRAGMENT_SHADER_3D),
        };
        let (instanced_vertex_shader, instanced_fragment_shader) = match backend.shader_language() {
            ShaderLanguage::Wgsl => (
                INSTANCED_VERTEX_SHADER_3D_WGSL,
                INSTANCED_FRAGMENT_SHADER_3D_WGSL,
            ),
            ShaderLanguage::Glsl => (INSTANCED_VERTEX_SHADER_3D, INSTANCED_FRAGMENT_SHADER_3D),
        };
        let (grid_vertex_shader, grid_fragment_shader) = match backend.shader_language() {
            ShaderLanguage::Wgsl => (GRID_VERTEX_SHADER_WGSL, GRID_FRAGMENT_SHADER_WGSL),
            ShaderLanguage::Glsl => (GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER),
        };
        let (postprocess_vertex_shader, postprocess_fragment_shader) =
            match backend.shader_language() {
                ShaderLanguage::Wgsl => (
                    POSTPROCESS_VERTEX_SHADER_WGSL,
                    POSTPROCESS_FRAGMENT_SHADER_WGSL,
                ),
                ShaderLanguage::Glsl => (POSTPROCESS_VERTEX_SHADER, POSTPROCESS_FRAGMENT_SHADER),
            };

        let shader_handle = backend
            .create_shader(vertex_shader, fragment_shader)
            .map_err(|e| format!("Main 3D shader: {e}"))?;
        let uniforms = resolve_main_uniforms(backend.as_ref(), shader_handle);

        let instanced_shader_handle = backend
            .create_shader(instanced_vertex_shader, instanced_fragment_shader)
            .map_err(|e| format!("Instanced 3D shader: {e}"))?;
        let instanced_uniforms = resolve_main_uniforms(backend.as_ref(), instanced_shader_handle);

        let grid_shader_handle = backend
            .create_shader(grid_vertex_shader, grid_fragment_shader)
            .map_err(|e| format!("Grid shader: {e}"))?;
        let grid_uniforms = resolve_grid_uniforms(backend.as_ref(), grid_shader_handle);
        let postprocess_shader_handle = backend
            .create_shader(postprocess_vertex_shader, postprocess_fragment_shader)
            .map_err(|e| format!("Postprocess shader: {e}"))?;

        // Skinned mesh shader — uses the same fragment shader as the main 3D shader.
        let (skinned_vertex_src, skinned_fragment_src) = match backend.shader_language() {
            ShaderLanguage::Wgsl => (SKINNED_VERTEX_SHADER_WGSL, FRAGMENT_SHADER_3D_WGSL),
            ShaderLanguage::Glsl => (SKINNED_VERTEX_SHADER, FRAGMENT_SHADER_3D),
        };
        let skinned_shader_handle = backend
            .create_shader(skinned_vertex_src, skinned_fragment_src)
            .map_err(|e| format!("Skinned 3D shader: {e}"))?;
        let skinned_uniforms = resolve_skinned_uniforms(backend.as_ref(), skinned_shader_handle);

        let grid_layout = grid_vertex_layout();
        let instance_layout = instance_vertex_layout();
        let postprocess_layout = postprocess_vertex_layout();
        let (grid_buffer, grid_vertex_count) = create_grid_mesh(backend.as_mut(), 20.0, 20)?;
        let (axis_buffer, axis_vertex_count) = create_axis_mesh(backend.as_mut(), 5.0)?;
        let particle_vertices = generate_plane_vertices(1.0, 1.0);
        let particle_quad_vertex_count = (particle_vertices.len() / 8) as u32;
        let particle_quad_buffer = upload_buffer(backend.as_mut(), &particle_vertices)
            .map_err(|e| format!("Particle quad buffer: {e}"))?;
        let postprocess_quad = create_postprocess_quad();
        let postprocess_quad_buffer = upload_buffer(backend.as_mut(), &postprocess_quad)
            .map_err(|e| format!("Postprocess quad buffer: {e}"))?;

        Ok(Self {
            backend,
            shader_handle,
            instanced_shader_handle,
            grid_shader_handle,
            postprocess_shader_handle,
            grid_buffer,
            grid_vertex_count,
            axis_buffer,
            axis_vertex_count,
            debug_draw_buffer: BufferHandle::INVALID,
            debug_draw_buffer_capacity_bytes: 0,
            debug_draw_vertex_count: 0,
            objects: HashMap::new(),
            instanced_meshes: HashMap::new(),
            particle_emitters: HashMap::new(),
            lights: HashMap::new(),
            next_object_id: 1,
            next_instanced_mesh_id: 1,
            next_light_id: 1,
            next_particle_emitter_id: 1,
            camera: Camera3D::default(),
            window_width,
            window_height,
            viewport: (0, 0, window_width.max(1), window_height.max(1)),
            grid_config: GridConfig::default(),
            skybox_config: SkyboxConfig::default(),
            fog_config: FogConfig::default(),
            uniforms,
            instanced_uniforms,
            grid_uniforms,
            object_layout: object_vertex_layout(),
            instance_layout,
            grid_layout,
            particle_quad_buffer,
            particle_quad_vertex_count,
            postprocess_quad_buffer,
            postprocess_layout,
            postprocess_texture: None,
            postprocess_texture_size: (0, 0),
            shadow_texture: None,
            shadow_map_size: 256,
            shadow_bias: 0.005,
            materials: HashMap::new(),
            object_materials: HashMap::new(),
            next_material_id: 1,
            skinned_meshes: HashMap::new(),
            next_skinned_mesh_id: 1,
            skinned_shader_handle,
            skinned_uniforms,
            skinned_layout: skinned_vertex_layout(),
            models: HashMap::new(),
            model_instances: HashMap::new(),
            next_model_id: 1,
            animation_players: HashMap::new(),
            postprocess_pipeline: PostProcessPipeline::new(),
            stats: Renderer3DStats::default(),
            anti_aliasing_mode: AntiAliasingMode::Off,
            msaa_samples: 1,
            scenes: HashMap::new(),
            next_scene_id: 1,
            current_scene: None,
        })
    }

    /// Set object position
    pub fn set_object_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.position = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Set object rotation (in degrees)
    pub fn set_object_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.rotation = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Set object scale
    pub fn set_object_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.scale = Vector3::new(x, y, z);
            true
        } else {
            false
        }
    }

    /// Remove an object
    pub fn remove_object(&mut self, id: u32) -> bool {
        if let Some(obj) = self.objects.remove(&id) {
            self.backend.destroy_buffer(obj.buffer);
            true
        } else {
            false
        }
    }

    // Lighting

    /// Add a light and return its ID.
    pub fn add_light(&mut self, light: Light) -> u32 {
        let id = self.next_light_id;
        self.next_light_id += 1;
        self.lights.insert(id, light);
        id
    }

    /// Update a light by ID. Returns `true` if the light existed.
    pub fn update_light(&mut self, id: u32, light: Light) -> bool {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(mut e) = self.lights.entry(id) {
            e.insert(light);
            true
        } else {
            false
        }
    }

    /// Remove a light by ID. Returns `true` if the light existed.
    pub fn remove_light(&mut self, id: u32) -> bool {
        self.lights.remove(&id).is_some()
    }

    // Camera

    /// Set camera position
    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) {
        self.camera.position = Vector3::new(x, y, z);
    }

    /// Updates the framebuffer dimensions used for projection.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
    }

    /// Sets the active viewport rectangle for rendering.
    pub fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.viewport = (x, y, width.max(1), height.max(1));
    }

    /// Set camera rotation (pitch, yaw, roll in degrees)
    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.camera.rotation = Vector3::new(pitch, yaw, roll);
    }

    /// Returns the last frame's renderer stats.
    pub fn stats(&self) -> Renderer3DStats {
        self.stats
    }

    /// Returns the active anti-aliasing mode.
    pub fn anti_aliasing_mode(&self) -> AntiAliasingMode {
        self.anti_aliasing_mode
    }

    /// Returns the configured MSAA sample count.
    pub fn msaa_samples(&self) -> u32 {
        self.msaa_samples
    }

    /// Updates the anti-aliasing mode.
    pub fn set_anti_aliasing_mode(&mut self, mode: AntiAliasingMode) -> Result<(), String> {
        self.anti_aliasing_mode = mode;
        self.backend.set_multisampling_enabled(mode.uses_msaa());
        Ok(())
    }

    /// Updates the requested MSAA sample count.
    pub fn set_msaa_samples(&mut self, samples: u32) {
        self.msaa_samples = match samples {
            2 | 4 | 8 => samples,
            _ => 1,
        };
    }

    /// Sets the directional shadow bias.
    pub fn set_shadow_bias(&mut self, bias: f32) {
        self.shadow_bias = bias.max(0.0);
    }

    /// Returns the directional shadow bias.
    pub fn shadow_bias(&self) -> f32 {
        self.shadow_bias
    }

    // Grid / Skybox / Fog configuration

    /// Configure grid
    pub fn configure_grid(&mut self, config: GridConfig) {
        if config.size != self.grid_config.size || config.divisions != self.grid_config.divisions {
            self.backend.destroy_buffer(self.grid_buffer);
            if let Ok((buf, count)) =
                create_grid_mesh(self.backend.as_mut(), config.size, config.divisions)
            {
                self.grid_buffer = buf;
                self.grid_vertex_count = count;
            }
        }
        self.grid_config = config;
    }

    /// Set grid enabled state
    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_config.enabled = enabled;
    }

    /// Configure skybox
    pub fn configure_skybox(&mut self, config: SkyboxConfig) {
        self.skybox_config = config;
    }

    /// Configure fog
    pub fn configure_fog(&mut self, config: FogConfig) {
        self.fog_config = config;
    }

    /// Set fog enabled state
    pub fn set_fog_enabled(&mut self, enabled: bool) {
        self.fog_config.enabled = enabled;
    }

    /// Upload debug draw shapes as line vertices (position + color) for rendering.
    pub fn set_debug_draw_shapes(&mut self, shapes: &[DebugShape3D]) {
        let vertices = build_debug_draw_vertices(shapes);
        if vertices.is_empty() {
            self.debug_draw_vertex_count = 0;
            return;
        }

        let vertex_bytes = bytemuck::cast_slice(vertices.as_slice());
        let required_bytes = vertex_bytes.len();

        let needs_new_buffer = !self.backend.is_buffer_valid(self.debug_draw_buffer)
            || self.debug_draw_buffer_capacity_bytes < required_bytes;

        if needs_new_buffer {
            if self.backend.is_buffer_valid(self.debug_draw_buffer) {
                self.backend.destroy_buffer(self.debug_draw_buffer);
                self.debug_draw_buffer = BufferHandle::INVALID;
                self.debug_draw_buffer_capacity_bytes = 0;
            }

            match self
                .backend
                .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, vertex_bytes)
            {
                Ok(buffer) => {
                    self.debug_draw_buffer = buffer;
                    self.debug_draw_buffer_capacity_bytes = required_bytes;
                }
                Err(e) => {
                    log::error!("Failed to create debug draw buffer: {e}");
                    self.debug_draw_vertex_count = 0;
                    return;
                }
            }
        } else if let Err(e) = self
            .backend
            .update_buffer(self.debug_draw_buffer, 0, vertex_bytes)
        {
            log::error!("Failed to update debug draw buffer: {e}");
            self.debug_draw_vertex_count = 0;
            return;
        }

        self.debug_draw_vertex_count = (vertices.len() / 6) as i32;
    }
}

impl Drop for Renderer3D {
    fn drop(&mut self) {
        for obj in self.objects.values() {
            self.backend.destroy_buffer(obj.buffer);
        }
        for mesh in self.instanced_meshes.values() {
            self.backend.destroy_buffer(mesh.mesh_buffer);
            self.backend.destroy_buffer(mesh.instance_buffer);
        }
        for emitter in self.particle_emitters.values() {
            self.backend.destroy_buffer(emitter.instance_buffer);
        }
        for mesh in self.skinned_meshes.values() {
            self.backend.destroy_buffer(mesh.buffer);
        }
        self.backend.destroy_buffer(self.grid_buffer);
        self.backend.destroy_buffer(self.axis_buffer);
        self.backend.destroy_buffer(self.particle_quad_buffer);
        self.backend.destroy_buffer(self.postprocess_quad_buffer);
        if let Some(texture) = self.postprocess_texture.take() {
            self.backend.destroy_texture(texture);
        }
        if let Some(texture) = self.shadow_texture.take() {
            self.backend.destroy_texture(texture);
        }
        if self.backend.is_buffer_valid(self.debug_draw_buffer) {
            self.backend.destroy_buffer(self.debug_draw_buffer);
        }
        self.backend.destroy_shader(self.shader_handle);
        self.backend.destroy_shader(self.instanced_shader_handle);
        self.backend.destroy_shader(self.grid_shader_handle);
        self.backend.destroy_shader(self.postprocess_shader_handle);
        self.backend.destroy_shader(self.skinned_shader_handle);
    }
}
