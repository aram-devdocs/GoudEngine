//! Rendering utility functions.

use super::super::core::Renderer3D;
use cgmath::{Deg, Matrix4};

impl Renderer3D {
    /// Build a TRS (translate-rotate-scale) model matrix from object components.
    pub(in crate::libs::graphics::renderer3d) fn create_model_matrix(
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Vector3<f32>,
        scale: cgmath::Vector3<f32>,
    ) -> Matrix4<f32> {
        let translation = Matrix4::from_translation(position);
        let rot_x = Matrix4::from_angle_x(Deg(rotation.x));
        let rot_y = Matrix4::from_angle_y(Deg(rotation.y));
        let rot_z = Matrix4::from_angle_z(Deg(rotation.z));
        let rotation_matrix = rot_z * rot_y * rot_x;
        let scale_matrix = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);
        translation * rotation_matrix * scale_matrix
    }
}

/// Convert a cgmath [`Matrix4`] to a column-major `[f32; 16]` array.
///
/// cgmath matrices are already column-major, which matches the backend expectation.
pub(in crate::libs::graphics::renderer3d) fn mat4_to_array(m: &Matrix4<f32>) -> [f32; 16] {
    let cols: &[[f32; 4]; 4] = m.as_ref();
    [
        cols[0][0], cols[0][1], cols[0][2], cols[0][3], cols[1][0], cols[1][1], cols[1][2],
        cols[1][3], cols[2][0], cols[2][1], cols[2][2], cols[2][3], cols[3][0], cols[3][1],
        cols[3][2], cols[3][3],
    ]
}
