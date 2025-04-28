use super::Camera;
use crate::types::Camera3D;
use cgmath::{Deg, Euler, InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Vector3};

impl Camera3D {
    pub fn new() -> Self {
        Camera3D {
            position: Vector3::new(0.0, 0.0, 3.0),
            target: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            zoom: 1.0,
            rotation: Vector3::new(0.0, 0.0, 0.0), // Default rotation (pitch, yaw, roll)
        }
    }

    /// Updates the target and up vectors based on rotation
    fn update_orientation(&mut self) {
        // Convert Euler angles from degrees to radians
        let pitch_rad = Rad::from(Deg(self.rotation.x));
        let yaw_rad = Rad::from(Deg(self.rotation.y));
        let roll_rad = Rad::from(Deg(self.rotation.z));

        // Calculate the forward direction (target is position + forward)
        let forward = Vector3::new(
            yaw_rad.0.sin() * pitch_rad.0.cos(),
            pitch_rad.0.sin(),
            yaw_rad.0.cos() * pitch_rad.0.cos(),
        );

        // Calculate right vector (perpendicular to forward and world up)
        let world_up = Vector3::new(0.0, 1.0, 0.0);
        let right = forward.cross(world_up).normalize();

        // Calculate local up vector (perpendicular to forward and right)
        let up = right.cross(forward).normalize();

        // Apply roll rotation to the up vector if needed
        if roll_rad.0 != 0.0 {
            // Create a quaternion for roll around the forward axis
            let q = Quaternion::from(Euler {
                x: Rad(0.0),
                y: Rad(0.0),
                z: roll_rad,
            });

            // Rotate the up vector by the quaternion
            self.up = q.rotate_vector(up);
        } else {
            self.up = up;
        }

        // Set target based on position and forward direction
        self.target = self.position + forward;
    }

    // pub fn set_target(&mut self, x: f32, y: f32, z: f32) {
    //     self.target = Vector3::new(x, y, z);
    // }

    // pub fn get_target(&self) -> Vector3<f32> {
    //     self.target
    // }

    // pub fn set_up(&mut self, x: f32, y: f32, z: f32) {
    //     self.up = Vector3::new(x, y, z);
    // }

    // pub fn get_up(&self) -> Vector3<f32> {
    //     self.up
    // }
}

impl Camera for Camera3D {
    fn get_view_matrix(&self) -> Matrix4<f32> {
        // For the view matrix, we use the look_at matrix based on position, target, and up
        Matrix4::look_at_rh(
            Point3::new(self.position.x, self.position.y, self.position.z),
            Point3::new(self.target.x, self.target.y, self.target.z),
            self.up,
        )
    }

    fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.position = Vector3::new(x, y, z);
        // Update target based on position and current rotation
        self.update_orientation();
    }

    fn set_position_xy(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
        // Update target based on position and current rotation
        self.update_orientation();
    }

    fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    fn set_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation = Vector3::new(pitch, yaw, roll);
        // Update target and up vectors based on rotation
        self.update_orientation();
    }

    fn get_rotation(&self) -> Vector3<f32> {
        self.rotation
    }

    fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom;
    }

    fn get_zoom(&self) -> f32 {
        self.zoom
    }
}
