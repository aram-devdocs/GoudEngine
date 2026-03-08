//! Query, debug, and property accessor methods extracted from `Rapier3DPhysicsProvider`.
//!
//! These are inherent methods called by the `PhysicsProvider3D` trait impl
//! in `mod.rs` to keep that file under 500 lines.

use rapier3d::prelude::*;

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::types::{BodyHandle, ContactPair3D, DebugShape3D, RaycastHit3D, ColliderHandle as EngineColliderHandle};

use super::Rapier3DPhysicsProvider;

impl Rapier3DPhysicsProvider {
    /// Cast a ray and return the first hit with normal information.
    pub(crate) fn query_raycast(
        &self,
        origin: [f32; 3],
        dir: [f32; 3],
        max_dist: f32,
    ) -> Option<RaycastHit3D> {
        let ray = Ray::new(
            point![origin[0], origin[1], origin[2]],
            vector![dir[0], dir[1], dir[2]],
        );

        self.query_pipeline
            .cast_ray_and_get_normal(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                max_dist,
                true,
                QueryFilter::default(),
            )
            .and_then(|(collider_handle, intersection)| {
                let rb_handle = self.collider_to_body.get(&collider_handle)?;
                let engine_id = self.body_reverse.get(rb_handle)?;
                let hit_point = ray.point_at(intersection.time_of_impact);
                Some(RaycastHit3D {
                    body: BodyHandle(*engine_id),
                    point: [hit_point.x, hit_point.y, hit_point.z],
                    normal: [
                        intersection.normal.x,
                        intersection.normal.y,
                        intersection.normal.z,
                    ],
                    distance: intersection.time_of_impact,
                })
            })
    }

    /// Find all bodies whose colliders overlap a sphere.
    pub(crate) fn query_overlap_sphere(&self, center: [f32; 3], radius: f32) -> Vec<BodyHandle> {
        let shape = SharedShape::ball(radius);
        let pos = Isometry::translation(center[0], center[1], center[2]);
        let mut results = Vec::new();

        self.query_pipeline.intersections_with_shape(
            &self.rigid_body_set,
            &self.collider_set,
            &pos,
            shape.as_ref(),
            QueryFilter::default(),
            |collider_handle| {
                if let Some(rb_handle) = self.collider_to_body.get(&collider_handle) {
                    if let Some(engine_id) = self.body_reverse.get(rb_handle) {
                        results.push(BodyHandle(*engine_id));
                    }
                }
                true // continue searching
            },
        );

        results
    }

    /// Collect active contact pairs from the narrow phase.
    pub(crate) fn query_contact_pairs(&self) -> Vec<ContactPair3D> {
        let mut pairs = Vec::new();
        for pair in self.narrow_phase.contact_pairs() {
            if !pair.has_any_active_contact {
                continue;
            }
            let body_a = self
                .collider_to_body
                .get(&pair.collider1)
                .and_then(|rb| self.body_reverse.get(rb))
                .copied()
                .unwrap_or(0);
            let body_b = self
                .collider_to_body
                .get(&pair.collider2)
                .and_then(|rb| self.body_reverse.get(rb))
                .copied()
                .unwrap_or(0);

            let (normal, depth) = pair
                .manifolds
                .iter()
                .find(|m| !m.points.is_empty())
                .map(|m| {
                    let n = m.data.normal;
                    let d = m.points[0].dist;
                    ([n.x, n.y, n.z], d)
                })
                .unwrap_or(([0.0, 0.0, 1.0], 0.0));

            pairs.push(ContactPair3D {
                body_a: BodyHandle(body_a),
                body_b: BodyHandle(body_b),
                normal,
                depth,
            });
        }
        pairs
    }

    /// Build debug visualization shapes for all colliders.
    pub(crate) fn query_debug_shapes(&self) -> Vec<DebugShape3D> {
        let mut shapes = Vec::new();
        for (_handle, collider) in self.collider_set.iter() {
            let pos = collider.position().translation;
            let rot = collider.position().rotation;
            let shape_ref = collider.shape();

            let (shape_type, size) = if let Some(ball) = shape_ref.as_ball() {
                let r = ball.radius;
                (0, [r, r, r])
            } else if let Some(cuboid) = shape_ref.as_cuboid() {
                let he = cuboid.half_extents;
                (1, [he.x, he.y, he.z])
            } else {
                (2, [0.1, 0.1, 0.1])
            };

            shapes.push(DebugShape3D {
                shape_type,
                position: [pos.x, pos.y, pos.z],
                size,
                rotation: [rot.i, rot.j, rot.k, rot.w],
                color: [0.0, 1.0, 0.0, 0.5],
            });
        }
        shapes
    }

    /// Get the gravity scale of a body.
    pub(crate) fn query_body_gravity_scale(&self, handle: BodyHandle) -> GoudResult<f32> {
        let rb = self.resolve_body(handle)?;
        Ok(self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?
            .gravity_scale())
    }

    /// Set the gravity scale of a body.
    pub(crate) fn query_set_body_gravity_scale(&mut self, handle: BodyHandle, scale: f32) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        self.rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?
            .set_gravity_scale(scale, true);
        Ok(())
    }

    /// Get the friction of a collider.
    pub(crate) fn query_collider_friction(&self, handle: EngineColliderHandle) -> GoudResult<f32> {
        let rh = self.resolve_collider(handle)?;
        Ok(self
            .collider_set
            .get(rh)
            .ok_or(GoudError::InvalidHandle)?
            .friction())
    }

    /// Set the friction of a collider.
    pub(crate) fn query_set_collider_friction(
        &mut self,
        handle: EngineColliderHandle,
        friction: f32,
    ) -> GoudResult<()> {
        let rh = self.resolve_collider(handle)?;
        self.collider_set
            .get_mut(rh)
            .ok_or(GoudError::InvalidHandle)?
            .set_friction(friction);
        Ok(())
    }

    /// Get the restitution of a collider.
    pub(crate) fn query_collider_restitution(&self, handle: EngineColliderHandle) -> GoudResult<f32> {
        let rh = self.resolve_collider(handle)?;
        Ok(self
            .collider_set
            .get(rh)
            .ok_or(GoudError::InvalidHandle)?
            .restitution())
    }

    /// Set the restitution of a collider.
    pub(crate) fn query_set_collider_restitution(
        &mut self,
        handle: EngineColliderHandle,
        restitution: f32,
    ) -> GoudResult<()> {
        let rh = self.resolve_collider(handle)?;
        self.collider_set
            .get_mut(rh)
            .ok_or(GoudError::InvalidHandle)?
            .set_restitution(restitution);
        Ok(())
    }

    /// Helper method to resolve a body handle.
    fn resolve_body(&self, handle: BodyHandle) -> GoudResult<rapier3d::dynamics::RigidBodyHandle> {
        self.body_map
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }

    /// Helper method to resolve a collider handle.
    fn resolve_collider(&self, handle: EngineColliderHandle) -> GoudResult<rapier3d::geometry::ColliderHandle> {
        self.collider_map
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }
}
