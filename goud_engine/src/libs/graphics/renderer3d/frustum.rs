//! View frustum extraction and bounding sphere culling.
//!
//! Pure math — no GPU calls. Uses the Gribb/Hartmann method to extract six
//! frustum planes from a view-projection matrix and tests bounding spheres
//! against them.

use cgmath::{Matrix4, Vector3};

/// A plane in 3D space represented as `normal · point + d = 0`.
#[derive(Debug, Clone, Copy)]
struct Plane {
    normal: Vector3<f32>,
    d: f32,
}

impl Plane {
    /// Signed distance from a point to the plane.
    /// Positive = same side as normal, negative = opposite side.
    fn distance_to_point(&self, point: Vector3<f32>) -> f32 {
        self.normal.x * point.x + self.normal.y * point.y + self.normal.z * point.z + self.d
    }
}

/// Six planes of a view-projection frustum.
#[derive(Debug, Clone, Copy)]
pub(super) struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    /// Extract 6 frustum planes from a combined view-projection matrix.
    ///
    /// Uses the Gribb/Hartmann method: each plane is derived from the rows
    /// of the VP matrix.  Planes point inward (positive side is inside).
    pub(super) fn from_view_projection(vp: &Matrix4<f32>) -> Self {
        let m: &[[f32; 4]; 4] = vp.as_ref();

        // Row-major extraction (cgmath stores column-major, so m[col][row]).
        let row = |r: usize| -> [f32; 4] { [m[0][r], m[1][r], m[2][r], m[3][r]] };

        let r0 = row(0);
        let r1 = row(1);
        let r2 = row(2);
        let r3 = row(3);

        let mut planes = [
            // Left:   row3 + row0
            make_plane(r3[0] + r0[0], r3[1] + r0[1], r3[2] + r0[2], r3[3] + r0[3]),
            // Right:  row3 - row0
            make_plane(r3[0] - r0[0], r3[1] - r0[1], r3[2] - r0[2], r3[3] - r0[3]),
            // Bottom: row3 + row1
            make_plane(r3[0] + r1[0], r3[1] + r1[1], r3[2] + r1[2], r3[3] + r1[3]),
            // Top:    row3 - row1
            make_plane(r3[0] - r1[0], r3[1] - r1[1], r3[2] - r1[2], r3[3] - r1[3]),
            // Near:   row3 + row2
            make_plane(r3[0] + r2[0], r3[1] + r2[1], r3[2] + r2[2], r3[3] + r2[3]),
            // Far:    row3 - row2
            make_plane(r3[0] - r2[0], r3[1] - r2[1], r3[2] - r2[2], r3[3] - r2[3]),
        ];

        // Normalize each plane.
        for p in &mut planes {
            let len = (p.normal.x * p.normal.x + p.normal.y * p.normal.y + p.normal.z * p.normal.z)
                .sqrt();
            if len > f32::EPSILON {
                let inv = 1.0 / len;
                p.normal.x *= inv;
                p.normal.y *= inv;
                p.normal.z *= inv;
                p.d *= inv;
            }
        }

        Self { planes }
    }

    /// Returns `true` if the sphere is at least partially inside the frustum.
    pub(super) fn intersects_sphere(&self, center: Vector3<f32>, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false; // Entirely outside this plane.
            }
        }
        true
    }
}

fn make_plane(a: f32, b: f32, c: f32, d: f32) -> Plane {
    Plane {
        normal: Vector3::new(a, b, c),
        d,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cgmath::{perspective, Deg, Matrix4, Point3};

    #[test]
    fn test_sphere_inside_frustum() {
        let proj = perspective(Deg(45.0), 1.0, 0.1, 100.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));

        // Object at origin should be visible.
        assert!(frustum.intersects_sphere(Vector3::new(0.0, 0.0, 0.0), 1.0));
    }

    #[test]
    fn test_sphere_outside_frustum() {
        let proj = perspective(Deg(45.0), 1.0, 0.1, 100.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));

        // Object far to the left should be culled.
        assert!(!frustum.intersects_sphere(Vector3::new(100.0, 0.0, 0.0), 1.0));
    }

    #[test]
    fn test_sphere_behind_camera() {
        let proj = perspective(Deg(45.0), 1.0, 0.1, 100.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));

        // Object behind the camera should be culled.
        assert!(!frustum.intersects_sphere(Vector3::new(0.0, 0.0, 10.0), 0.5));
    }

    #[test]
    fn test_sphere_beyond_far_plane() {
        let proj = perspective(Deg(45.0), 1.0, 0.1, 100.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));

        // Object beyond far plane should be culled.
        assert!(!frustum.intersects_sphere(Vector3::new(0.0, 0.0, -200.0), 1.0));
    }
}
