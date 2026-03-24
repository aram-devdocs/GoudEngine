use crate::assets::loaders::{
    MaterialAsset, MaterialLoader, ShaderAsset, ShaderLoader, SpriteSheetAsset, SpriteSheetLoader,
    TextureAsset, TextureLoader, UniformValue,
};
use crate::assets::{AssetHandle, AssetServer};
use crate::libs::graphics::backend::ShaderLanguage;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const DEFAULT_SPRITE_SHADER_ASSET_PATH: &str = "engine/shaders/sprite_batch.shader";
const DEFAULT_SPRITE_SHADER_ASSET_BYTES: &[u8] = br#"#pragma stage vertex
#version 330 core

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_texcoord;
layout(location = 2) in vec4 a_color;

uniform vec2 u_viewport;

out vec2 v_texcoord;
out vec4 v_color;

void main() {
    vec2 safe_viewport = max(u_viewport, vec2(1.0, 1.0));
    vec2 ndc;
    ndc.x = (a_position.x / safe_viewport.x) * 2.0 - 1.0;
    ndc.y = 1.0 - (a_position.y / safe_viewport.y) * 2.0;
    gl_Position = vec4(ndc, 0.0, 1.0);
    v_texcoord = a_texcoord;
    v_color = a_color;
}

#pragma stage fragment
#version 330 core

in vec2 v_texcoord;
in vec4 v_color;

uniform sampler2D u_texture;

out vec4 FragColor;

void main() {
    FragColor = texture(u_texture, v_texcoord) * v_color;
}
"#;

const DEFAULT_SPRITE_SHADER_ASSET_PATH_WGSL: &str = "engine/shaders/sprite_batch_wgsl.shader";
const DEFAULT_SPRITE_SHADER_ASSET_BYTES_WGSL: &[u8] = br#"#pragma stage vertex
struct Uniforms {
    u_viewport: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) a_position: vec2<f32>,
    @location(1) a_texcoord: vec2<f32>,
    @location(2) a_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) v_texcoord: vec2<f32>,
    @location(1) v_color: vec4<f32>,
}

@vertex
fn main(in: VertexInput) -> VertexOutput {
    let safe_viewport = max(uniforms.u_viewport, vec2<f32>(1.0, 1.0));
    let ndc_x = (in.a_position.x / safe_viewport.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (in.a_position.y / safe_viewport.y) * 2.0;

    var out: VertexOutput;
    out.position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.v_texcoord = in.a_texcoord;
    out.v_color = in.a_color;
    return out;
}

#pragma stage fragment
struct Uniforms {
    u_viewport: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var u_texture: texture_2d<f32>;
@group(1) @binding(1) var u_sampler: sampler;

@fragment
fn main(@location(0) v_texcoord: vec2<f32>, @location(1) v_color: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, v_texcoord) * v_color;
}
"#;

pub(crate) fn ensure_sprite_asset_loaders(asset_server: &mut AssetServer) {
    if !asset_server.has_loader_for_type::<TextureAsset>() {
        asset_server.register_loader(TextureLoader);
    }
    if !asset_server.has_loader_for_type::<ShaderAsset>() {
        asset_server.register_loader(ShaderLoader::default());
    }
    if !asset_server.has_loader_for_type::<MaterialAsset>() {
        asset_server.register_loader(MaterialLoader);
    }
    if !asset_server.has_loader_for_type::<SpriteSheetAsset>() {
        asset_server.register_loader(SpriteSheetLoader);
    }
}

pub(crate) fn ensure_default_sprite_shader_loaded(
    asset_server: &mut AssetServer,
    language: ShaderLanguage,
) -> AssetHandle<ShaderAsset> {
    ensure_sprite_asset_loaders(asset_server);
    let (path, bytes) = match language {
        ShaderLanguage::Wgsl => (
            DEFAULT_SPRITE_SHADER_ASSET_PATH_WGSL,
            DEFAULT_SPRITE_SHADER_ASSET_BYTES_WGSL,
        ),
        ShaderLanguage::Glsl => (
            DEFAULT_SPRITE_SHADER_ASSET_PATH,
            DEFAULT_SPRITE_SHADER_ASSET_BYTES,
        ),
    };
    asset_server.load_from_bytes::<ShaderAsset>(path, bytes)
}

pub(super) fn shader_signature(
    shader_asset: &ShaderAsset,
    material_asset: Option<&MaterialAsset>,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    let mut stages: Vec<_> = shader_asset.stages().collect();
    stages.sort_by_key(|(stage, _)| **stage as u8);
    for (stage, source) in stages {
        (*stage as u8).hash(&mut hasher);
        source.version.hash(&mut hasher);
        source.source.hash(&mut hasher);
    }

    if let Some(material) = material_asset {
        material.shader_path.hash(&mut hasher);

        let mut uniform_names: Vec<_> = material.uniforms().iter().collect();
        uniform_names.sort_by(|a, b| a.0.cmp(b.0));
        for (name, value) in uniform_names {
            name.hash(&mut hasher);
            hash_uniform_value(value, &mut hasher);
        }

        let mut texture_slots: Vec<_> = material.texture_slots().iter().collect();
        texture_slots.sort_by(|a, b| a.0.cmp(b.0));
        for (slot, path) in texture_slots {
            slot.hash(&mut hasher);
            path.hash(&mut hasher);
        }
    }

    hasher.finish()
}

pub(super) fn texture_signature(texture: &TextureAsset) -> u64 {
    let mut hasher = DefaultHasher::new();
    texture.width.hash(&mut hasher);
    texture.height.hash(&mut hasher);
    texture.data.hash(&mut hasher);
    hasher.finish()
}

fn hash_uniform_value(value: &UniformValue, hasher: &mut DefaultHasher) {
    match value {
        UniformValue::Float(value) => value.to_bits().hash(hasher),
        UniformValue::Vec2(value) => value
            .iter()
            .for_each(|component| component.to_bits().hash(hasher)),
        UniformValue::Vec3(value) => value
            .iter()
            .for_each(|component| component.to_bits().hash(hasher)),
        UniformValue::Vec4(value) => value
            .iter()
            .for_each(|component| component.to_bits().hash(hasher)),
        UniformValue::Int(value) => value.hash(hasher),
        UniformValue::Mat4(value) => value
            .iter()
            .flat_map(|row| row.iter())
            .for_each(|component| component.to_bits().hash(hasher)),
    }
}
