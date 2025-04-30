use cgmath::{Matrix4, Vector3};

pub mod camera2d;
pub mod camera3d;

pub trait Camera {
    /// Get the view matrix for the camera
    fn get_view_matrix(&self) -> Matrix4<f32>;

    /// Set the camera position in 3D space
    fn set_position(&mut self, x: f32, y: f32, z: f32);

    /// Set only the x and y components of position (convenience method)
    fn set_position_xy(&mut self, x: f32, y: f32);

    /// Get the camera position
    fn get_position(&self) -> Vector3<f32>;

    /// Set the camera rotation (Euler angles in degrees)
    fn set_rotation(&mut self, pitch: f32, yaw: f32, roll: f32);

    /// Get the camera rotation (Euler angles in degrees)
    fn get_rotation(&self) -> Vector3<f32>;

    /// Set the camera zoom level
    fn set_zoom(&mut self, zoom: f32);

    /// Get the camera zoom level
    fn get_zoom(&self) -> f32;
}
