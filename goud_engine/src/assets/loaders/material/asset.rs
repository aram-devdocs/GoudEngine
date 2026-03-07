//! [`MaterialAsset`] -- parsed material definition.

use std::collections::HashMap;

use crate::assets::{Asset, AssetType};

use super::uniform::UniformValue;

/// A loaded material asset describing shader configuration.
///
/// Materials bind a shader program to a set of uniform values and
/// texture slots, forming the visual appearance of rendered geometry.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use goud_engine::assets::loaders::material::{MaterialAsset, UniformValue};
///
/// let mut uniforms = HashMap::new();
/// uniforms.insert("color".to_string(), UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]));
///
/// let asset = MaterialAsset::new(
///     "red_material".to_string(),
///     "shaders/basic.glsl".to_string(),
///     uniforms,
///     HashMap::new(),
/// );
///
/// assert_eq!(asset.name(), "red_material");
/// assert_eq!(asset.shader_path(), "shaders/basic.glsl");
/// ```
#[derive(Debug, Clone)]
pub struct MaterialAsset {
    /// Human-readable name for this material.
    pub name: String,
    /// Path to the shader program asset.
    pub shader_path: String,
    /// Named uniform values to bind when rendering.
    pub uniforms: HashMap<String, UniformValue>,
    /// Named texture slots mapping slot name to texture asset path.
    pub texture_slots: HashMap<String, String>,
}

impl MaterialAsset {
    /// Creates a new material asset.
    pub fn new(
        name: String,
        shader_path: String,
        uniforms: HashMap<String, UniformValue>,
        texture_slots: HashMap<String, String>,
    ) -> Self {
        Self {
            name,
            shader_path,
            uniforms,
            texture_slots,
        }
    }

    /// Returns the material name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the shader path.
    #[inline]
    pub fn shader_path(&self) -> &str {
        &self.shader_path
    }

    /// Returns a reference to the uniforms map.
    #[inline]
    pub fn uniforms(&self) -> &HashMap<String, UniformValue> {
        &self.uniforms
    }

    /// Returns a reference to the texture slots map.
    #[inline]
    pub fn texture_slots(&self) -> &HashMap<String, String> {
        &self.texture_slots
    }

    /// Returns a specific uniform value by name.
    pub fn get_uniform(&self, name: &str) -> Option<&UniformValue> {
        self.uniforms.get(name)
    }

    /// Returns a specific texture path by slot name.
    pub fn get_texture_slot(&self, slot: &str) -> Option<&str> {
        self.texture_slots.get(slot).map(String::as_str)
    }
}

impl Asset for MaterialAsset {
    fn asset_type_name() -> &'static str {
        "Material"
    }

    fn asset_type() -> AssetType {
        AssetType::Material
    }

    fn extensions() -> &'static [&'static str] {
        &["mat.json"]
    }
}
