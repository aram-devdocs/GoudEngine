//! [`RigidBody`] component struct, flags, constructors, builder, accessors, and mutators.

use crate::core::math::Vec2;

use super::body_type::RigidBodyType;

// =============================================================================
// RigidBodyFlags (private)
// =============================================================================

/// Bit flags for rigid body state.
pub(super) struct RigidBodyFlags;

impl RigidBodyFlags {
    /// Body is currently sleeping (optimization).
    pub(super) const SLEEPING: u8 = 1 << 0;
    /// Body can sleep when idle.
    pub(super) const CAN_SLEEP: u8 = 1 << 1;
    /// Continuous collision detection enabled.
    pub(super) const CONTINUOUS_CD: u8 = 1 << 2;
    /// Fixed rotation (prevents rotation from collisions).
    pub(super) const FIXED_ROTATION: u8 = 1 << 3;
}

// =============================================================================
// RigidBody Component
// =============================================================================

/// A physics body component for 2D physics simulation.
///
/// RigidBody stores all the physics state for an entity, including velocity,
/// forces, mass, and behavior type. It works together with the `Transform2D`
/// component (for position/rotation) and optionally with `Collider` components
/// for collision detection.
///
/// # Memory Layout
///
/// The component is laid out as:
/// - `body_type`: u8 (1 byte) + 3 bytes padding
/// - `linear_velocity`: Vec2 (8 bytes)
/// - `angular_velocity`: f32 (4 bytes)
/// - `linear_damping`: f32 (4 bytes)
/// - `angular_damping`: f32 (4 bytes)
/// - `mass`: f32 (4 bytes)
/// - `inverse_mass`: f32 (4 bytes)
/// - `inertia`: f32 (4 bytes)
/// - `inverse_inertia`: f32 (4 bytes)
/// - `gravity_scale`: f32 (4 bytes)
/// - `flags`: u8 (1 byte) + 3 bytes padding
/// - `sleep_time`: f32 (4 bytes)
/// - Total: 56 bytes (may vary with padding)
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::components::{RigidBody, RigidBodyType};
/// use goud_engine::core::math::Vec2;
///
/// // Dynamic body with custom properties
/// let body = RigidBody::dynamic()
///     .with_mass(2.0)
///     .with_velocity(Vec2::new(100.0, 0.0))
///     .with_gravity_scale(1.5);
///
/// // Kinematic platform
/// let platform = RigidBody::kinematic()
///     .with_velocity(Vec2::new(50.0, 0.0));
///
/// // Static wall
/// let wall = RigidBody::static_body();
/// ```
///
/// # Material Properties
///
/// Material properties (restitution, friction) live on [`Collider`], not `RigidBody`.
/// Set them via [`Collider::with_restitution`] and [`Collider::with_friction`].
///
/// [`Collider`]: crate::ecs::components::Collider
/// [`Collider::with_restitution`]: crate::ecs::components::Collider::with_restitution
/// [`Collider::with_friction`]: crate::ecs::components::Collider::with_friction
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RigidBody {
    /// Physics behavior type (Dynamic, Kinematic, or Static).
    pub body_type: RigidBodyType,

    /// Linear velocity in pixels/second.
    pub linear_velocity: Vec2,

    /// Angular velocity in radians/second.
    pub angular_velocity: f32,

    /// Linear damping (air resistance) - 0.0 = no damping, 1.0 = full damping.
    /// Applied each frame as: velocity *= (1.0 - damping * dt)
    pub linear_damping: f32,

    /// Angular damping (rotational resistance) - 0.0 = no damping, 1.0 = full damping.
    pub angular_damping: f32,

    /// Mass in arbitrary units (kg assumed for consistency).
    /// Must be positive for dynamic bodies.
    pub mass: f32,

    /// Inverse mass (1.0 / mass). Pre-calculated for performance.
    /// Zero for static/kinematic bodies (infinite mass).
    pub inverse_mass: f32,

    /// Rotational inertia (moment of inertia) in kg·m².
    /// Calculated from mass and collider shape.
    pub inertia: f32,

    /// Inverse inertia (1.0 / inertia). Pre-calculated for performance.
    /// Zero for static/kinematic bodies.
    pub inverse_inertia: f32,

    /// Gravity scale multiplier.
    /// - 0.0 = no gravity
    /// - 1.0 = normal gravity
    /// - 2.0 = double gravity
    pub gravity_scale: f32,

    /// Bit flags for body state (see RigidBodyFlags).
    pub(super) flags: u8,

    /// Time the body has been below sleep thresholds (seconds).
    /// When this exceeds PhysicsWorld::sleep_time_threshold, the body sleeps.
    pub(super) sleep_time: f32,
}

// =============================================================================
// Constructors
// =============================================================================

impl Default for RigidBody {
    /// Creates a default dynamic rigid body.
    fn default() -> Self {
        Self::dynamic()
    }
}

impl RigidBody {
    /// Creates a new rigid body with custom parameters.
    ///
    /// # Arguments
    ///
    /// * `body_type` - Physics behavior type
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::{RigidBody, RigidBodyType};
    ///
    /// let body = RigidBody::new(RigidBodyType::Dynamic);
    /// ```
    pub fn new(body_type: RigidBodyType) -> Self {
        let (mass, inverse_mass) = if body_type == RigidBodyType::Dynamic {
            (1.0, 1.0)
        } else {
            (0.0, 0.0) // Infinite mass for static/kinematic
        };

        Self {
            body_type,
            linear_velocity: Vec2::zero(),
            angular_velocity: 0.0,
            linear_damping: 0.01, // Slight air resistance
            angular_damping: 0.01,
            mass,
            inverse_mass,
            inertia: 1.0,
            inverse_inertia: if body_type == RigidBodyType::Dynamic {
                1.0
            } else {
                0.0
            },
            gravity_scale: 1.0,               // Full gravity
            flags: RigidBodyFlags::CAN_SLEEP, // Can sleep by default
            sleep_time: 0.0,
        }
    }

    /// Creates a dynamic rigid body (fully simulated).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::RigidBody;
    ///
    /// let player = RigidBody::dynamic()
    ///     .with_mass(2.0);
    /// ```
    #[inline]
    pub fn dynamic() -> Self {
        Self::new(RigidBodyType::Dynamic)
    }

    /// Creates a kinematic rigid body (controlled velocity).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::RigidBody;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let platform = RigidBody::kinematic()
    ///     .with_velocity(Vec2::new(50.0, 0.0));
    /// ```
    #[inline]
    pub fn kinematic() -> Self {
        Self::new(RigidBodyType::Kinematic)
    }

    /// Creates a static rigid body (immovable).
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::components::RigidBody;
    ///
    /// let wall = RigidBody::static_body();
    /// ```
    #[inline]
    pub fn static_body() -> Self {
        Self::new(RigidBodyType::Static)
    }

    // =========================================================================
    // Builder Pattern
    // =========================================================================

    /// Sets the linear velocity.
    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.linear_velocity = velocity;
        self
    }

    /// Sets the angular velocity.
    pub fn with_angular_velocity(mut self, angular_velocity: f32) -> Self {
        self.angular_velocity = angular_velocity;
        self
    }

    /// Sets the mass (dynamic bodies only).
    ///
    /// # Panics
    ///
    /// Panics if mass is not positive and finite.
    pub fn with_mass(mut self, mass: f32) -> Self {
        assert!(
            mass > 0.0 && mass.is_finite(),
            "Mass must be positive and finite"
        );
        self.mass = mass;
        self.inverse_mass = 1.0 / mass;
        self
    }

    /// Sets the linear damping.
    pub fn with_linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = damping.max(0.0);
        self
    }

    /// Sets the angular damping.
    pub fn with_angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = damping.max(0.0);
        self
    }

    /// Sets the gravity scale.
    pub fn with_gravity_scale(mut self, scale: f32) -> Self {
        self.gravity_scale = scale;
        self
    }

    /// Sets whether the body can sleep.
    pub fn with_can_sleep(mut self, can_sleep: bool) -> Self {
        if can_sleep {
            self.flags |= RigidBodyFlags::CAN_SLEEP;
        } else {
            self.flags &= !RigidBodyFlags::CAN_SLEEP;
        }
        self
    }

    /// Enables continuous collision detection (for fast-moving objects).
    pub fn with_continuous_cd(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= RigidBodyFlags::CONTINUOUS_CD;
        } else {
            self.flags &= !RigidBodyFlags::CONTINUOUS_CD;
        }
        self
    }

    /// Fixes the rotation (prevents rotation from collisions).
    pub fn with_fixed_rotation(mut self, fixed: bool) -> Self {
        if fixed {
            self.flags |= RigidBodyFlags::FIXED_ROTATION;
            self.angular_velocity = 0.0;
            self.inertia = 0.0;
            self.inverse_inertia = 0.0;
        } else {
            self.flags &= !RigidBodyFlags::FIXED_ROTATION;
        }
        self
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns the body type.
    #[inline]
    pub fn body_type(&self) -> RigidBodyType {
        self.body_type
    }

    /// Returns true if this is a dynamic body.
    #[inline]
    pub fn is_dynamic(&self) -> bool {
        self.body_type == RigidBodyType::Dynamic
    }

    /// Returns true if this is a kinematic body.
    #[inline]
    pub fn is_kinematic(&self) -> bool {
        self.body_type == RigidBodyType::Kinematic
    }

    /// Returns true if this is a static body.
    #[inline]
    pub fn is_static(&self) -> bool {
        self.body_type == RigidBodyType::Static
    }

    /// Returns true if the body is currently sleeping.
    #[inline]
    pub fn is_sleeping(&self) -> bool {
        self.flags & RigidBodyFlags::SLEEPING != 0
    }

    /// Returns true if the body can sleep.
    #[inline]
    pub fn can_sleep(&self) -> bool {
        self.flags & RigidBodyFlags::CAN_SLEEP != 0
    }

    /// Returns true if continuous collision detection is enabled.
    #[inline]
    pub fn has_continuous_cd(&self) -> bool {
        self.flags & RigidBodyFlags::CONTINUOUS_CD != 0
    }

    /// Returns true if rotation is fixed.
    #[inline]
    pub fn has_fixed_rotation(&self) -> bool {
        self.flags & RigidBodyFlags::FIXED_ROTATION != 0
    }

    /// Returns the current sleep time.
    #[inline]
    pub fn sleep_time(&self) -> f32 {
        self.sleep_time
    }

    /// Returns the linear speed (magnitude of velocity).
    #[inline]
    pub fn linear_speed(&self) -> f32 {
        self.linear_velocity.length()
    }

    /// Returns the linear speed squared (avoids sqrt).
    #[inline]
    pub fn linear_speed_squared(&self) -> f32 {
        self.linear_velocity.length_squared()
    }

    /// Returns the kinetic energy of the body (0.5 * m * v²).
    #[inline]
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.linear_speed_squared()
    }

    // =========================================================================
    // Mutators
    // =========================================================================

    /// Sets the linear velocity.
    pub fn set_velocity(&mut self, velocity: Vec2) {
        self.linear_velocity = velocity;
        self.wake();
    }

    /// Sets the angular velocity.
    pub fn set_angular_velocity(&mut self, angular_velocity: f32) {
        self.angular_velocity = angular_velocity;
        self.wake();
    }

    /// Sets the mass (dynamic bodies only).
    ///
    /// # Panics
    ///
    /// Panics if mass is not positive and finite, or if called on non-dynamic body.
    pub fn set_mass(&mut self, mass: f32) {
        assert!(self.is_dynamic(), "Cannot set mass on non-dynamic body");
        assert!(
            mass > 0.0 && mass.is_finite(),
            "Mass must be positive and finite"
        );
        self.mass = mass;
        self.inverse_mass = 1.0 / mass;
    }

    /// Sets the body type, updating mass accordingly.
    pub fn set_body_type(&mut self, body_type: RigidBodyType) {
        if self.body_type == body_type {
            return;
        }

        self.body_type = body_type;

        // Update mass/inertia based on new type
        if body_type == RigidBodyType::Dynamic {
            if self.mass == 0.0 {
                self.mass = 1.0;
            }
            self.inverse_mass = 1.0 / self.mass;
            if self.inertia == 0.0 {
                self.inertia = 1.0;
            }
            self.inverse_inertia = 1.0 / self.inertia;
        } else {
            self.inverse_mass = 0.0;
            self.inverse_inertia = 0.0;
        }

        self.wake();
    }

    // =========================================================================
    // Internal Helpers (used by physics_ops submodule)
    // =========================================================================

    /// Sets or clears the sleeping flag without touching velocity.
    #[inline]
    pub(super) fn set_sleeping(&mut self, sleeping: bool) {
        if sleeping {
            self.flags |= RigidBodyFlags::SLEEPING;
        } else {
            self.flags &= !RigidBodyFlags::SLEEPING;
        }
    }

    /// Resets accumulated sleep time to zero.
    #[inline]
    pub(super) fn reset_sleep_time(&mut self) {
        self.sleep_time = 0.0;
    }

    /// Adds `dt` to the accumulated sleep time.
    #[inline]
    pub(super) fn increment_sleep_time(&mut self, dt: f32) {
        self.sleep_time += dt;
    }
}
