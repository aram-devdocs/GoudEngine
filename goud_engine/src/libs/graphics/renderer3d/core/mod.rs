//! Core [`Renderer3D`] struct, constructor, and object/light/camera manipulation.

mod drop;

use std::collections::HashMap;

use super::config::Render3DConfig;
use super::scene::Scene3D;
use crate::libs::graphics::backend::BufferHandle;
use crate::libs::graphics::backend::{RenderBackend, ShaderLanguage, VertexLayout};

use super::animation::AnimationPlayer;
use super::mesh::generate_plane_vertices;
use super::mesh::{
    create_axis_mesh, create_grid_mesh, create_postprocess_quad, depth_only_vertex_layout,
    grid_vertex_layout, instance_vertex_layout, instanced_skinned_instance_layout,
    object_vertex_layout, postprocess_vertex_layout, skinned_vertex_layout, upload_buffer,
};
use super::model::{Model3D, ModelInstance3D};
use super::shaders::{
    resolve_grid_uniforms, resolve_instanced_skinned_uniforms, resolve_main_uniforms,
    resolve_skinned_uniforms, DepthOnlyUniforms, GridUniforms, InstancedSkinnedUniforms,
    MainUniforms, SkinnedUniforms, DEPTH_ONLY_FRAGMENT_SHADER, DEPTH_ONLY_FRAGMENT_SHADER_WGSL,
    DEPTH_ONLY_VERTEX_SHADER, DEPTH_ONLY_VERTEX_SHADER_WGSL, FRAGMENT_SHADER_3D,
    FRAGMENT_SHADER_3D_WGSL, GRID_FRAGMENT_SHADER, GRID_FRAGMENT_SHADER_WGSL, GRID_VERTEX_SHADER,
    GRID_VERTEX_SHADER_WGSL, INSTANCED_FRAGMENT_SHADER_3D, INSTANCED_FRAGMENT_SHADER_3D_WGSL,
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

/// Core 3D renderer. All GPU operations go through the backend trait; no direct
/// graphics API calls are made outside the backend.
pub struct Renderer3D {
    pub(in crate::libs::graphics::renderer3d) backend: Box<dyn RenderBackend>,
    pub(in crate::libs::graphics::renderer3d) shader_handle: ShaderHandle,
    pub(in crate::libs::graphics::renderer3d) instanced_shader_handle: ShaderHandle,
    pub(in crate::libs::graphics::renderer3d) grid_shader_handle: ShaderHandle,
    pub(in crate::libs::graphics::renderer3d) postprocess_shader_handle: ShaderHandle,
    pub(in crate::libs::graphics::renderer3d) grid_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) grid_vertex_count: i32,
    pub(in crate::libs::graphics::renderer3d) axis_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) axis_vertex_count: i32,
    pub(in crate::libs::graphics::renderer3d) debug_draw_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) debug_draw_buffer_capacity_bytes: usize,
    pub(in crate::libs::graphics::renderer3d) debug_draw_vertex_count: i32,
    pub(in crate::libs::graphics::renderer3d) objects: HashMap<u32, Object3D>,
    pub(in crate::libs::graphics::renderer3d) instanced_meshes: HashMap<u32, InstancedMesh>,
    pub(in crate::libs::graphics::renderer3d) particle_emitters: HashMap<u32, ParticleEmitter>,
    pub(in crate::libs::graphics::renderer3d) lights: HashMap<u32, Light>,
    pub(in crate::libs::graphics::renderer3d) next_object_id: u32,
    pub(in crate::libs::graphics::renderer3d) next_instanced_mesh_id: u32,
    pub(in crate::libs::graphics::renderer3d) next_light_id: u32,
    pub(in crate::libs::graphics::renderer3d) next_particle_emitter_id: u32,
    pub(in crate::libs::graphics::renderer3d) camera: Camera3D,
    pub(in crate::libs::graphics::renderer3d) window_width: u32,
    pub(in crate::libs::graphics::renderer3d) window_height: u32,
    pub(in crate::libs::graphics::renderer3d) viewport: (i32, i32, u32, u32),
    pub(in crate::libs::graphics::renderer3d) grid_config: GridConfig,
    pub(in crate::libs::graphics::renderer3d) skybox_config: SkyboxConfig,
    pub(in crate::libs::graphics::renderer3d) fog_config: FogConfig,
    pub(in crate::libs::graphics::renderer3d) uniforms: MainUniforms,
    pub(in crate::libs::graphics::renderer3d) instanced_uniforms: MainUniforms,
    pub(in crate::libs::graphics::renderer3d) grid_uniforms: GridUniforms,
    pub(in crate::libs::graphics::renderer3d) object_layout: VertexLayout,
    pub(in crate::libs::graphics::renderer3d) instance_layout: VertexLayout,
    pub(in crate::libs::graphics::renderer3d) grid_layout: VertexLayout,
    pub(in crate::libs::graphics::renderer3d) particle_quad_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) particle_quad_vertex_count: u32,
    pub(in crate::libs::graphics::renderer3d) postprocess_quad_buffer: BufferHandle,
    pub(in crate::libs::graphics::renderer3d) postprocess_layout: VertexLayout,
    pub(in crate::libs::graphics::renderer3d) postprocess_texture:
        Option<crate::libs::graphics::backend::TextureHandle>,
    pub(in crate::libs::graphics::renderer3d) postprocess_texture_size: (u32, u32),
    pub(in crate::libs::graphics::renderer3d) shadow_texture:
        Option<crate::libs::graphics::backend::TextureHandle>,
    pub(in crate::libs::graphics::renderer3d) materials: HashMap<u32, Material3D>,
    pub(in crate::libs::graphics::renderer3d) object_materials: HashMap<u32, u32>,
    pub(in crate::libs::graphics::renderer3d) next_material_id: u32,
    pub(in crate::libs::graphics::renderer3d) skinned_meshes: HashMap<u32, SkinnedMesh3D>,
    pub(in crate::libs::graphics::renderer3d) next_skinned_mesh_id: u32,
    pub(in crate::libs::graphics::renderer3d) skinned_shader_handle: ShaderHandle,
    pub(in crate::libs::graphics::renderer3d) skinned_uniforms: SkinnedUniforms,
    pub(in crate::libs::graphics::renderer3d) skinned_layout: VertexLayout,
    pub(in crate::libs::graphics::renderer3d) models: HashMap<u32, Model3D>,
    pub(in crate::libs::graphics::renderer3d) model_instances: HashMap<u32, ModelInstance3D>,
    pub(in crate::libs::graphics::renderer3d) next_model_id: u32,
    pub(in crate::libs::graphics::renderer3d) animation_players: HashMap<u32, AnimationPlayer>,
    /// Shader and uniforms for instanced skinned rendering.
    pub(in crate::libs::graphics::renderer3d) instanced_skinned_shader_handle: ShaderHandle,
    pub(in crate::libs::graphics::renderer3d) instanced_skinned_uniforms: InstancedSkinnedUniforms,
    /// Per-instance vertex layout for instanced skinned rendering:
    /// model_0..model_3 (4 x vec4) + bone_offset (f32) + color (vec4) = 84 bytes.
    pub(in crate::libs::graphics::renderer3d) instanced_skinned_instance_layout: VertexLayout,
    /// Storage buffer handle for GPU skinning bone matrices (per-object path).
    pub(in crate::libs::graphics::renderer3d) bone_storage_buffer: Option<BufferHandle>,
    /// Tracks allocated size of bone_storage_buffer in bytes.
    pub(in crate::libs::graphics::renderer3d) bone_storage_buffer_size: usize,
    /// Separate storage buffer for the instanced skinned path.
    pub(in crate::libs::graphics::renderer3d) instanced_bone_storage_buffer: Option<BufferHandle>,
    /// Tracks allocated size of instanced_bone_storage_buffer in bytes.
    pub(in crate::libs::graphics::renderer3d) instanced_bone_storage_buffer_size: usize,
    pub(in crate::libs::graphics::renderer3d) postprocess_pipeline: PostProcessPipeline,
    pub(in crate::libs::graphics::renderer3d) stats: Renderer3DStats,
    pub(in crate::libs::graphics::renderer3d) anti_aliasing_mode: AntiAliasingMode,
    pub(in crate::libs::graphics::renderer3d) msaa_samples: u32,
    pub(in crate::libs::graphics::renderer3d) scenes: HashMap<u32, Scene3D>,
    pub(in crate::libs::graphics::renderer3d) next_scene_id: u32,
    pub(in crate::libs::graphics::renderer3d) current_scene: Option<u32>,
    /// Object IDs belonging to skinned models/instances -- maintained
    /// incrementally to avoid per-frame recomputation.
    pub(in crate::libs::graphics::renderer3d) skinned_object_ids: std::collections::HashSet<u32>,
    /// Game-developer-controlled configuration for the 3D renderer.
    pub(in crate::libs::graphics::renderer3d) config: Render3DConfig,
    /// Reusable scratch buffer for CPU skinning output (avoids per-submesh allocation).
    pub(in crate::libs::graphics::renderer3d) skin_scratch_buffer: Vec<f32>,
    /// Monotonically increasing frame counter, used for animation LOD (half-rate skipping).
    pub(in crate::libs::graphics::renderer3d) frame_counter: u64,
    /// Global animation clocks for phase-locked playback.
    /// Key: (source_model_id, clip_index). Value: elapsed time in seconds.
    pub(in crate::libs::graphics::renderer3d) phase_lock_clocks: HashMap<(u32, usize), f32>,
    /// Pool of per-group instance buffers for instanced skinned rendering.
    /// Each group gets its own buffer to avoid wgpu write-staging overwrites.
    pub(in crate::libs::graphics::renderer3d) instanced_skinned_instance_buffers:
        Vec<(BufferHandle, usize)>,
    /// Reusable G5 shared animation evaluation cache -- cleared each frame.
    pub(in crate::libs::graphics::renderer3d) bone_eval_cache:
        HashMap<(u32, usize, u32), Vec<[f32; 16]>>,
    /// Depth-only shader handle for the GPU shadow pre-pass.
    pub(in crate::libs::graphics::renderer3d) depth_only_shader_handle: ShaderHandle,
    /// Cached uniform locations for the depth-only shadow shader.
    pub(in crate::libs::graphics::renderer3d) depth_only_uniforms: DepthOnlyUniforms,
    /// Vertex layout for the depth-only shader (position only, stride 32 bytes).
    pub(in crate::libs::graphics::renderer3d) depth_only_layout: VertexLayout,
    /// Pre-allocated buffer of visible object IDs, reused across frames to avoid
    /// per-frame Vec allocation during the render snapshot phase.
    pub(in crate::libs::graphics::renderer3d) visible_object_ids: Vec<u32>,
    /// Whether the static batch VBO needs rebuilding (set when `set_object_static` changes).
    pub(in crate::libs::graphics::renderer3d) static_batch_dirty: bool,
    /// Pre-baked VBO containing all static objects' transformed vertices.
    pub(in crate::libs::graphics::renderer3d) static_batch_buffer: Option<BufferHandle>,
    /// Material/texture groups within the static batch buffer.
    pub(in crate::libs::graphics::renderer3d) static_batch_groups: Vec<StaticBatchGroup>,
    /// Total vertex count in the static batch buffer.
    pub(in crate::libs::graphics::renderer3d) static_batch_vertex_count: u32,
}

// StaticBatchGroup is defined in core_static_batch.rs
pub(in crate::libs::graphics::renderer3d) use super::core_static_batch::StaticBatchGroup;

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

        // Depth-only shader for the GPU shadow pre-pass.
        let (vs, fs) = shaders!(
            (DEPTH_ONLY_VERTEX_SHADER, DEPTH_ONLY_FRAGMENT_SHADER),
            (DEPTH_ONLY_VERTEX_SHADER_WGSL, DEPTH_ONLY_FRAGMENT_SHADER_WGSL)
        );
        let depth_only_shader_handle = backend
            .create_shader(vs, fs)
            .map_err(|e| format!("Depth-only shader: {e}"))?;
        let depth_only_uniforms =
            super::shaders::resolve_depth_only_uniforms(backend.as_ref(), depth_only_shader_handle);

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
            instanced_bone_storage_buffer: None,
            instanced_bone_storage_buffer_size: 0,
            postprocess_pipeline: PostProcessPipeline::new(),
            stats: Renderer3DStats::default(),
            anti_aliasing_mode: AntiAliasingMode::Off,
            msaa_samples: 1,
            scenes: HashMap::new(),
            next_scene_id: 1,
            current_scene: None,
            skinned_object_ids: std::collections::HashSet::new(),
            config: Render3DConfig::default(),
            skin_scratch_buffer: Vec::new(),
            frame_counter: 0,
            phase_lock_clocks: HashMap::new(),
            instanced_skinned_instance_buffers: Vec::new(),
            bone_eval_cache: HashMap::new(),
            depth_only_shader_handle,
            depth_only_uniforms,
            depth_only_layout: depth_only_vertex_layout(),
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
        self.config.shadows.bias = bias.max(0.0);
    }

    pub fn shadow_bias(&self) -> f32 {
        self.config.shadows.bias
    }

    pub fn set_shadows_enabled(&mut self, enabled: bool) {
        self.config.shadows.enabled = enabled;
    }

    pub fn shadows_enabled(&self) -> bool {
        self.config.shadows.enabled
    }

    pub fn render_config(&self) -> &Render3DConfig {
        &self.config
    }

    pub fn set_render_config(&mut self, config: Render3DConfig) {
        self.config = config;
    }
}
