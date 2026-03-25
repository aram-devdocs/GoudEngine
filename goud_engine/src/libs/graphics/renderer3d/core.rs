//! Core [`Renderer3D`] struct, constructor, and object/light/camera manipulation.

use std::collections::HashMap;

use super::config::Render3DConfig;
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
    instance_vertex_layout, instanced_skinned_instance_layout, object_vertex_layout,
    postprocess_vertex_layout, skinned_vertex_layout, upload_buffer,
};
use super::model::{Model3D, ModelInstance3D};
use super::shaders::{
    resolve_grid_uniforms, resolve_instanced_skinned_uniforms, resolve_main_uniforms,
    resolve_skinned_uniforms, GridUniforms, InstancedSkinnedUniforms, MainUniforms,
    SkinnedUniforms, FRAGMENT_SHADER_3D, FRAGMENT_SHADER_3D_WGSL, GRID_FRAGMENT_SHADER,
    GRID_FRAGMENT_SHADER_WGSL, GRID_VERTEX_SHADER, GRID_VERTEX_SHADER_WGSL,
    INSTANCED_FRAGMENT_SHADER_3D, INSTANCED_FRAGMENT_SHADER_3D_WGSL,
    INSTANCED_SKINNED_VERTEX_SHADER, INSTANCED_SKINNED_VERTEX_SHADER_WGSL,
    INSTANCED_VERTEX_SHADER_3D, INSTANCED_VERTEX_SHADER_3D_WGSL, POSTPROCESS_FRAGMENT_SHADER,
    POSTPROCESS_FRAGMENT_SHADER_WGSL, POSTPROCESS_VERTEX_SHADER, POSTPROCESS_VERTEX_SHADER_WGSL,
    SKINNED_VERTEX_SHADER, SKINNED_VERTEX_SHADER_WGSL, VERTEX_SHADER_3D, VERTEX_SHADER_3D_WGSL,
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

///
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
    /// Shader and uniforms for instanced skinned rendering.
    #[allow(dead_code)] // Infrastructure for instanced skinned render path.
    pub(super) instanced_skinned_shader_handle: ShaderHandle,
    #[allow(dead_code)]
    pub(super) instanced_skinned_uniforms: InstancedSkinnedUniforms,
    /// Per-instance vertex layout for instanced skinned rendering:
    /// model_0..model_3 (4 x vec4) + bone_offset (f32) + color (vec4) = 84 bytes.
    #[allow(dead_code)]
    pub(super) instanced_skinned_instance_layout: VertexLayout,
    /// Storage buffer handle for GPU skinning bone matrices.
    pub(super) bone_storage_buffer: Option<BufferHandle>,
    /// Tracks allocated size of bone_storage_buffer in bytes.
    #[allow(dead_code)]
    pub(super) bone_storage_buffer_size: usize,
    pub(super) postprocess_pipeline: PostProcessPipeline,
    pub(super) stats: Renderer3DStats,
    pub(super) anti_aliasing_mode: AntiAliasingMode,
    pub(super) msaa_samples: u32,
    pub(super) scenes: HashMap<u32, Scene3D>,
    pub(super) next_scene_id: u32,
    pub(super) current_scene: Option<u32>,
    /// When `false`, the CPU software shadow map pass is skipped entirely.
    pub(super) shadows_enabled: bool,
    /// Object IDs belonging to skinned models/instances — maintained
    /// incrementally to avoid per-frame recomputation.
    pub(super) skinned_object_ids: std::collections::HashSet<u32>,
    /// Game-developer-controlled configuration for the 3D renderer.
    pub(super) config: Render3DConfig,
    /// Reusable scratch buffer for CPU skinning output (avoids per-submesh allocation).
    pub(super) skin_scratch_buffer: Vec<f32>,
    /// Monotonically increasing frame counter, used for animation LOD (half-rate skipping).
    pub(super) frame_counter: u64,
    /// Persistent GPU buffer for instanced skinned per-instance data (reused across frames).
    pub(super) instanced_skinned_instance_buffer: Option<BufferHandle>,
    /// Current allocated size in bytes of `instanced_skinned_instance_buffer`.
    pub(super) instanced_skinned_instance_buffer_size: usize,
    /// Pre-allocated buffer of visible object IDs, reused across frames to avoid
    /// per-frame Vec allocation during the render snapshot phase.
    pub(super) visible_object_ids: Vec<u32>,
    /// Whether the static batch VBO needs rebuilding (set when `set_object_static` changes).
    pub(super) static_batch_dirty: bool,
    /// Pre-baked VBO containing all static objects' transformed vertices.
    pub(super) static_batch_buffer: Option<BufferHandle>,
    /// Material/texture groups within the static batch buffer.
    pub(super) static_batch_groups: Vec<StaticBatchGroup>,
    /// Total vertex count in the static batch buffer.
    pub(super) static_batch_vertex_count: u32,
}

/// A contiguous range of vertices in the static batch buffer sharing material and texture.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields read during static batch rendering pass.
pub(super) struct StaticBatchGroup {
    /// Material ID for this group (0 = default).
    pub material_id: u32,
    /// Texture ID for this group (0 = untextured).
    pub texture_id: u32,
    /// First vertex index in the batch buffer.
    pub start_vertex: u32,
    /// Number of vertices in this group.
    pub vertex_count: u32,
    /// RGBA color resolved from the material.
    pub color: [f32; 4],
}

#[allow(missing_docs)]
impl Renderer3D {
    #[rustfmt::skip]
    pub fn new(
        mut backend: Box<dyn RenderBackend>,
        window_width: u32,
        window_height: u32,
    ) -> Result<Self, String> {
        // Pick GLSL vs WGSL shader pair for the backend's shader language.
        let wgsl = matches!(backend.shader_language(), ShaderLanguage::Wgsl);
        macro_rules! shaders {
            ($gl:expr, $wg:expr) => {
                if wgsl {
                    $wg
                } else {
                    $gl
                }
            };
        }

        let (vs, fs) = shaders!(
            (VERTEX_SHADER_3D, FRAGMENT_SHADER_3D),
            (VERTEX_SHADER_3D_WGSL, FRAGMENT_SHADER_3D_WGSL)
        );
        let shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Main 3D shader: {e}"))?;
        let uniforms = resolve_main_uniforms(backend.as_ref(), shader_handle);

        let (vs, fs) = shaders!(
            (INSTANCED_VERTEX_SHADER_3D, INSTANCED_FRAGMENT_SHADER_3D),
            (
                INSTANCED_VERTEX_SHADER_3D_WGSL,
                INSTANCED_FRAGMENT_SHADER_3D_WGSL
            )
        );
        let instanced_shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Instanced 3D shader: {e}"))?;
        let instanced_uniforms = resolve_main_uniforms(backend.as_ref(), instanced_shader_handle);

        let (vs, fs) = shaders!(
            (GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER),
            (GRID_VERTEX_SHADER_WGSL, GRID_FRAGMENT_SHADER_WGSL)
        );
        let grid_shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Grid shader: {e}"))?;
        let grid_uniforms = resolve_grid_uniforms(backend.as_ref(), grid_shader_handle);

        let (vs, fs) = shaders!(
            (POSTPROCESS_VERTEX_SHADER, POSTPROCESS_FRAGMENT_SHADER),
            (
                POSTPROCESS_VERTEX_SHADER_WGSL,
                POSTPROCESS_FRAGMENT_SHADER_WGSL
            )
        );
        let postprocess_shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Postprocess shader: {e}"))?;

        // Skinned mesh shader uses the same fragment shader as the main 3D shader.
        let (vs, fs) = shaders!(
            (SKINNED_VERTEX_SHADER, FRAGMENT_SHADER_3D),
            (SKINNED_VERTEX_SHADER_WGSL, FRAGMENT_SHADER_3D_WGSL)
        );
        let skinned_shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Skinned 3D shader: {e}"))?;
        let skinned_uniforms = resolve_skinned_uniforms(backend.as_ref(), skinned_shader_handle);

        // Instanced skinned shader uses the instanced fragment shader.
        let (vs, fs) = shaders!(
            (INSTANCED_SKINNED_VERTEX_SHADER, INSTANCED_FRAGMENT_SHADER_3D),
            (
                INSTANCED_SKINNED_VERTEX_SHADER_WGSL,
                INSTANCED_FRAGMENT_SHADER_3D_WGSL
            )
        );
        let instanced_skinned_shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Instanced skinned shader: {e}"))?;
        let instanced_skinned_uniforms =
            resolve_instanced_skinned_uniforms(backend.as_ref(), instanced_skinned_shader_handle);

        let (grid_buffer, grid_vertex_count) = create_grid_mesh(backend.as_mut(), 20.0, 20)?;
        let (axis_buffer, axis_vertex_count) = create_axis_mesh(backend.as_mut(), 5.0)?;
        let particle_verts = generate_plane_vertices(1.0, 1.0);
        let particle_quad_vertex_count = (particle_verts.len() / 8) as u32;
        let particle_quad_buffer = upload_buffer(backend.as_mut(), &particle_verts)
            .map_err(|e| format!("Particle quad buffer: {e}"))?;
        let postprocess_quad_buffer = upload_buffer(backend.as_mut(), &create_postprocess_quad())
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
            instance_layout: instance_vertex_layout(),
            grid_layout: grid_vertex_layout(),
            particle_quad_buffer,
            particle_quad_vertex_count,
            postprocess_quad_buffer,
            postprocess_layout: postprocess_vertex_layout(),
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
            instanced_skinned_shader_handle,
            instanced_skinned_uniforms,
            instanced_skinned_instance_layout: instanced_skinned_instance_layout(),
            bone_storage_buffer: None,
            bone_storage_buffer_size: 0,
            postprocess_pipeline: PostProcessPipeline::new(),
            stats: Renderer3DStats::default(),
            anti_aliasing_mode: AntiAliasingMode::Off,
            msaa_samples: 1,
            scenes: HashMap::new(),
            next_scene_id: 1,
            current_scene: None,
            shadows_enabled: true,
            skinned_object_ids: std::collections::HashSet::new(),
            config: Render3DConfig::default(),
            skin_scratch_buffer: Vec::new(),
            frame_counter: 0,
            instanced_skinned_instance_buffer: None,
            instanced_skinned_instance_buffer_size: 0,
            visible_object_ids: Vec::with_capacity(1024),
            static_batch_dirty: false,
            static_batch_buffer: None,
            static_batch_groups: Vec::new(),
            static_batch_vertex_count: 0,
        })
    }
    pub fn set_object_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        self.mutate_object(id, |obj| obj.position = Vector3::new(x, y, z))
    }
    pub fn set_object_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        self.mutate_object(id, |obj| obj.rotation = Vector3::new(x, y, z))
    }
    pub fn set_object_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        self.mutate_object(id, |obj| obj.scale = Vector3::new(x, y, z))
    }

    /// Mark an object as static (transform never changes) or dynamic.
    ///
    /// Static objects are batched into a single VBO when
    /// [`BatchingConfig::static_batching_enabled`] is `true`, reducing draw calls.
    pub fn set_object_static(&mut self, id: u32, is_static: bool) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.is_static = is_static;
            self.static_batch_dirty = true;
            true
        } else {
            false
        }
    }

    fn mutate_object(&mut self, id: u32, f: impl FnOnce(&mut Object3D)) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            f(obj);
            true
        } else {
            false
        }
    }

    pub fn remove_object(&mut self, id: u32) -> bool {
        if let Some(obj) = self.objects.remove(&id) {
            if obj.is_static {
                self.static_batch_dirty = true;
            }
            self.backend.destroy_buffer(obj.buffer);
            true
        } else {
            false
        }
    }

    pub fn add_light(&mut self, light: Light) -> u32 {
        let id = self.next_light_id;
        self.next_light_id += 1;
        self.lights.insert(id, light);
        id
    }

    pub fn update_light(&mut self, id: u32, light: Light) -> bool {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(mut e) = self.lights.entry(id) {
            e.insert(light);
            true
        } else {
            false
        }
    }

    pub fn remove_light(&mut self, id: u32) -> bool {
        self.lights.remove(&id).is_some()
    }

    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) {
        self.camera.position = Vector3::new(x, y, z);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.viewport = (x, y, width.max(1), height.max(1));
    }

    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.camera.rotation = Vector3::new(pitch, yaw, roll);
    }

    pub fn stats(&self) -> Renderer3DStats {
        self.stats
    }

    pub fn anti_aliasing_mode(&self) -> AntiAliasingMode {
        self.anti_aliasing_mode
    }

    pub fn msaa_samples(&self) -> u32 {
        self.msaa_samples
    }

    pub fn set_anti_aliasing_mode(&mut self, mode: AntiAliasingMode) -> Result<(), String> {
        self.anti_aliasing_mode = mode;
        self.backend.set_multisampling_enabled(mode.uses_msaa());
        Ok(())
    }

    pub fn set_msaa_samples(&mut self, samples: u32) {
        self.msaa_samples = match samples {
            2 | 4 | 8 => samples,
            _ => 1,
        };
    }

    pub fn set_shadow_bias(&mut self, bias: f32) {
        self.shadow_bias = bias.max(0.0);
    }

    pub fn shadow_bias(&self) -> f32 {
        self.shadow_bias
    }

    pub fn set_shadows_enabled(&mut self, enabled: bool) {
        self.shadows_enabled = enabled;
    }

    pub fn shadows_enabled(&self) -> bool {
        self.shadows_enabled
    }

    pub fn render_config(&self) -> &Render3DConfig {
        &self.config
    }

    pub fn set_render_config(&mut self, config: Render3DConfig) {
        self.config = config;
    }

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
        self.grid_config = config.clone();
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.grid = config;
            }
        }
    }

    pub fn set_grid_enabled(&mut self, enabled: bool) {
        self.grid_config.enabled = enabled;
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.grid.enabled = enabled;
            }
        }
    }

    pub fn configure_skybox(&mut self, config: SkyboxConfig) {
        self.skybox_config = config.clone();
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.skybox = config;
            }
        }
    }

    pub fn configure_fog(&mut self, config: FogConfig) {
        self.fog_config = config.clone();
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.fog = config;
            }
        }
    }

    pub fn set_fog_enabled(&mut self, enabled: bool) {
        self.fog_config.enabled = enabled;
        if let Some(scene_id) = self.current_scene {
            if let Some(scene) = self.scenes.get_mut(&scene_id) {
                scene.fog.enabled = enabled;
            }
        }
    }

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

impl Renderer3D {
    /// Rebuild the static batch VBO from all objects with `is_static == true`.
    ///
    /// Vertices are transformed (baked) by each object's model matrix so the
    /// batch can be drawn with an identity model uniform.  Objects are grouped
    /// by `(material_id, texture_id)` to minimise state changes.
    pub(super) fn rebuild_static_batch(&mut self) {
        // Destroy old buffer.
        if let Some(buf) = self.static_batch_buffer.take() {
            self.backend.destroy_buffer(buf);
        }
        self.static_batch_groups.clear();
        self.static_batch_vertex_count = 0;
        self.static_batch_dirty = false;

        // Collect static object IDs with their sort key.
        let mut entries: Vec<(u32, u32, u32)> = Vec::new(); // (mat, tex, obj_id)
        for (&id, obj) in &self.objects {
            if !obj.is_static {
                continue;
            }
            let mat_id = self.object_materials.get(&id).copied().unwrap_or(0);
            entries.push((mat_id, obj.texture_id, id));
        }

        if entries.is_empty() {
            return;
        }

        entries.sort_by_key(|&(m, t, _)| (m, t));

        // Floats-per-vertex in the object layout: 3 pos + 3 normal + 2 uv = 8.
        const FPV: usize = 8;

        let max_verts = self.config.batching.max_static_batch_vertices;
        let mut all_verts: Vec<f32> = Vec::new();
        let mut current_mat: u32 = entries[0].0;
        let mut current_tex: u32 = entries[0].1;
        let mut group_start: u32 = 0;

        for &(mat_id, tex_id, obj_id) in &entries {
            let obj = match self.objects.get(&obj_id) {
                Some(o) => o,
                None => continue,
            };

            // Start a new group when material or texture changes.
            if mat_id != current_mat || tex_id != current_tex {
                let group_vertex_count = (all_verts.len() / FPV) as u32 - group_start;
                if group_vertex_count > 0 {
                    let color = self.resolve_material_color(current_mat, current_tex);
                    self.static_batch_groups.push(StaticBatchGroup {
                        material_id: current_mat,
                        texture_id: current_tex,
                        start_vertex: group_start,
                        vertex_count: group_vertex_count,
                        color,
                    });
                }
                group_start = (all_verts.len() / FPV) as u32;
                current_mat = mat_id;
                current_tex = tex_id;
            }

            // Enforce vertex budget.
            let obj_vert_count = obj.vertices.len() / FPV;
            if (all_verts.len() / FPV) + obj_vert_count > max_verts {
                log::warn!(
                    "Static batch vertex limit ({max_verts}) reached, skipping remaining objects"
                );
                break;
            }

            // Build model matrix and bake transform into vertices.
            let model = Self::create_model_matrix(obj.position, obj.rotation, obj.scale);
            let normal_matrix = Self::normal_matrix_from_model(&model);

            for v in 0..obj_vert_count {
                let base = v * FPV;
                // Transform position: model * vec4(pos, 1.0)
                let px = obj.vertices[base];
                let py = obj.vertices[base + 1];
                let pz = obj.vertices[base + 2];
                let cols: &[[f32; 4]; 4] = model.as_ref();
                let tx = cols[0][0] * px + cols[1][0] * py + cols[2][0] * pz + cols[3][0];
                let ty = cols[0][1] * px + cols[1][1] * py + cols[2][1] * pz + cols[3][1];
                let tz = cols[0][2] * px + cols[1][2] * py + cols[2][2] * pz + cols[3][2];
                all_verts.push(tx);
                all_verts.push(ty);
                all_verts.push(tz);

                // Transform normal: normal_matrix * normal (no translation).
                let nx = obj.vertices[base + 3];
                let ny = obj.vertices[base + 4];
                let nz = obj.vertices[base + 5];
                let tnx = normal_matrix[0] * nx + normal_matrix[3] * ny + normal_matrix[6] * nz;
                let tny = normal_matrix[1] * nx + normal_matrix[4] * ny + normal_matrix[7] * nz;
                let tnz = normal_matrix[2] * nx + normal_matrix[5] * ny + normal_matrix[8] * nz;
                // Normalize the transformed normal.
                let len = (tnx * tnx + tny * tny + tnz * tnz).sqrt().max(1e-10);
                all_verts.push(tnx / len);
                all_verts.push(tny / len);
                all_verts.push(tnz / len);

                // UV passthrough.
                all_verts.push(obj.vertices[base + 6]);
                all_verts.push(obj.vertices[base + 7]);
            }
        }

        // Close the final group.
        let group_vertex_count = (all_verts.len() / FPV) as u32 - group_start;
        if group_vertex_count > 0 {
            let color = self.resolve_material_color(current_mat, current_tex);
            self.static_batch_groups.push(StaticBatchGroup {
                material_id: current_mat,
                texture_id: current_tex,
                start_vertex: group_start,
                vertex_count: group_vertex_count,
                color,
            });
        }

        if all_verts.is_empty() {
            return;
        }

        self.static_batch_vertex_count = (all_verts.len() / FPV) as u32;

        match self.backend.create_buffer(
            BufferType::Vertex,
            BufferUsage::Static,
            bytemuck::cast_slice(&all_verts),
        ) {
            Ok(buf) => {
                self.static_batch_buffer = Some(buf);
            }
            Err(e) => {
                log::error!("Failed to create static batch buffer: {e}");
                self.static_batch_groups.clear();
                self.static_batch_vertex_count = 0;
            }
        }
    }

    /// Resolve RGBA color for a material, following the same logic as the draw loop.
    fn resolve_material_color(&self, mat_id: u32, texture_id: u32) -> [f32; 4] {
        if let Some(mat) = self.materials.get(&mat_id) {
            let c = &mat.color;
            [c.x, c.y, c.z, c.w]
        } else if texture_id > 0 {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            [0.8, 0.8, 0.8, 1.0]
        }
    }

    /// Compute the 3x3 normal matrix (transpose of inverse of upper-left 3x3) as a flat [f32; 9].
    fn normal_matrix_from_model(model: &cgmath::Matrix4<f32>) -> [f32; 9] {
        let cols: &[[f32; 4]; 4] = model.as_ref();
        // Upper-left 3x3.
        let a = cols[0][0]; let b = cols[1][0]; let c = cols[2][0];
        let d = cols[0][1]; let e = cols[1][1]; let f = cols[2][1];
        let g = cols[0][2]; let h = cols[1][2]; let i = cols[2][2];
        let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
        if det.abs() < 1e-10 {
            return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        }
        let inv_det = 1.0 / det;
        // Transpose of inverse = cofactor / det  (column-major output for row-major multiply).
        [
            (e * i - f * h) * inv_det,
            (f * g - d * i) * inv_det,
            (d * h - e * g) * inv_det,
            (c * h - b * i) * inv_det,
            (a * i - c * g) * inv_det,
            (b * g - a * h) * inv_det,
            (b * f - c * e) * inv_det,
            (c * d - a * f) * inv_det,
            (a * e - b * d) * inv_det,
        ]
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
        for e in self.particle_emitters.values() {
            self.backend.destroy_buffer(e.instance_buffer);
        }
        for m in self.skinned_meshes.values() {
            self.backend.destroy_buffer(m.buffer);
        }
        self.backend.destroy_buffer(self.grid_buffer);
        self.backend.destroy_buffer(self.axis_buffer);
        self.backend.destroy_buffer(self.particle_quad_buffer);
        self.backend.destroy_buffer(self.postprocess_quad_buffer);
        for tex in [self.postprocess_texture.take(), self.shadow_texture.take()]
            .into_iter()
            .flatten()
        {
            self.backend.destroy_texture(tex);
        }
        if self.backend.is_buffer_valid(self.debug_draw_buffer) {
            self.backend.destroy_buffer(self.debug_draw_buffer);
        }
        if let Some(buf) = self.bone_storage_buffer {
            self.backend.destroy_buffer(buf);
        }
        if let Some(buf) = self.instanced_skinned_instance_buffer {
            self.backend.destroy_buffer(buf);
        }
        if let Some(buf) = self.static_batch_buffer {
            self.backend.destroy_buffer(buf);
        }
        for &sh in &[
            self.shader_handle,
            self.instanced_shader_handle,
            self.grid_shader_handle,
            self.postprocess_shader_handle,
            self.skinned_shader_handle,
            self.instanced_skinned_shader_handle,
        ] {
            self.backend.destroy_shader(sh);
        }
    }
}
