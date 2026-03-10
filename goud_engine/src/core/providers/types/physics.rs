//! Shared 2D physics descriptor, event, and capability types.

use super::handles::{BodyHandle, ColliderHandle};

/// Describes a physics body to be created.
#[derive(Debug, Clone)]
pub struct BodyDesc {
    /// Initial position as [x, y].
    pub position: [f32; 2],
    /// Body type (0 = static, 1 = dynamic, 2 = kinematic).
    pub body_type: u32,
    /// Linear damping.
    pub linear_damping: f32,
    /// Angular damping.
    pub angular_damping: f32,
    /// Gravity scale.
    pub gravity_scale: f32,
    /// Whether continuous collision detection is enabled for this body.
    pub ccd_enabled: bool,
    /// Fixed rotation (no angular velocity).
    pub fixed_rotation: bool,
}

/// Selection of 2D physics backends available to `EngineConfig` and native FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum PhysicsBackend2D {
    /// Use whatever default backend `EngineConfig` currently selects.
    #[default]
    Default = 0,
    /// Use the Rapier2D backend.
    Rapier = 1,
    /// Use the lightweight simple 2D backend.
    Simple = 2,
}

impl PhysicsBackend2D {
    /// Attempts to convert a raw backend selector into a validated enum value.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Default),
            1 => Some(Self::Rapier),
            2 => Some(Self::Simple),
            _ => None,
        }
    }
}

impl Default for BodyDesc {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            body_type: 0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            ccd_enabled: false,
            fixed_rotation: false,
        }
    }
}

/// Describes a physics collider to be attached to a body.
#[derive(Debug, Clone)]
pub struct ColliderDesc {
    /// Collider shape (0 = circle, 1 = box, 2 = capsule).
    pub shape: u32,
    /// Half-extents for box shapes as [half_w, half_h].
    pub half_extents: [f32; 2],
    /// Radius for circle/capsule shapes.
    pub radius: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Restitution (bounciness).
    pub restitution: f32,
    /// Whether this collider is a sensor (triggers events, no physical response).
    pub is_sensor: bool,
    /// Collision layer membership bitmask.
    ///
    /// Defaults to layer 1 (`0b0001`).
    pub layer: u32,
    /// Collision mask bitmask.
    ///
    /// A collider interacts when its layer overlaps the other collider's mask,
    /// and vice-versa.
    pub mask: u32,
}

impl Default for ColliderDesc {
    fn default() -> Self {
        Self {
            shape: 0,
            half_extents: [0.0, 0.0],
            radius: 0.0,
            friction: 0.0,
            restitution: 0.0,
            is_sensor: false,
            layer: 0b0001,
            mask: u32::MAX,
        }
    }
}

/// Describes the kind of joint connecting two bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum JointKind {
    /// Distance-limited joint.
    Distance,
    /// Revolute/hinge joint.
    #[default]
    Revolute,
    /// Prismatic/slider joint.
    Prismatic,
}

/// Shared joint limits configuration.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JointLimits {
    /// Minimum travel/rotation along the constrained axis.
    pub min: f32,
    /// Maximum travel/rotation along the constrained axis.
    pub max: f32,
}

/// Shared joint motor configuration.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct JointMotor {
    /// Desired angular or linear velocity.
    pub target_velocity: f32,
    /// Maximum force/torque the motor can apply.
    pub max_force: f32,
}

/// Describes a physics joint connecting two bodies.
#[derive(Debug, Clone, Default)]
pub struct JointDesc {
    /// First body in the joint.
    pub body_a: Option<BodyHandle>,
    /// Second body in the joint.
    pub body_b: Option<BodyHandle>,
    /// High-level joint kind.
    pub kind: JointKind,
    /// Anchor point on body A as [x, y] in local space.
    pub anchor_a: [f32; 2],
    /// Anchor point on body B as [x, y] in local space.
    pub anchor_b: [f32; 2],
    /// Local axis used by prismatic joints.
    pub axis: [f32; 2],
    /// Optional travel/rotation limits.
    pub limits: Option<JointLimits>,
    /// Optional motor settings.
    pub motor: Option<JointMotor>,
}

/// Result of a physics raycast query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    /// The body that was hit.
    pub body: BodyHandle,
    /// The collider that was hit.
    pub collider: ColliderHandle,
    /// The hit point in world space as [x, y].
    pub point: [f32; 2],
    /// The surface normal at the hit point as [x, y].
    pub normal: [f32; 2],
    /// Distance from ray origin to hit point.
    pub distance: f32,
}

/// Collision event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionEventKind {
    /// Two bodies started overlapping this step.
    Enter,
    /// Two bodies continued overlapping this step.
    Stay,
    /// Two bodies stopped overlapping this step.
    Exit,
}

/// A collision event between two bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CollisionEvent {
    /// First body involved in the collision.
    pub body_a: BodyHandle,
    /// Second body involved in the collision.
    pub body_b: BodyHandle,
    /// Collision lifecycle kind.
    pub kind: CollisionEventKind,
}

/// A contact pair with contact point information.
#[derive(Debug, Clone)]
pub struct ContactPair {
    /// First body in contact.
    pub body_a: BodyHandle,
    /// Second body in contact.
    pub body_b: BodyHandle,
    /// Contact normal as [x, y].
    pub normal: [f32; 2],
    /// Penetration depth.
    pub depth: f32,
}

/// A debug visualization shape from the physics engine.
#[derive(Debug, Clone)]
pub struct DebugShape {
    /// Shape type (0 = circle, 1 = box, 2 = line).
    pub shape_type: u32,
    /// Position as [x, y].
    pub position: [f32; 2],
    /// Size/extent data (interpretation depends on shape_type).
    pub size: [f32; 2],
    /// Rotation in radians.
    pub rotation: f32,
    /// Color as [r, g, b, a].
    pub color: [f32; 4],
}
