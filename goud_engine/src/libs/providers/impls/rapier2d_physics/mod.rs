//! Rapier2D physics provider implementation.
//!
//! Wraps the `rapier2d` crate behind the engine's `PhysicsProvider` trait,
//! translating between engine handle types and Rapier's internal handles.

pub mod conversions;
#[cfg(test)]
mod tests;

use crossbeam_channel::{unbounded, Receiver};
use rapier2d::prelude::*;
use std::collections::HashMap;

use crate::core::error::{GoudError, GoudResult};
use crate::core::providers::physics::PhysicsProvider;
use crate::core::providers::types::ColliderHandle as EngineColliderHandle;
use crate::core::providers::types::CollisionEvent as EngineCollisionEvent;
use crate::core::providers::types::{
    BodyDesc, BodyHandle, ColliderDesc, ContactPair, DebugShape, JointDesc, JointHandle,
    PhysicsCapabilities, RaycastHit,
};
use crate::core::providers::{Provider, ProviderLifecycle};

use rapier2d::prelude::ColliderHandle as RapierColliderHandle;

/// Physics provider backed by Rapier2D.
pub struct Rapier2DPhysicsProvider {
    physics_pipeline: PhysicsPipeline,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    gravity: Vector<f32>,

    // Handle mappings: engine u64 <-> rapier handles
    body_handles: HashMap<u64, RigidBodyHandle>,
    body_handles_rev: HashMap<RigidBodyHandle, u64>,
    collider_handles: HashMap<u64, RapierColliderHandle>,
    collider_handles_rev: HashMap<RapierColliderHandle, u64>,
    joint_handles: HashMap<u64, ImpulseJointHandle>,
    joint_handles_rev: HashMap<ImpulseJointHandle, u64>,
    next_id: u64,

    // Collision event handling
    collision_events: Vec<EngineCollisionEvent>,
    collision_recv: Receiver<rapier2d::prelude::CollisionEvent>,
    // Stored to keep the channel alive; contact force events are drained
    // to prevent unbounded growth but processed via narrow_phase directly.
    #[allow(dead_code)]
    contact_recv: Receiver<ContactForceEvent>,
    event_handler: ChannelEventCollector,

    capabilities: PhysicsCapabilities,
}

impl Rapier2DPhysicsProvider {
    /// Create a new Rapier2D physics provider with the given gravity vector.
    pub fn new(gravity: [f32; 2]) -> Self {
        let (collision_send, collision_recv) = unbounded();
        let (contact_send, contact_recv) = unbounded();
        let event_handler = ChannelEventCollector::new(collision_send, contact_send);

        Self {
            physics_pipeline: PhysicsPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            gravity: vector![gravity[0], gravity[1]],
            body_handles: HashMap::new(),
            body_handles_rev: HashMap::new(),
            collider_handles: HashMap::new(),
            collider_handles_rev: HashMap::new(),
            joint_handles: HashMap::new(),
            joint_handles_rev: HashMap::new(),
            next_id: 1,
            collision_events: Vec::new(),
            collision_recv,
            contact_recv,
            event_handler,
            capabilities: PhysicsCapabilities {
                supports_continuous_collision: true,
                supports_joints: true,
                max_bodies: u32::MAX,
            },
        }
    }

    /// Allocate the next engine handle ID.
    fn next_handle_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Drain rapier collision events from the channel and convert to engine events.
    fn drain_rapier_collision_events(&mut self) {
        while let Ok(event) = self.collision_recv.try_recv() {
            let (h1, h2, started) = match event {
                rapier2d::prelude::CollisionEvent::Started(h1, h2, _) => (h1, h2, true),
                rapier2d::prelude::CollisionEvent::Stopped(h1, h2, _) => (h1, h2, false),
            };

            // Map collider handles back to body handles
            let body_a = self.collider_body_handle(h1);
            let body_b = self.collider_body_handle(h2);

            if let (Some(a), Some(b)) = (body_a, body_b) {
                self.collision_events.push(EngineCollisionEvent {
                    body_a: BodyHandle(a),
                    body_b: BodyHandle(b),
                    started,
                });
            }
        }
    }

    /// Look up the engine body handle for a rapier collider handle.
    fn collider_body_handle(&self, collider: RapierColliderHandle) -> Option<u64> {
        let parent = self.collider_set.get(collider)?.parent()?;
        self.body_handles_rev.get(&parent).copied()
    }

    /// Look up a rapier body handle from an engine handle, returning an error if not found.
    fn get_rapier_body(&self, handle: BodyHandle) -> GoudResult<RigidBodyHandle> {
        self.body_handles
            .get(&handle.0)
            .copied()
            .ok_or(GoudError::InvalidHandle)
    }
}

impl Provider for Rapier2DPhysicsProvider {
    fn name(&self) -> &str {
        "rapier2d"
    }

    fn version(&self) -> &str {
        "0.22"
    }

    fn capabilities(&self) -> Box<dyn std::any::Any> {
        Box::new(self.capabilities.clone())
    }
}

impl ProviderLifecycle for Rapier2DPhysicsProvider {
    fn init(&mut self) -> GoudResult<()> {
        Ok(())
    }

    fn update(&mut self, delta: f32) -> GoudResult<()> {
        self.integration_parameters.dt = delta;
        self.step(delta)
    }

    fn shutdown(&mut self) {}
}

impl PhysicsProvider for Rapier2DPhysicsProvider {
    fn physics_capabilities(&self) -> &PhysicsCapabilities {
        &self.capabilities
    }

    fn step(&mut self, delta: f32) -> GoudResult<()> {
        self.integration_parameters.dt = delta;

        // Drain any events from the previous step
        self.drain_rapier_collision_events();

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
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
            &self.event_handler,
        );

        // Drain events produced by this step
        self.drain_rapier_collision_events();

        self.query_pipeline.update(&self.collider_set);

        Ok(())
    }

    fn set_gravity(&mut self, gravity: [f32; 2]) {
        self.gravity = vector![gravity[0], gravity[1]];
    }

    fn gravity(&self) -> [f32; 2] {
        [self.gravity.x, self.gravity.y]
    }

    fn create_body(&mut self, desc: &BodyDesc) -> GoudResult<BodyHandle> {
        let body_type = conversions::body_type_from_u32(desc.body_type);
        let rb = RigidBodyBuilder::new(body_type)
            .translation(vector![desc.position[0], desc.position[1]])
            .linear_damping(desc.linear_damping)
            .angular_damping(desc.angular_damping)
            .gravity_scale(desc.gravity_scale)
            .locked_axes(if desc.fixed_rotation {
                LockedAxes::ROTATION_LOCKED
            } else {
                LockedAxes::empty()
            })
            .build();

        let rapier_handle = self.rigid_body_set.insert(rb);
        let engine_id = self.next_handle_id();
        self.body_handles.insert(engine_id, rapier_handle);
        self.body_handles_rev.insert(rapier_handle, engine_id);

        Ok(BodyHandle(engine_id))
    }

    fn destroy_body(&mut self, handle: BodyHandle) {
        if let Some(rapier_handle) = self.body_handles.remove(&handle.0) {
            self.body_handles_rev.remove(&rapier_handle);
            self.rigid_body_set.remove(
                rapier_handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
        }
    }

    fn body_position(&self, handle: BodyHandle) -> GoudResult<[f32; 2]> {
        let rh = self.get_rapier_body(handle)?;
        let t = self
            .rigid_body_set
            .get(rh)
            .ok_or(GoudError::InvalidHandle)?
            .translation();
        Ok([t.x, t.y])
    }

    fn set_body_position(&mut self, handle: BodyHandle, pos: [f32; 2]) -> GoudResult<()> {
        let rh = self.get_rapier_body(handle)?;
        self.rigid_body_set
            .get_mut(rh)
            .ok_or(GoudError::InvalidHandle)?
            .set_translation(vector![pos[0], pos[1]], true);
        Ok(())
    }

    fn body_velocity(&self, handle: BodyHandle) -> GoudResult<[f32; 2]> {
        let rh = self.get_rapier_body(handle)?;
        let v = self
            .rigid_body_set
            .get(rh)
            .ok_or(GoudError::InvalidHandle)?
            .linvel();
        Ok([v.x, v.y])
    }

    fn set_body_velocity(&mut self, handle: BodyHandle, vel: [f32; 2]) -> GoudResult<()> {
        let rh = self.get_rapier_body(handle)?;
        self.rigid_body_set
            .get_mut(rh)
            .ok_or(GoudError::InvalidHandle)?
            .set_linvel(vector![vel[0], vel[1]], true);
        Ok(())
    }

    fn apply_force(&mut self, handle: BodyHandle, force: [f32; 2]) -> GoudResult<()> {
        let rh = self.get_rapier_body(handle)?;
        self.rigid_body_set
            .get_mut(rh)
            .ok_or(GoudError::InvalidHandle)?
            .add_force(vector![force[0], force[1]], true);
        Ok(())
    }

    fn apply_impulse(&mut self, handle: BodyHandle, impulse: [f32; 2]) -> GoudResult<()> {
        let rh = self.get_rapier_body(handle)?;
        self.rigid_body_set
            .get_mut(rh)
            .ok_or(GoudError::InvalidHandle)?
            .apply_impulse(vector![impulse[0], impulse[1]], true);
        Ok(())
    }

    fn create_collider(
        &mut self,
        body: BodyHandle,
        desc: &ColliderDesc,
    ) -> GoudResult<EngineColliderHandle> {
        let rapier_body = self.get_rapier_body(body)?;
        let shape = conversions::shape_from_desc(desc);

        let collider = ColliderBuilder::new(shape)
            .friction(desc.friction)
            .restitution(desc.restitution)
            .sensor(desc.is_sensor)
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .build();

        let rapier_handle =
            self.collider_set
                .insert_with_parent(collider, rapier_body, &mut self.rigid_body_set);

        let engine_id = self.next_handle_id();
        self.collider_handles.insert(engine_id, rapier_handle);
        self.collider_handles_rev.insert(rapier_handle, engine_id);

        Ok(EngineColliderHandle(engine_id))
    }

    fn destroy_collider(&mut self, handle: EngineColliderHandle) {
        if let Some(rapier_handle) = self.collider_handles.remove(&handle.0) {
            self.collider_handles_rev.remove(&rapier_handle);
            self.collider_set.remove(
                rapier_handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
        }
    }

    fn raycast(&self, origin: [f32; 2], dir: [f32; 2], max_dist: f32) -> Option<RaycastHit> {
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

    fn overlap_circle(&self, center: [f32; 2], radius: f32) -> Vec<BodyHandle> {
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

    fn drain_collision_events(&mut self) -> Vec<EngineCollisionEvent> {
        self.drain_rapier_collision_events();
        std::mem::take(&mut self.collision_events)
    }

    fn contact_pairs(&self) -> Vec<ContactPair> {
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

    fn create_joint(&mut self, desc: &JointDesc) -> GoudResult<JointHandle> {
        let rapier_a = self.get_rapier_body(desc.body_a.ok_or(GoudError::InvalidHandle)?)?;
        let rapier_b = self.get_rapier_body(desc.body_b.ok_or(GoudError::InvalidHandle)?)?;
        let (joint, _, _) = conversions::joint_from_desc(desc, rapier_a, rapier_b);
        let rapier_handle = self
            .impulse_joint_set
            .insert(rapier_a, rapier_b, joint, true);
        let engine_id = self.next_handle_id();
        self.joint_handles.insert(engine_id, rapier_handle);
        self.joint_handles_rev.insert(rapier_handle, engine_id);
        Ok(JointHandle(engine_id))
    }

    fn destroy_joint(&mut self, handle: JointHandle) {
        if let Some(rapier_handle) = self.joint_handles.remove(&handle.0) {
            self.joint_handles_rev.remove(&rapier_handle);
            self.impulse_joint_set.remove(rapier_handle, true);
        }
    }

    fn debug_shapes(&self) -> Vec<DebugShape> {
        let mut shapes = Vec::new();
        for (handle, collider) in self.collider_set.iter() {
            let pos = collider.position().translation;
            let rot = collider.position().rotation.angle();

            let is_sensor = collider.is_sensor();
            let color = if is_sensor {
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

            let _ = handle; // suppress unused warning
        }
        shapes
    }
}
