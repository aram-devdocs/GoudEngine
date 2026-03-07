//! [`UniformValue`] -- shader uniform value types.

use serde::{Deserialize, Serialize};

/// A value that can be bound to a shader uniform.
///
/// Represents the common types used for GPU shader parameters.
/// Each variant maps to a corresponding GLSL type.
///
/// # Example
///
/// ```
/// use goud_engine::assets::loaders::material::UniformValue;
///
/// let color = UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]);
/// let intensity = UniformValue::Float(0.8);
/// let count = UniformValue::Int(3);
///
/// assert_eq!(color, UniformValue::Vec4([1.0, 0.0, 0.0, 1.0]));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum UniformValue {
    /// A single floating-point value (GLSL `float`).
    Float(f32),
    /// A 2-component float vector (GLSL `vec2`).
    Vec2([f32; 2]),
    /// A 3-component float vector (GLSL `vec3`).
    Vec3([f32; 3]),
    /// A 4-component float vector (GLSL `vec4`).
    Vec4([f32; 4]),
    /// A single integer value (GLSL `int`).
    Int(i32),
    /// A 4x4 float matrix (GLSL `mat4`).
    Mat4([[f32; 4]; 4]),
}

impl UniformValue {
    /// Returns a human-readable name for the uniform type.
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Float(_) => "Float",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Vec4(_) => "Vec4",
            Self::Int(_) => "Int",
            Self::Mat4(_) => "Mat4",
        }
    }
}
