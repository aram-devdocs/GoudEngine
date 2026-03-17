//! Core [`Renderer3D`] struct, constructor, and object/light/camera manipulation.

use std::collections::HashMap;

use crate::core::providers::types::DebugShape3D;
use crate::libs::graphics::backend::BufferHandle;
use crate::libs::graphics::backend::BufferType;
use crate::libs::graphics::backend::BufferUsage;
use crate::libs::graphics::backend::{RenderBackend, VertexLayout};

use super::debug_draw::build_debug_draw_vertices;
use super::mesh::{create_axis_mesh, create_grid_mesh, grid_vertex_layout, object_vertex_layout};
use super::mesh::{
    generate_cube_vertices, generate_cylinder_vertices, generate_plane_vertices,
    generate_sphere_vertices, upload_buffer,
};
use super::shaders::{
    resolve_grid_uniforms, resolve_main_uniforms, GridUniforms, MainUniforms, FRAGMENT_SHADER_3D,
    FRAGMENT_SHADER_3D_WGSL, GRID_FRAGMENT_SHADER, GRID_FRAGMENT_SHADER_WGSL, GRID_VERTEX_SHADER,
    GRID_VERTEX_SHADER_WGSL, VERTEX_SHADER_3D, VERTEX_SHADER_3D_WGSL,
};
use super::types::{
    Camera3D, FogConfig, GridConfig, Light, Object3D, PrimitiveCreateInfo, PrimitiveType,
    SkyboxConfig,
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
    pub(super) grid_shader_handle: ShaderHandle,
    pub(super) grid_buffer: BufferHandle,
    pub(super) grid_vertex_count: i32,
    pub(super) axis_buffer: BufferHandle,
    pub(super) axis_vertex_count: i32,
    pub(super) debug_draw_buffer: BufferHandle,
    pub(super) debug_draw_buffer_capacity_bytes: usize,
    pub(super) debug_draw_vertex_count: i32,
    pub(super) objects: HashMap<u32, Object3D>,
    pub(super) lights: HashMap<u32, Light>,
    pub(super) next_object_id: u32,
    pub(super) next_light_id: u32,
    pub(super) camera: Camera3D,
    pub(super) window_width: u32,
    pub(super) window_height: u32,
    pub(super) viewport: (i32, i32, u32, u32),
    pub(super) grid_config: GridConfig,
    pub(super) skybox_config: SkyboxConfig,
    pub(super) fog_config: FogConfig,
    pub(super) uniforms: MainUniforms,
    pub(super) grid_uniforms: GridUniforms,
    pub(super) object_layout: VertexLayout,
    pub(super) grid_layout: VertexLayout,
}

impl Renderer3D {
    /// Create a new 3D renderer with the given backend.
    pub fn new(
        mut backend: Box<dyn RenderBackend>,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        let use_wgpu_shaders = backend.info().name == "wgpu";
        let (vertex_shader, fragment_shader) = if use_wgpu_shaders {
            (VERTEX_SHADER_3D_WGSL, FRAGMENT_SHADER_3D_WGSL)
        } else {
            (VERTEX_SHADER_3D, FRAGMENT_SHADER_3D)
        };
        let (grid_vertex_shader, grid_fragment_shader) = if use_wgpu_shaders {
            (GRID_VERTEX_SHADER_WGSL, GRID_FRAGMENT_SHADER_WGSL)
        } else {
            (GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER)
        };

        let shader_handle = backend
            .create_shader(vertex_shader, fragment_shader)
            .map_err(|e| format!("Main 3D shader: {e}"))?;
        let uniforms = resolve_main_uniforms(backend.as_ref(), shader_handle);

        let grid_shader_handle = backend
            .create_shader(grid_vertex_shader, grid_fragment_shader)
            .map_err(|e| format!("Grid shader: {e}"))?;
        let grid_uniforms = resolve_grid_uniforms(backend.as_ref(), grid_shader_handle);

        let grid_layout = grid_vertex_layout();
        let (grid_buffer, grid_vertex_count) = create_grid_mesh(backend.as_mut(), 20.0, 20)?;
        let (axis_buffer, axis_vertex_count) = create_axis_mesh(backend.as_mut(), 5.0)?;

        Ok(Self {
            backend,
            shader_handle,
            grid_shader_handle,
            grid_buffer,
            grid_vertex_count,
            axis_buffer,
            axis_vertex_count,
            debug_draw_buffer: BufferHandle::INVALID,
            debug_draw_buffer_capacity_bytes: 0,
            debug_draw_vertex_count: 0,
            objects: HashMap::new(),
            lights: HashMap::new(),
            next_object_id: 1,
            next_light_id: 1,
            camera: Camera3D::default(),
            window_width,
            window_height,
            viewport: (0, 0, window_width.max(1), window_height.max(1)),
            grid_config: GridConfig::default(),
            skybox_config: SkyboxConfig::default(),
            fog_config: FogConfig::default(),
            uniforms,
            grid_uniforms,
            object_layout: object_vertex_layout(),
            grid_layout,
        })
    }

    // ========================================================================
    // Primitive creation
    // ========================================================================

    /// Create a primitive object and return its ID.
    pub fn create_primitive(&mut self, info: PrimitiveCreateInfo) -> u32 {
        let vertices = match info.primitive_type {
            PrimitiveType::Cube => generate_cube_vertices(info.width, info.height, info.depth),
            PrimitiveType::Plane => generate_plane_vertices(info.width, info.depth),
            PrimitiveType::Sphere => generate_sphere_vertices(info.width / 2.0, info.segments),
            PrimitiveType::Cylinder => {
                generate_cylinder_vertices(info.width / 2.0, info.height, info.segments)
            }
        };

        let buffer = match upload_buffer(self.backend.as_mut(), &vertices) {
            Ok(h) => h,
            Err(e) => {
                log::error!("Failed to create primitive buffer: {e}");
                return 0;
            }
        };

        let id = self.next_object_id;
        self.next_object_id += 1;

        self.objects.insert(
            id,
            Object3D {
                buffer,
                vertex_count: (vertices.len() / 8) as i32,
                position: Vector3::new(0.0, 0.0, 0.0),
                rotation: Vector3::new(0.0, 0.0, 0.0),
                scale: Vector3::new(1.0, 1.0, 1.0),
                texture_id: info.texture_id,
            },
        );

        id
    }

    // ========================================================================
    // Object manipulation
    // ========================================================================

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

    // ========================================================================
    // Lighting
    // ========================================================================

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

    // ========================================================================
    // Camera
    // ========================================================================

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

    // ========================================================================
    // Grid / Skybox / Fog configuration
    // ========================================================================

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
        self.backend.destroy_buffer(self.grid_buffer);
        self.backend.destroy_buffer(self.axis_buffer);
        if self.backend.is_buffer_valid(self.debug_draw_buffer) {
            self.backend.destroy_buffer(self.debug_draw_buffer);
        }
        self.backend.destroy_shader(self.shader_handle);
        self.backend.destroy_shader(self.grid_shader_handle);
    }
}
