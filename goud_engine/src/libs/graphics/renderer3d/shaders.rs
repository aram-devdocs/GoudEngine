//! Shader sources and cached uniform location structs for the 3D renderer.

use crate::libs::graphics::backend::{RenderBackend, ShaderHandle};

use super::types::MAX_LIGHTS;

// ============================================================================
// Shader sources
// ============================================================================

include!("shader_sources.in");

#[derive(Clone)]
pub(super) struct LightUniforms {
    pub(super) light_type: i32,
    pub(super) position: i32,
    pub(super) direction: i32,
    pub(super) color: i32,
    pub(super) intensity: i32,
    pub(super) range: i32,
    pub(super) spot_angle: i32,
    pub(super) enabled: i32,
}

#[derive(Clone)]
pub(super) struct MainUniforms {
    pub(super) model: i32,
    pub(super) view: i32,
    pub(super) projection: i32,
    pub(super) light_space_matrix: i32,
    pub(super) view_pos: i32,
    pub(super) num_lights: i32,
    pub(super) use_texture: i32,
    pub(super) object_color: i32,
    pub(super) texture1: i32,
    pub(super) shadow_map: i32,
    pub(super) shadows_enabled: i32,
    pub(super) shadow_bias: i32,
    pub(super) shadow_strength: i32,
    pub(super) shadow_texel_size: i32,
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
    pub(super) fog_mode: i32,
    pub(super) fog_start: i32,
    pub(super) fog_end: i32,
    pub(super) lights: Vec<LightUniforms>,
    pub(super) primary_light_dir: i32,
    pub(super) primary_light_color: i32,
    pub(super) primary_light_intensity: i32,
}

#[derive(Clone)]
pub(super) struct GridUniforms {
    pub(super) view: i32,
    pub(super) projection: i32,
    pub(super) view_pos: i32,
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
    pub(super) fog_mode: i32,
    pub(super) fog_start: i32,
    pub(super) fog_end: i32,
    pub(super) alpha: i32,
}

fn uniform_location_or_inactive(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
    name: &str,
) -> i32 {
    match backend.get_uniform_location(shader, name) {
        Some(location) => location,
        None => {
            log::debug!("Renderer3D expected uniform '{name}' was optimized out or missing");
            -1
        }
    }
}

pub(super) fn resolve_main_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> MainUniforms {
    let loc = |name: &str| -> i32 { uniform_location_or_inactive(backend, shader, name) };

    let mut lights = Vec::with_capacity(MAX_LIGHTS);
    for i in 0..MAX_LIGHTS {
        lights.push(LightUniforms {
            light_type: loc(&format!("lights[{i}].light_type")),
            position: loc(&format!("lights[{i}].position")),
            direction: loc(&format!("lights[{i}].direction")),
            color: loc(&format!("lights[{i}].color")),
            intensity: loc(&format!("lights[{i}].intensity")),
            range: loc(&format!("lights[{i}].range")),
            spot_angle: loc(&format!("lights[{i}].spotAngle")),
            enabled: loc(&format!("lights[{i}].enabled")),
        });
    }

    MainUniforms {
        model: loc("model"),
        view: loc("view"),
        projection: loc("projection"),
        light_space_matrix: loc("lightSpaceMatrix"),
        view_pos: loc("viewPos"),
        num_lights: loc("numLights"),
        use_texture: loc("useTexture"),
        object_color: loc("objectColor"),
        texture1: loc("texture1"),
        shadow_map: loc("shadowMap"),
        shadows_enabled: loc("shadowsEnabled"),
        shadow_bias: loc("shadowBias"),
        shadow_strength: loc("shadowStrength"),
        shadow_texel_size: loc("shadowTexelSize"),
        fog_enabled: loc("fogEnabled"),
        fog_color: loc("fogColor"),
        fog_density: loc("fogDensity"),
        fog_mode: loc("fogMode"),
        fog_start: loc("fogStart"),
        fog_end: loc("fogEnd"),
        lights,
        primary_light_dir: loc("primaryLightDir"),
        primary_light_color: loc("primaryLightColor"),
        primary_light_intensity: loc("primaryLightIntensity"),
    }
}

/// Cached uniform locations for the skinned mesh shader.
#[derive(Clone)]
pub(super) struct SkinnedUniforms {
    /// Standard scene uniforms (model, view, projection, lights, etc.).
    pub(super) main: MainUniforms,
    /// Uniform locations for `boneMatrices[i]` (one per bone slot).
    pub(super) bone_matrices: Vec<i32>,
    /// Uniform location for the bone offset into the storage buffer.
    pub(super) bone_offset: i32,
}

pub(super) fn resolve_skinned_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> SkinnedUniforms {
    let main = resolve_main_uniforms(backend, shader);
    let loc = |name: &str| -> i32 { uniform_location_or_inactive(backend, shader, name) };

    let mut bone_matrices = Vec::with_capacity(super::skinned_mesh::MAX_BONES);
    for i in 0..super::skinned_mesh::MAX_BONES {
        bone_matrices.push(loc(&format!("boneMatrices[{i}]")));
    }

    let bone_offset = loc("boneOffset");

    SkinnedUniforms {
        main,
        bone_matrices,
        bone_offset,
    }
}

/// Cached uniform locations for the instanced skinned mesh shader.
///
/// Uses the instanced shader's `MainUniforms` (no model matrix, since model
/// comes from per-instance attributes). Bone matrices are uploaded via storage
/// buffers, not per-bone uniforms.
#[derive(Clone)]
pub(super) struct InstancedSkinnedUniforms {
    /// Standard instanced scene uniforms (view, projection, lights, etc.).
    pub(super) main: MainUniforms,
}

pub(super) fn resolve_instanced_skinned_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> InstancedSkinnedUniforms {
    let main = resolve_main_uniforms(backend, shader);
    InstancedSkinnedUniforms { main }
}

/// Cached uniform locations for the depth-only shadow shader.
#[derive(Clone)]
pub(super) struct DepthOnlyUniforms {
    /// MVP matrix uniform location.
    pub(super) mvp: i32,
}

pub(super) fn resolve_depth_only_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> DepthOnlyUniforms {
    let loc = |name: &str| -> i32 { uniform_location_or_inactive(backend, shader, name) };
    DepthOnlyUniforms { mvp: loc("mvp") }
}

pub(super) fn resolve_grid_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> GridUniforms {
    let loc = |name: &str| -> i32 { uniform_location_or_inactive(backend, shader, name) };

    GridUniforms {
        view: loc("view"),
        projection: loc("projection"),
        view_pos: loc("viewPos"),
        fog_enabled: loc("fogEnabled"),
        fog_color: loc("fogColor"),
        fog_density: loc("fogDensity"),
        fog_mode: loc("fogMode"),
        fog_start: loc("fogStart"),
        fog_end: loc("fogEnd"),
        alpha: loc("alpha"),
    }
}
