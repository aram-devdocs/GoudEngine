//! FFI-safe mathematical types for the engine.
//!
//! This module provides `#[repr(C)]` mathematical types that are safe to pass across
//! FFI boundaries. These types wrap functionality from cgmath but guarantee a stable,
//! predictable memory layout for use with C#, Python, and other language bindings.
//!
//! # Design Decision
//!
//! We wrap cgmath rather than replacing it because:
//! 1. **Internal Operations**: cgmath provides battle-tested matrix/vector operations
//!    (look_at, quaternion math, etc.) that would be error-prone to reimplement
//! 2. **FFI Safety**: cgmath types like `Vector3<f32>` are newtypes over arrays and
//!    don't guarantee a specific memory layout suitable for FFI
//! 3. **Type Safety**: Our wrappers ensure compile-time FFI compatibility while
//!    maintaining ergonomic conversions for internal use
//!
//! # Usage
//!
//! ```rust
//! use goud_engine::core::math::{Vec3, Color};
//!
//! // Create FFI-safe types
//! let position = Vec3::new(1.0, 2.0, 3.0);
//! let color = Color::RED;
//!
//! // Convert to cgmath for internal math operations
//! let cgmath_vec: cgmath::Vector3<f32> = position.into();
//!
//! // Convert back from cgmath results
//! let result = Vec3::from(cgmath_vec);
//! ```

// Re-export cgmath types for internal use where FFI is not needed
pub use cgmath::{Matrix3, Matrix4, Point3, Quaternion};

mod color;
mod easing;
mod rect;
mod tween;
mod vec2;
mod vec3;
mod vec4;

#[cfg(test)]
mod tests;

pub use color::Color;
pub use easing::{
    ease_in, ease_in_back, ease_in_out, ease_out, ease_out_bounce, linear, BezierEasing, Easing,
    EasingFn,
};
pub use rect::Rect;
pub use tween::{tween, Tweenable};
pub use vec2::Vec2;
pub use vec3::Vec3;
pub use vec4::Vec4;
