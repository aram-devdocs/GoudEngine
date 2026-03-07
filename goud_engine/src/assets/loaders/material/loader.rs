//! [`MaterialLoader`] -- parses material descriptor JSON into [`MaterialAsset`].

use std::collections::HashMap;

use serde::Deserialize;

use crate::assets::{AssetLoadError, AssetLoader, LoadContext};

use super::{asset::MaterialAsset, uniform::UniformValue};

/// Intermediate deserialization target for material JSON files.
#[derive(Deserialize)]
struct MaterialDescriptor {
    name: String,
    shader_path: String,
    #[serde(default)]
    uniforms: HashMap<String, UniformValue>,
    #[serde(default)]
    texture_slots: HashMap<String, String>,
}

/// Asset loader for material descriptor files (`.mat.json`).
///
/// Parses a JSON descriptor into a [`MaterialAsset`] and declares
/// shader and texture dependencies via [`LoadContext`].
///
/// # JSON Format
///
/// ```json
/// {
///   "name": "brick_wall",
///   "shader_path": "shaders/pbr.glsl",
///   "uniforms": {
///     "roughness": { "type": "Float", "value": 0.7 },
///     "base_color": { "type": "Vec4", "value": [0.8, 0.3, 0.1, 1.0] }
///   },
///   "texture_slots": {
///     "albedo": "textures/brick_albedo.png",
///     "normal": "textures/brick_normal.png"
///   }
/// }
/// ```
///
/// # Example
///
/// ```no_run
/// use goud_engine::assets::{AssetServer, loaders::material::{MaterialLoader, MaterialAsset}};
///
/// let mut server = AssetServer::new();
/// server.register_loader(MaterialLoader::default());
///
/// let handle = server.load::<MaterialAsset>("materials/brick.mat.json");
/// ```
#[derive(Debug, Clone, Default)]
pub struct MaterialLoader;

impl MaterialLoader {
    /// Creates a new material loader.
    pub fn new() -> Self {
        Self
    }
}

impl AssetLoader for MaterialLoader {
    type Asset = MaterialAsset;
    type Settings = ();

    fn extensions(&self) -> &[&str] {
        &["mat.json"]
    }

    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        _settings: &'a Self::Settings,
        context: &'a mut LoadContext,
    ) -> Result<Self::Asset, AssetLoadError> {
        let descriptor: MaterialDescriptor = serde_json::from_slice(bytes).map_err(|e| {
            AssetLoadError::decode_failed(format!("Material JSON parse error: {e}"))
        })?;

        // Declare shader dependency.
        context.add_dependency(&descriptor.shader_path);

        // Declare texture dependencies.
        for texture_path in descriptor.texture_slots.values() {
            context.add_dependency(texture_path);
        }

        Ok(MaterialAsset::new(
            descriptor.name,
            descriptor.shader_path,
            descriptor.uniforms,
            descriptor.texture_slots,
        ))
    }
}
