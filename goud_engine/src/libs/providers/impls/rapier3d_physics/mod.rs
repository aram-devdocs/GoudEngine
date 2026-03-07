//! Rapier3D physics provider implementation.
//!
//! Wraps the `rapier3d` crate behind the `PhysicsProvider3D` trait, providing
//! 3D rigid-body simulation, collision detection, raycasting, and joints.

mod conversions;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::physics3d::PhysicsProvider3D;
use crate::core::providers::types::ColliderHandle as EngineColliderHandle;
use crate::core::providers::types::CollisionEvent as EngineCollisionEvent;
use crate::core::providers::types::{
    BodyDesc3D, ColliderDesc3D, ContactPair3D, DebugShape3D, JointDesc3D, PhysicsCapabilities3D,
    RaycastHit3D,
};
use crate::core::providers::types::{BodyHandle, JointHandle};
use crate::core::providers::{Provider, ProviderLifecycle};

use rapier3d::prelude::ColliderHandle as RapierColliderHandle;

use conversions::{body_type_from_u32, joint_from_desc, shape_from_desc};

/// A 3D physics provider backed by Rapier3D.
pub struct Rapier3DPhysicsProvider {
    capabilities: PhysicsCapabilities3D,
    gravity: Vector<f32>,
    integration_params: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,

    // Handle mapping: engine u64 <-> rapier handles
    next_body_id: u64,
    next_collider_id: u64,
    next_joint_id: u64,
    body_map: HashMap<u64, RigidBodyHandle>,
    body_reverse: HashMap<RigidBodyHandle, u64>,
    collider_map: HashMap<u64, RapierColliderHandle>,
    collider_to_body: HashMap<RapierColliderHandle, RigidBodyHandle>,
    joint_map: HashMap<u64, ImpulseJointHandle>,

    // Collision event buffer
    collision_events: Vec<EngineCollisionEvent>,
    event_collector: CollisionEventCollector,
}

/// Collector that buffers collision events from rapier's event handler.
struct CollisionEventCollector {
    events: Vec<(
        rapier3d::prelude::ColliderHandle,
        rapier3d::prelude::ColliderHandle,
        bool,
    )>,
}

impl CollisionEventCollector {
    fn new() -> Self {
        Self { events: Vec::new() }
    }

    fn drain(
        &mut self,
    ) -> Vec<(
        rapier3d::prelude::ColliderHandle,
        rapier3d::prelude::ColliderHandle,
        bool,
    )> {
        std::mem::take(&mut self.events)
    }
}

impl Rapier3DPhysicsProvider {
    /// Create a new Rapier3D physics provider with default gravity [0, -9.81, 0].
    pub fn new() -> Self {
        Self {
            capabilities: PhysicsCapabilities3D {
                supports_continuous_collision: true,
                supports_joints: true,
                max_bodies: u32::MAX,
            },
            gravity: vector![0.0, -9.81, 0.0],
            integration_params: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),

            next_body_id: 1,
            next_collider_id: 1,
            next_joint_id: 1,
            body_map: HashMap::new(),
            body_reverse: HashMap::new(),
            collider_map: HashMap::new(),
            collider_to_body: HashMap::new(),
            joint_map: HashMap::new(),

            collision_events: Vec::new(),
            event_collector: CollisionEventCollector::new(),
        }
    }

    /// Look up the rapier body handle for an engine handle, returning an error
    /// if not found.
    fn resolve_body(&self, handle: BodyHandle) -> GoudResult<RigidBodyHandle> {
        self.body_map
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }
}

impl Default for Rapier3DPhysicsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Provider for Rapier3DPhysicsProvider {
    fn name(&self) -> &str {
        "rapier3d"
    }

    fn version(&self) -> &str {
        "0.22"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for Rapier3DPhysicsProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.step(delta)
    }

    fn shutdown(&mut self) {}
}

impl PhysicsProvider3D for Rapier3DPhysicsProvider {
    fn physics_capabilities(&self) -> &PhysicsCapabilities3D {
        &self.capabilities
    }

    fn step(&mut self, delta: f32) -> GoudResult<()> {
        self.integration_params.dt = delta;

        let (collision_send, collision_recv) = crossbeam_channel::unbounded();
        let (contact_force_send, contact_force_recv) = crossbeam_channel::unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &event_handler,
        );

        // Collect collision events
        while let Ok(event) = collision_recv.try_recv() {
            let started = event.started();
            let (h1, h2) = match event {
                rapier3d::geometry::CollisionEvent::Started(a, b, _) => (a, b),
                rapier3d::geometry::CollisionEvent::Stopped(a, b, _) => (a, b),
            };
            self.event_collector.events.push((h1, h2, started));
        }

        // Drain contact force events to avoid channel buildup
        while contact_force_recv.try_recv().is_ok() {}

        // Convert buffered rapier events to engine events
        for (h1, h2, started) in self.event_collector.drain() {
            let body_a = self
                .collider_to_body
                .get(&h1)
                .and_then(|rb| self.body_reverse.get(rb))
                .copied()
                .unwrap_or(0);
            let body_b = self
                .collider_to_body
                .get(&h2)
                .and_then(|rb| self.body_reverse.get(rb))
                .copied()
                .unwrap_or(0);
            self.collision_events.push(EngineCollisionEvent {
                body_a: BodyHandle(body_a),
                body_b: BodyHandle(body_b),
                started,
            });
        }

        Ok(())
    }

    fn set_gravity(&mut self, gravity: [f32; 3]) {
        self.gravity = vector![gravity[0], gravity[1], gravity[2]];
    }

    fn gravity(&self) -> [f32; 3] {
        [self.gravity.x, self.gravity.y, self.gravity.z]
    }

    fn create_body(&mut self, desc: &BodyDesc3D) -> GoudResult<BodyHandle> {
        let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
            desc.rotation[3],
            desc.rotation[0],
            desc.rotation[1],
            desc.rotation[2],
        ));
        let body_type = body_type_from_u32(desc.body_type);
        let builder = match body_type {
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            _ => RigidBodyBuilder::kinematic_position_based(),
        };
        let body = builder
            .translation(vector![
                desc.position[0],
                desc.position[1],
                desc.position[2]
            ])
            .rotation(rotation.scaled_axis())
            .linear_damping(desc.linear_damping)
            .angular_damping(desc.angular_damping)
            .gravity_scale(desc.gravity_scale)
            .locked_axes(if desc.fixed_rotation {
                LockedAxes::ROTATION_LOCKED
            } else {
                LockedAxes::empty()
            })
            .build();

        let rb_handle = self.rigid_body_set.insert(body);
        let id = self.next_body_id;
        self.next_body_id += 1;
        self.body_map.insert(id, rb_handle);
        self.body_reverse.insert(rb_handle, id);
        Ok(BodyHandle(id))
    }

    fn destroy_body(&mut self, handle: BodyHandle) {
        if let Some(rb_handle) = self.body_map.remove(&handle.0) {
            self.body_reverse.remove(&rb_handle);
            self.rigid_body_set.remove(
                rb_handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }

    fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 3]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let t = body.translation();
        Ok([t.x, t.y, t.z])
    }

    fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_translation(vector![pos[0], pos[1], pos[2]], true);
        Ok(())
    }

    fn body_rotation(&self, handle: BodyHandle) -> GoudResult<[f32; 4]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let q = body.rotation();
        Ok([q.i, q.j, q.k, q.w])
    }

    fn set_body_rotation(&mut self, handle: BodyHandle, rot: [f32; 4]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let rotation =
            UnitQuaternion::from_quaternion(Quaternion::new(rot[3], rot[0], rot[1], rot[2]));
        body.set_rotation(rotation, true);
        Ok(())
    }

    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 3]> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get(rb)
            .ok_or(GoudError::InvalidHandle)?;
        let v = body.linvel();
        Ok([v.x, v.y, v.z])
    }

    fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.set_linvel(vector![vel[0], vel[1], vel[2]], true);
        Ok(())
    }

    fn apply_force(&mut self, handle: BodyHandle, force: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.add_force(vector![force[0], force[1], force[2]], true);
        Ok(())
    }

    fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 3]) -> GoudResult<()> {
        let rb = self.resolve_body(handle)?;
        let body = self
            .rigid_body_set
            .get_mut(rb)
            .ok_or(GoudError::InvalidHandle)?;
        body.apply_impulse(vector![impulse[0], impulse[1], impulse[2]], true);
        Ok(())
    }

    fn create_collider(
        &mut self,
        body: BodyHandle,
        desc: &ColliderDesc3D,
    ) -> GoudResult<EngineColliderHandle> {
        let rb = self.resolve_body(body)?;
        let shape = shape_from_desc(desc);
        let collider = ColliderBuilder::new(shape)
            .friction(desc.friction)
            .restitution(desc.restitution)
            .sensor(desc.is_sensor)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let rapier_handle =
            self.collider_set
                .insert_with_parent(collider, rb, &mut self.rigid_body_set);

        let id = self.next_collider_id;
        self.next_collider_id += 1;
        self.collider_map.insert(id, rapier_handle);
        self.collider_to_body.insert(rapier_handle, rb);
        Ok(EngineColliderHandle(id))
    }

    fn destroy_collider(&mut self, handle: EngineColliderHandle) {
        if let Some(rapier_handle) = self.collider_map.remove(&handle.0) {
            self.collider_to_body.remove(&rapier_handle);
            self.collider_set.remove(
                rapier_handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
        }
    }

    fn raycast(&self, origin: [f32; 3], dir: [f32; 3], max_dist: f32) -> Option<RaycastHit3D> {
        let ray = Ray::new(
            point![origin[0], origin[1], origin[2]],
            vector![dir[0], dir[1], dir[2]],
        );
        let filter = QueryFilter::default();

        self.query_pipeline
            .cast_ray_and_get_normal(
                &self.rigid_body_set,
                &self.collider_set,
                &ray,
                max_dist,
                true,
                filter,
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

    fn overlap_sphere(&self, center: [f32; 3], radius: f32) -> Vec<BodyHandle> {
        let shape = SharedShape::ball(radius);
        let pos = Isometry::translation(center[0], center[1], center[2]);
        let filter = QueryFilter::default();
        let mut results = Vec::new();

        self.query_pipeline.intersections_with_shape(
            &self.rigid_body_set,
            &self.collider_set,
            &pos,
            shape.as_ref(),
            filter,
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

    fn drain_collision_events(&mut self) -> Vec<EngineCollisionEvent> {
        std::mem::take(&mut self.collision_events)
    }

    fn contact_pairs(&self) -> Vec<ContactPair3D> {
        let mut pairs = Vec::new();
        for pair in self.narrow_phase.contact_pairs() {
            if pair.has_any_active_contact {
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

                // Extract first contact manifold data
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
        }
        pairs
    }

    fn create_joint(&mut self, desc: &JointDesc3D) -> GoudResult<JointHandle> {
        let body_a_handle = desc
            .body_a
            .ok_or(GoudError::InvalidHandle)
            .and_then(|h| self.resolve_body(h))?;
        let body_b_handle = desc
            .body_b
            .ok_or(GoudError::InvalidHandle)
            .and_then(|h| self.resolve_body(h))?;

        let joint = joint_from_desc(desc, body_a_handle, body_b_handle);
        let rapier_handle =
            self.impulse_joint_set
                .insert(body_a_handle, body_b_handle, joint, true);

        let id = self.next_joint_id;
        self.next_joint_id += 1;
        self.joint_map.insert(id, rapier_handle);
        Ok(JointHandle(id))
    }

    fn destroy_joint(&mut self, handle: JointHandle) {
        if let Some(rapier_handle) = self.joint_map.remove(&handle.0) {
            self.impulse_joint_set.remove(rapier_handle, true);
        }
    }

    fn debug_shapes(&self) -> Vec<DebugShape3D> {
        let mut shapes = Vec::new();
        for (handle, collider) in self.collider_set.iter() {
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

            let _ = handle; // suppress unused warning

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
}
