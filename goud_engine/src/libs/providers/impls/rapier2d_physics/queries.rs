//! Query and debug methods extracted from `Rapier2DPhysicsProvider`.
//!
//! These are inherent methods called by the `PhysicsProvider` trait impl
//! in `mod.rs` to keep that file under 500 lines.

use rapier2d::prelude::*;

use crate::core::providers::types::{BodyHandle, ContactPair, DebugShape, RaycastHit};

use super::Rapier2DPhysicsProvider;

impl Rapier2DPhysicsProvider {
    /// Cast a ray and return the first hit, if any.
    pub(crate) fn query_raycast(
        &self,
        origin: [f32; 2],
        dir: [f32; 2],
        max_dist: f32,
    ) -> Option<RaycastHit> {
        let ray = Ray::new(point![origin[0], origin[1]], vector![dir[0], dir[1]]);

        let (collider_handle, toi) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_dist,
            true,
            QueryFilter::default(),
        )?;

        let collider = self.collider_set.get(collider_handle)?;
        let engine_id = self.body_handles_rev.get(&collider.parent()?)?;
        let hit_point = ray.point_at(toi);
        let proj = collider.shape().project_point(
            collider.position(),
            &point![hit_point.x, hit_point.y],
            false,
        );
        let normal = if proj.is_inside {
            [0.0, 0.0]
        } else {
            let d = point![hit_point.x, hit_point.y] - proj.point;
            let len = (d.x * d.x + d.y * d.y).sqrt();
            if len > f32::EPSILON {
                [d.x / len, d.y / len]
            } else {
                [0.0, 0.0]
            }
        };
        Some(RaycastHit {
            body: BodyHandle(*engine_id),
            point: [hit_point.x, hit_point.y],
            normal,
            distance: toi,
        })
    }

    /// Find all bodies whose colliders overlap a circle.
    pub(crate) fn query_overlap_circle(&self, center: [f32; 2], radius: f32) -> Vec<BodyHandle> {
        let shape = Ball::new(radius);
        let shape_pos = Isometry::translation(center[0], center[1]);
        let mut results = Vec::new();

        self.query_pipeline.intersections_with_shape(
            &self.rigid_body_set,
            &self.collider_set,
            &shape_pos,
            &shape,
            QueryFilter::default(),
            |collider_handle| {
                if let Some(collider) = self.collider_set.get(collider_handle) {
                    if let Some(parent) = collider.parent() {
                        if let Some(&engine_id) = self.body_handles_rev.get(&parent) {
                            results.push(BodyHandle(engine_id));
                        }
                    }
                }
                true // continue searching
            },
        );

        results
    }

    /// Collect active contact pairs from the narrow phase.
    pub(crate) fn query_contact_pairs(&self) -> Vec<ContactPair> {
        let mut pairs = Vec::new();
        for pair in self.narrow_phase.contact_pairs() {
            if !pair.has_any_active_contact {
                continue;
            }
            let body_a = self
                .collider_set
                .get(pair.collider1)
                .and_then(|c| c.parent())
                .and_then(|p| self.body_handles_rev.get(&p).copied());
            let body_b = self
                .collider_set
                .get(pair.collider2)
                .and_then(|c| c.parent())
                .and_then(|p| self.body_handles_rev.get(&p).copied());

            if let (Some(a), Some(b)) = (body_a, body_b) {
                for manifold in &pair.manifolds {
                    let normal = manifold.local_n1;
                    let depth = manifold.points.first().map(|p| p.dist).unwrap_or(0.0);
                    pairs.push(ContactPair {
                        body_a: BodyHandle(a),
                        body_b: BodyHandle(b),
                        normal: [normal.x, normal.y],
                        depth,
                    });
                }
            }
        }
        pairs
    }

    /// Build debug visualization shapes for all colliders.
    pub(crate) fn query_debug_shapes(&self) -> Vec<DebugShape> {
        let mut shapes = Vec::new();
        for (_handle, collider) in self.collider_set.iter() {
            let pos = collider.position().translation;
            let rot = collider.position().rotation.angle();

            let color = if collider.is_sensor() {
                [0.0, 1.0, 0.0, 0.5] // green for sensors
            } else {
                [0.0, 1.0, 1.0, 0.5] // cyan for solid
            };

            if let Some(ball) = collider.shape().as_ball() {
                shapes.push(DebugShape {
                    shape_type: 0,
                    position: [pos.x, pos.y],
                    size: [ball.radius, ball.radius],
                    rotation: rot,
                    color,
                });
            } else if let Some(cuboid) = collider.shape().as_cuboid() {
                shapes.push(DebugShape {
                    shape_type: 1,
                    position: [pos.x, pos.y],
                    size: [cuboid.half_extents.x * 2.0, cuboid.half_extents.y * 2.0],
                    rotation: rot,
                    color,
                });
            } else {
                // Fallback: use AABB
                let aabb = collider.compute_aabb();
                let center = aabb.center();
                let extents = aabb.extents();
                shapes.push(DebugShape {
                    shape_type: 1,
                    position: [center.x, center.y],
                    size: [extents.x, extents.y],
                    rotation: 0.0,
                    color,
                });
            }
        }
        shapes
    }
}
