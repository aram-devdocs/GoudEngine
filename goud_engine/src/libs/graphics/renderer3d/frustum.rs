//! View frustum extraction and bounding sphere culling.
//!
//! Pure math — no GPU calls. Uses the Gribb/Hartmann method to extract six
//! frustum planes from a view-projection matrix and tests bounding spheres
//! against them.

use cgmath::{Matrix4, SquareMatrix, Vector3, Vector4};

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

/// World-space AABB enclosing the view frustum produced by `view * projection`.
///
/// Computed by transforming the eight clip-space cube corners through the
/// inverse view-projection matrix and taking the per-axis min/max. Used by
/// the spatial index to pick the cell range to query — if `inv(VP)` is
/// non-invertible (degenerate camera), returns `None` so the caller can fall
/// back to a full scan.
///
/// NDC z is sampled at `-1` (near) and `+1` (far), matching cgmath's
/// `perspective()` output. Even when the active backend clips at `[0, 1]`,
/// this AABB is a conservative superset of the actual visible volume.
pub(in crate::libs::graphics::renderer3d) fn frustum_world_aabb(
    view: &Matrix4<f32>,
    projection: &Matrix4<f32>,
) -> Option<(Vector3<f32>, Vector3<f32>)> {
    let vp = projection * view;
    let inv_vp = vp.invert()?;
    const NDC_CORNERS: [[f32; 3]; 8] = [
        [-1.0, -1.0, -1.0],
        [1.0, -1.0, -1.0],
        [-1.0, 1.0, -1.0],
        [1.0, 1.0, -1.0],
        [-1.0, -1.0, 1.0],
        [1.0, -1.0, 1.0],
        [-1.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for c in &NDC_CORNERS {
        let v = inv_vp * Vector4::new(c[0], c[1], c[2], 1.0);
        if v.w.abs() < f32::EPSILON {
            return None;
        }
        let inv_w = 1.0 / v.w;
        let p = Vector3::new(v.x * inv_w, v.y * inv_w, v.z * inv_w);
        if !(p.x.is_finite() && p.y.is_finite() && p.z.is_finite()) {
            return None;
        }
        min.x = min.x.min(p.x);
        min.y = min.y.min(p.y);
        min.z = min.z.min(p.z);
        max.x = max.x.max(p.x);
        max.y = max.y.max(p.y);
        max.z = max.z.max(p.z);
    }
    Some((min, max))
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

    /// Regression test: objects placed away from origin must remain visible
    /// when the camera is near them. Previously, cached bounding spheres
    /// were initialized in local space (near origin) causing objects to be
    /// culled when the camera moved away from the origin.
    #[test]
    fn test_world_space_bounds_near_camera() {
        // Camera at (50, 5, 50) looking at (50, 0, 45) — far from origin,
        // looking roughly along -Z. Simulates walking toward a building
        // at (50, 0, 45).
        let proj = perspective(Deg(60.0), 16.0 / 9.0, 0.1, 200.0);
        let view = Matrix4::look_at_rh(
            Point3::new(50.0, 5.0, 50.0),
            Point3::new(50.0, 0.0, 45.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));

        // Object at (50, 0, 45) with local bounds center (0, 1, 0) and radius 2.0.
        let obj_position = Vector3::new(50.0, 0.0, 45.0);
        let bounds_center = Vector3::new(0.0, 1.0, 0.0);
        let bounds_radius = 2.0f32;

        // CORRECT: world-space center = position + local bounds center
        let world_center = obj_position + bounds_center;
        assert!(
            frustum.intersects_sphere(world_center, bounds_radius),
            "Object at world ({}, {}, {}) should be visible when camera looks at it",
            world_center.x,
            world_center.y,
            world_center.z,
        );

        // BUG REGRESSION: if we used local-space center (near origin) instead,
        // the object would be incorrectly placed at (0, 1, 0) — far from
        // the camera at (50, 5, 50). It should be culled.
        let stale_local_center = bounds_center;
        assert!(
            !frustum.intersects_sphere(stale_local_center, bounds_radius),
            "Local-space center at origin should NOT be visible from camera at (50, 5, 50)"
        );
    }

    #[test]
    fn test_frustum_world_aabb_encloses_visible_object() {
        let proj = perspective(Deg(60.0), 16.0 / 9.0, 0.1, 200.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 5.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let (min, max) = super::frustum_world_aabb(&view, &proj)
            .expect("perspective frustum must be invertible");
        // Camera at z=5 looking toward origin (-z). Far plane should be roughly
        // at z = 5 - 200 = -195, near at z = 5 - 0.1 = 4.9.
        assert!(
            min.z <= -190.0 && max.z >= 4.0,
            "frustum z range {:?}..{:?} should span near to far plane",
            min.z,
            max.z
        );
        // X/Y extent grows with distance; far plane half-extent at fov=60,
        // aspect=16/9, far=200 is large.
        assert!(
            max.x.abs() > 10.0 && max.y.abs() > 10.0,
            "frustum should be wide at far plane, got max=({},{},{})",
            max.x,
            max.y,
            max.z
        );
    }

    #[test]
    fn test_frustum_world_aabb_returns_none_for_degenerate_projection() {
        // Zero matrix is non-invertible; helper should bail out cleanly.
        let zero = Matrix4::from_scale(0.0);
        assert!(super::frustum_world_aabb(&zero, &zero).is_none());
    }

    /// Regression test: objects with non-unit scale must have their bounding
    /// radius scaled correctly.
    #[test]
    fn test_scaled_object_bounds() {
        let proj = perspective(Deg(60.0), 1.0, 0.1, 100.0);
        let view = Matrix4::look_at_rh(
            Point3::new(0.0, 0.0, 20.0),
            Point3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        let frustum = Frustum::from_view_projection(&(proj * view));

        // Small object at origin with large scale should still be visible.
        let position = Vector3::new(0.0, 0.0, 0.0);
        let bounds_center = Vector3::new(0.0, 0.0, 0.0);
        let bounds_radius = 0.1f32;
        let scale = Vector3::new(50.0f32, 50.0, 50.0);

        let world_center = position + bounds_center;
        let max_scale = scale.x.max(scale.y).max(scale.z);
        let world_radius = bounds_radius * max_scale;

        assert!(frustum.intersects_sphere(world_center, world_radius));

        // Without scale factor, the tiny radius should be visible (it's at origin
        // which is in view), but this verifies the math is correct.
        assert!(frustum.intersects_sphere(world_center, bounds_radius));
    }
}
