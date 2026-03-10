//! ECS joint component data and conversions.

use crate::core::math::Vec2;
use crate::core::providers::types::{BodyHandle, JointDesc, JointKind, JointLimits, JointMotor};
use crate::ecs::entity::Entity;
use crate::ecs::Component;

/// A physics joint relationship authored at the ECS level.
///
/// The component stores the other entity participating in the joint along with
/// the shared joint configuration. A future physics sync system can translate
/// the entity references into provider body handles via `to_desc`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Joint {
    /// The other entity connected by this joint.
    pub connected_entity: Entity,
    /// High-level joint kind.
    pub kind: JointKind,
    /// Anchor point on the current entity in local space.
    pub anchor_a: Vec2,
    /// Anchor point on the connected entity in local space.
    pub anchor_b: Vec2,
    /// Local axis for prismatic joints.
    pub axis: Vec2,
    /// Optional travel or rotation limits.
    pub limits: Option<JointLimits>,
    /// Optional motor settings.
    pub motor: Option<JointMotor>,
}

impl Joint {
    /// Creates a joint with the given connected entity and kind.
    pub fn new(connected_entity: Entity, kind: JointKind) -> Self {
        Self {
            connected_entity,
            kind,
            anchor_a: Vec2::zero(),
            anchor_b: Vec2::zero(),
            axis: Vec2::unit_x(),
            limits: None,
            motor: None,
        }
    }

    /// Creates a distance joint.
    pub fn distance(connected_entity: Entity) -> Self {
        Self::new(connected_entity, JointKind::Distance)
    }

    /// Creates a revolute joint.
    pub fn revolute(connected_entity: Entity) -> Self {
        Self::new(connected_entity, JointKind::Revolute)
    }

    /// Creates a prismatic joint with the given axis.
    pub fn prismatic(connected_entity: Entity, axis: Vec2) -> Self {
        Self::new(connected_entity, JointKind::Prismatic).with_axis(axis)
    }

    /// Sets the anchor on the current entity.
    pub fn with_anchor_a(mut self, anchor: Vec2) -> Self {
        self.anchor_a = anchor;
        self
    }

    /// Sets the anchor on the connected entity.
    pub fn with_anchor_b(mut self, anchor: Vec2) -> Self {
        self.anchor_b = anchor;
        self
    }

    /// Sets both anchors at once.
    pub fn with_anchors(mut self, anchor_a: Vec2, anchor_b: Vec2) -> Self {
        self.anchor_a = anchor_a;
        self.anchor_b = anchor_b;
        self
    }

    /// Sets the prismatic axis.
    pub fn with_axis(mut self, axis: Vec2) -> Self {
        self.axis = axis;
        self
    }

    /// Sets optional limits.
    pub fn with_limits(mut self, limits: JointLimits) -> Self {
        self.limits = Some(limits);
        self
    }

    /// Sets optional motor settings.
    pub fn with_motor(mut self, motor: JointMotor) -> Self {
        self.motor = Some(motor);
        self
    }

    /// Converts the ECS-authored joint into a provider descriptor.
    pub fn to_desc(&self, body_a: BodyHandle, body_b: BodyHandle) -> JointDesc {
        JointDesc {
            body_a: Some(body_a),
            body_b: Some(body_b),
            kind: self.kind,
            anchor_a: [self.anchor_a.x, self.anchor_a.y],
            anchor_b: [self.anchor_b.x, self.anchor_b.y],
            axis: [self.axis.x, self.axis.y],
            limits: self.limits,
            motor: self.motor,
        }
    }
}

impl Default for Joint {
    fn default() -> Self {
        Self::revolute(Entity::PLACEHOLDER)
    }
}

impl Component for Joint {}
