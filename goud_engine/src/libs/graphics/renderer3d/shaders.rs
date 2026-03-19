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
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
    pub(super) lights: Vec<LightUniforms>,
}

#[derive(Clone)]
pub(super) struct GridUniforms {
    pub(super) view: i32,
    pub(super) projection: i32,
    pub(super) view_pos: i32,
    pub(super) fog_enabled: i32,
    pub(super) fog_color: i32,
    pub(super) fog_density: i32,
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
            light_type: loc(&format!("lights[{i}].type")),
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
        fog_enabled: loc("fogEnabled"),
        fog_color: loc("fogColor"),
        fog_density: loc("fogDensity"),
        lights,
    }
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
        alpha: loc("alpha"),
    }
}

/// Cached uniform locations for the PBR shader program.
#[derive(Clone)]
#[allow(dead_code)]
pub(super) struct PbrUniforms {
    /// Model matrix.
    pub(super) model: i32,
    /// View matrix.
    pub(super) view: i32,
    /// Projection matrix.
    pub(super) projection: i32,
    /// Camera position.
    pub(super) view_pos: i32,
    /// Number of active lights.
    pub(super) num_lights: i32,
    /// Albedo / base color.
    pub(super) albedo: i32,
    /// Metallic factor.
    pub(super) metallic: i32,
    /// Roughness factor.
    pub(super) roughness: i32,
    /// Ambient occlusion.
    pub(super) ao: i32,
    /// Albedo texture map.
    pub(super) albedo_map: i32,
    /// Normal map.
    pub(super) normal_map: i32,
    /// Metallic/roughness map.
    pub(super) metallic_roughness_map: i32,
    /// Per-light uniforms.
    pub(super) lights: Vec<LightUniforms>,
}

/// Cached uniform locations for the GPU skinning vertex shader.
#[derive(Clone)]
#[allow(dead_code)]
pub(super) struct SkinnedUniforms {
    /// Model matrix.
    pub(super) model: i32,
    /// View matrix.
    pub(super) view: i32,
    /// Projection matrix.
    pub(super) projection: i32,
    /// Array of bone matrices.
    pub(super) bone_matrices: Vec<i32>,
}

/// Combined skinned + PBR uniform locations.
#[derive(Clone)]
#[allow(dead_code)]
pub(super) struct SkinnedPbrUniforms {
    /// Skinning uniforms.
    pub(super) skinning: SkinnedUniforms,
    /// PBR material uniforms.
    pub(super) pbr: PbrUniforms,
}

/// Resolve PBR shader uniform locations.
#[allow(dead_code)]
pub(super) fn resolve_pbr_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
) -> PbrUniforms {
    let loc = |name: &str| -> i32 { uniform_location_or_inactive(backend, shader, name) };

    let mut lights = Vec::with_capacity(MAX_LIGHTS);
    for i in 0..MAX_LIGHTS {
        lights.push(LightUniforms {
            light_type: loc(&format!("lights[{i}].type")),
            position: loc(&format!("lights[{i}].position")),
            direction: loc(&format!("lights[{i}].direction")),
            color: loc(&format!("lights[{i}].color")),
            intensity: loc(&format!("lights[{i}].intensity")),
            range: loc(&format!("lights[{i}].range")),
            spot_angle: loc(&format!("lights[{i}].spotAngle")),
            enabled: loc(&format!("lights[{i}].enabled")),
        });
    }

    PbrUniforms {
        model: loc("model"),
        view: loc("view"),
        projection: loc("projection"),
        view_pos: loc("viewPos"),
        num_lights: loc("numLights"),
        albedo: loc("albedo"),
        metallic: loc("metallic"),
        roughness: loc("roughness"),
        ao: loc("ao"),
        albedo_map: loc("albedoMap"),
        normal_map: loc("normalMap"),
        metallic_roughness_map: loc("metallicRoughnessMap"),
        lights,
    }
}

/// Resolve GPU skinning vertex shader uniform locations.
#[allow(dead_code)]
pub(super) fn resolve_skinned_uniforms(
    backend: &dyn RenderBackend,
    shader: ShaderHandle,
    max_bones: usize,
) -> SkinnedUniforms {
    let loc = |name: &str| -> i32 { uniform_location_or_inactive(backend, shader, name) };

    let mut bone_matrices = Vec::with_capacity(max_bones);
    for i in 0..max_bones {
        bone_matrices.push(loc(&format!("boneMatrices[{i}]")));
    }

    SkinnedUniforms {
        model: loc("model"),
        view: loc("view"),
        projection: loc("projection"),
        bone_matrices,
    }
}
