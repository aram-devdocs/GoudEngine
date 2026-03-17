use crate::assets::loaders::{
    MaterialAsset, MaterialLoader, ShaderAsset, ShaderLoader, SpriteSheetAsset, SpriteSheetLoader,
    TextureAsset, TextureLoader, UniformValue,
};
use crate::assets::{AssetHandle, AssetServer};
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
) -> AssetHandle<ShaderAsset> {
    ensure_sprite_asset_loaders(asset_server);
    asset_server.load_from_bytes::<ShaderAsset>(
        DEFAULT_SPRITE_SHADER_ASSET_PATH,
        DEFAULT_SPRITE_SHADER_ASSET_BYTES,
    )
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
