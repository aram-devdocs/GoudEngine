//! 3D-specific types for provider traits.
//!
//! Descriptor structs, query results, and capability reports for 3D physics.
//! These types follow the same FFI-friendly patterns as their 2D counterparts.

use super::types::{BodyHandle, JointKind, JointLimits, JointMotor};

/// Describes a 3D physics body to be created.
#[derive(Debug, Clone)]
pub struct BodyDesc3D {
    /// Initial position as [x, y, z].
    pub position: [f32; 3],
    /// Rotation as quaternion [x, y, z, w].
    pub rotation: [f32; 4],
    /// Body type (0 = static, 1 = dynamic, 2 = kinematic).
    pub body_type: u32,
    /// Linear damping.
    pub linear_damping: f32,
    /// Angular damping.
    pub angular_damping: f32,
    /// Whether gravity applies to this body.
    pub gravity_scale: f32,
    /// Whether continuous collision detection is enabled for this body.
    pub ccd_enabled: bool,
    /// Fixed rotation (no angular velocity).
    pub fixed_rotation: bool,
}

impl Default for BodyDesc3D {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            body_type: 0,
            linear_damping: 0.0,
            angular_damping: 0.0,
            gravity_scale: 1.0,
            ccd_enabled: false,
            fixed_rotation: false,
        }
    }
}

/// Describes a 3D physics collider to be attached to a body.
#[derive(Debug, Clone, Default)]
pub struct ColliderDesc3D {
    /// Collider shape (0 = sphere, 1 = box, 2 = capsule).
    pub shape: u32,
    /// Half-extents for box shapes as [half_w, half_h, half_d].
    pub half_extents: [f32; 3],
    /// Radius for sphere/capsule shapes.
    pub radius: f32,
    /// Half-height for capsule shapes.
    pub half_height: f32,
    /// Friction coefficient.
    pub friction: f32,
    /// Restitution (bounciness).
    pub restitution: f32,
    /// Whether this collider is a sensor (triggers events, no physical response).
    pub is_sensor: bool,
}

/// Describes a 3D physics joint connecting two bodies.
#[derive(Debug, Clone, Default)]
pub struct JointDesc3D {
    /// First body in the joint.
    pub body_a: Option<BodyHandle>,
    /// Second body in the joint.
    pub body_b: Option<BodyHandle>,
    /// High-level joint kind.
    pub kind: JointKind,
    /// Anchor point on body A as [x, y, z] in local space.
    pub anchor_a: [f32; 3],
    /// Anchor point on body B as [x, y, z] in local space.
    pub anchor_b: [f32; 3],
    /// Local axis used by prismatic and revolute joints.
    pub axis: [f32; 3],
    /// Optional travel/rotation limits.
    pub limits: Option<JointLimits>,
    /// Optional motor settings.
    pub motor: Option<JointMotor>,
}

/// Describes a 3D character controller to be created.
#[derive(Debug, Clone)]
pub struct CharacterControllerDesc3D {
    /// Capsule radius.
    pub radius: f32,
    /// Capsule half-height (total height = half_height * 2 + radius * 2).
    pub half_height: f32,
    /// Initial position as [x, y, z].
    pub position: [f32; 3],
    /// Maximum slope angle (radians) the character can climb.
    pub max_slope_angle: f32,
    /// Step height for automatic stair climbing.
    pub step_height: f32,
}

impl Default for CharacterControllerDesc3D {
    fn default() -> Self {
        Self {
            radius: 0.3,
            half_height: 0.5,
            position: [0.0, 0.0, 0.0],
            max_slope_angle: std::f32::consts::FRAC_PI_4, // 45 degrees
            step_height: 0.25,
        }
    }
}

/// Result of a character controller movement.
#[derive(Debug, Clone)]
pub struct CharacterMoveResult3D {
    /// Final position after movement as [x, y, z].
    pub position: [f32; 3],
    /// Whether the character is touching the ground.
    pub grounded: bool,
}

/// Result of a 3D physics raycast query.
#[derive(Debug, Clone)]
pub struct RaycastHit3D {
    /// The body that was hit.
    pub body: BodyHandle,
    /// The hit point in world space as [x, y, z].
    pub point: [f32; 3],
    /// The surface normal at the hit point as [x, y, z].
    pub normal: [f32; 3],
    /// Distance from ray origin to hit point.
    pub distance: f32,
}

/// A 3D contact pair with contact point information.
#[derive(Debug, Clone)]
pub struct ContactPair3D {
    /// First body in contact.
    pub body_a: BodyHandle,
    /// Second body in contact.
    pub body_b: BodyHandle,
    /// Contact normal as [x, y, z].
    pub normal: [f32; 3],
    /// Penetration depth.
    pub depth: f32,
}

/// A 3D debug visualization shape from the physics engine.
#[derive(Debug, Clone)]
pub struct DebugShape3D {
    /// Shape type (0 = sphere, 1 = box, 2 = line).
    pub shape_type: u32,
    /// Position as [x, y, z].
    pub position: [f32; 3],
    /// Size/extent data (interpretation depends on shape_type).
    pub size: [f32; 3],
    /// Rotation as quaternion [x, y, z, w].
    pub rotation: [f32; 4],
    /// Color as [r, g, b, a].
    pub color: [f32; 4],
}

/// Capabilities reported by a 3D physics provider.
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct PhysicsCapabilities3D {
    /// Whether continuous collision detection is supported.
    pub supports_continuous_collision: bool,
    /// Whether joints are supported.
    pub supports_joints: bool,
    /// Maximum number of physics bodies.
    pub max_bodies: u32,
}
