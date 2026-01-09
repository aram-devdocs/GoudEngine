//! RigidBody component for physics simulation.
//!
//! The [`RigidBody`] component marks an entity as a physics object that participates
//! in physics simulation. It controls the entity's physics behavior type (dynamic,
//! kinematic, or static) and stores physics state like velocity and forces.
//!
//! # Physics Behavior Types
//!
//! - **Dynamic**: Fully simulated, affected by forces and collisions
//! - **Kinematic**: Moves via velocity but not affected by forces
//! - **Static**: Immovable, acts as obstacles for other bodies
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::components::{RigidBody, RigidBodyType};
//! use goud_engine::core::math::Vec2;
//!
//! // Create a dynamic body (player, enemies, etc.)
//! let player = RigidBody::dynamic()
//!     .with_velocity(Vec2::new(100.0, 0.0))
//!     .with_mass(1.0);
//!
//! // Create a kinematic body (moving platforms)
//! let platform = RigidBody::kinematic()
//!     .with_velocity(Vec2::new(50.0, 0.0));
//!
//! // Create a static body (walls, floors)
//! let wall = RigidBody::static_body();
//! ```
//!
//! # Integration with Physics Systems
//!
//! The physics system reads RigidBody components to:
//! - Apply forces and impulses
//! - Integrate velocity to update position
//! - Handle collisions and constraints
//! - Implement sleeping for optimization
//!
//! # Thread Safety
//!
//! RigidBody is `Send + Sync` and can be safely used in parallel systems.

use crate::core::math::Vec2;
use crate::ecs::Component;

// =============================================================================
// RigidBodyType Enum
// =============================================================================

/// Defines the physics behavior of a rigid body.
///
/// Different body types interact with the physics system in different ways.
/// This determines whether the body is affected by forces, can be moved by
/// the simulation, and how it collides with other bodies.
///
/// # FFI Safety
///
/// `#[repr(u8)]` ensures this enum has a stable ABI for FFI.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RigidBodyType {
    /// Dynamic bodies are fully simulated by the physics engine.
    ///
    /// - Affected by gravity and forces
    /// - Responds to collisions
    /// - Can be moved by constraints
    /// - Most expensive to simulate
    ///
    /// Use for: players, enemies, projectiles, movable objects
    Dynamic = 0,

    /// Kinematic bodies move via velocity but are not affected by forces.
    ///
    /// - NOT affected by gravity or forces
    /// - Does NOT respond to collisions (but affects other bodies)
    /// - Moved by setting velocity or position directly
    /// - Cheaper than dynamic
    ///
    /// Use for: moving platforms, elevators, doors, cutscene objects
    Kinematic = 1,

    /// Static bodies do not move and are not affected by forces.
    ///
    /// - Immovable
    /// - NOT affected by gravity or forces
    /// - Acts as obstacles for other bodies
    /// - Cheapest to simulate (often excluded from updates)
    ///
    /// Use for: walls, floors, terrain, static obstacles
    Static = 2,
}

impl Default for RigidBodyType {
    /// Defaults to Dynamic for most common use case.
    fn default() -> Self {
        Self::Dynamic
    }
}

impl RigidBodyType {
    /// Returns true if this body type is affected by gravity.
    #[inline]
    pub fn is_affected_by_gravity(self) -> bool {
        matches!(self, RigidBodyType::Dynamic)
    }

    /// Returns true if this body type is affected by forces and impulses.
    #[inline]
    pub fn is_affected_by_forces(self) -> bool {
        matches!(self, RigidBodyType::Dynamic)
    }

    /// Returns true if this body type can move.
    #[inline]
    pub fn can_move(self) -> bool {
        !matches!(self, RigidBodyType::Static)
    }

    /// Returns true if this body type responds to collisions.
    #[inline]
    pub fn responds_to_collisions(self) -> bool {
        matches!(self, RigidBodyType::Dynamic)
    }

    /// Returns the name of this body type.
    pub fn name(self) -> &'static str {
        match self {
            RigidBodyType::Dynamic => "Dynamic",
            RigidBodyType::Kinematic => "Kinematic",
            RigidBodyType::Static => "Static",
        }
    }
}

impl std::fmt::Display for RigidBodyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
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
/// - `restitution`: f32 (4 bytes)
/// - `friction`: f32 (4 bytes)
/// - `gravity_scale`: f32 (4 bytes)
/// - `flags`: u8 (1 byte) + 3 bytes padding
/// - `sleep_time`: f32 (4 bytes)
/// - Total: 64 bytes (may vary with padding)
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
///     .with_restitution(0.8)  // Bouncy
///     .with_friction(0.5);
///
/// // Kinematic platform
/// let platform = RigidBody::kinematic()
///     .with_velocity(Vec2::new(50.0, 0.0));
///
/// // Static wall
/// let wall = RigidBody::static_body();
/// ```
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

    /// Restitution (bounciness) coefficient.
    /// - 0.0 = inelastic (no bounce)
    /// - 1.0 = perfectly elastic (full bounce)
    /// - >1.0 = super-elastic (gains energy)
    pub restitution: f32,

    /// Friction coefficient.
    /// - 0.0 = frictionless (ice)
    /// - 1.0 = high friction (rubber)
    pub friction: f32,

    /// Gravity scale multiplier.
    /// - 0.0 = no gravity
    /// - 1.0 = normal gravity
    /// - 2.0 = double gravity
    pub gravity_scale: f32,

    /// Bit flags for body state (see RigidBodyFlags).
    flags: u8,

    /// Time the body has been below sleep thresholds (seconds).
    /// When this exceeds PhysicsWorld::sleep_time_threshold, the body sleeps.
    sleep_time: f32,
}

// =============================================================================
// RigidBodyFlags
// =============================================================================

/// Bit flags for rigid body state.
struct RigidBodyFlags;

impl RigidBodyFlags {
    /// Body is currently sleeping (optimization).
    const SLEEPING: u8 = 1 << 0;
    /// Body can sleep when idle.
    const CAN_SLEEP: u8 = 1 << 1;
    /// Continuous collision detection enabled.
    const CONTINUOUS_CD: u8 = 1 << 2;
    /// Fixed rotation (prevents rotation from collisions).
    const FIXED_ROTATION: u8 = 1 << 3;
}

// =============================================================================
// RigidBody Implementation
// =============================================================================

impl Default for RigidBody {
    /// Creates a default dynamic rigid body.
    fn default() -> Self {
        Self::dynamic()
    }
}

impl RigidBody {
    // =========================================================================
    // Constructors
    // =========================================================================

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
            restitution: 0.0,                 // No bounce by default
            friction: 0.5,                    // Moderate friction
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

    /// Sets the restitution (bounciness).
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.max(0.0);
        self
    }

    /// Sets the friction coefficient.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0);
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
    // Physics Operations
    // =========================================================================

    /// Applies a force to the body (affects acceleration).
    ///
    /// Force is accumulated and applied during physics integration.
    /// Only affects dynamic bodies.
    ///
    /// # Arguments
    ///
    /// * `_force` - Force vector in Newtons (kg·m/s²)
    ///
    /// # Note
    ///
    /// This is a placeholder for the public API. Forces will be accumulated
    /// in a separate `Forces` component or applied directly during physics
    /// integration in future implementation.
    pub fn apply_force(&mut self, _force: Vec2) {
        if !self.is_dynamic() {
            return;
        }
        // Forces will be accumulated in a separate Forces component
        // or applied directly during integration
        // This is a placeholder for the public API
        self.wake();
    }

    /// Applies an impulse to the body (instant velocity change).
    ///
    /// Impulse directly modifies velocity: Δv = impulse / mass
    /// Only affects dynamic bodies.
    ///
    /// # Arguments
    ///
    /// * `impulse` - Impulse vector in kg·m/s
    pub fn apply_impulse(&mut self, impulse: Vec2) {
        if !self.is_dynamic() {
            return;
        }
        self.linear_velocity = self.linear_velocity + impulse * self.inverse_mass;
        self.wake();
    }

    /// Applies an angular impulse to the body.
    ///
    /// # Arguments
    ///
    /// * `impulse` - Angular impulse in kg·m²/s
    pub fn apply_angular_impulse(&mut self, impulse: f32) {
        if !self.is_dynamic() || self.has_fixed_rotation() {
            return;
        }
        self.angular_velocity += impulse * self.inverse_inertia;
        self.wake();
    }

    /// Applies damping to velocity (called by physics system).
    ///
    /// # Arguments
    ///
    /// * `dt` - Delta time in seconds
    pub fn apply_damping(&mut self, dt: f32) {
        // Exponential damping: v *= exp(-damping * dt) ≈ v *= (1 - damping * dt)
        let linear_factor = 1.0 - self.linear_damping * dt;
        let angular_factor = 1.0 - self.angular_damping * dt;

        self.linear_velocity = self.linear_velocity * linear_factor.max(0.0);
        self.angular_velocity *= angular_factor.max(0.0);
    }

    // =========================================================================
    // Sleep Management
    // =========================================================================

    /// Puts the body to sleep (optimization).
    ///
    /// Sleeping bodies are excluded from physics updates until woken.
    pub fn sleep(&mut self) {
        self.flags |= RigidBodyFlags::SLEEPING;
        self.linear_velocity = Vec2::zero();
        self.angular_velocity = 0.0;
        self.sleep_time = 0.0;
    }

    /// Wakes the body from sleep.
    pub fn wake(&mut self) {
        self.flags &= !RigidBodyFlags::SLEEPING;
        self.sleep_time = 0.0;
    }

    /// Updates sleep time based on current motion (called by physics system).
    ///
    /// # Arguments
    ///
    /// * `dt` - Delta time in seconds
    /// * `linear_threshold` - Linear velocity threshold for sleep (pixels/s)
    /// * `angular_threshold` - Angular velocity threshold for sleep (radians/s)
    ///
    /// # Returns
    ///
    /// True if the body should sleep.
    pub fn update_sleep_time(
        &mut self,
        dt: f32,
        linear_threshold: f32,
        angular_threshold: f32,
    ) -> bool {
        if !self.can_sleep() || !self.is_dynamic() {
            self.sleep_time = 0.0;
            return false;
        }

        // Check if motion is below thresholds
        let below_threshold = self.linear_speed_squared() < linear_threshold * linear_threshold
            && self.angular_velocity.abs() < angular_threshold;

        if below_threshold {
            self.sleep_time += dt;
            true
        } else {
            self.sleep_time = 0.0;
            false
        }
    }
}

// Implement Component trait for ECS integration
impl Component for RigidBody {}

// =============================================================================
// Display Implementation
// =============================================================================

impl std::fmt::Display for RigidBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RigidBody({}, vel: {:?}, mass: {}, sleeping: {})",
            self.body_type,
            self.linear_velocity,
            self.mass,
            self.is_sleeping()
        )
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // RigidBodyType Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_type_default() {
        assert_eq!(RigidBodyType::default(), RigidBodyType::Dynamic);
    }

    #[test]
    fn test_rigidbody_type_is_affected_by_gravity() {
        assert!(RigidBodyType::Dynamic.is_affected_by_gravity());
        assert!(!RigidBodyType::Kinematic.is_affected_by_gravity());
        assert!(!RigidBodyType::Static.is_affected_by_gravity());
    }

    #[test]
    fn test_rigidbody_type_is_affected_by_forces() {
        assert!(RigidBodyType::Dynamic.is_affected_by_forces());
        assert!(!RigidBodyType::Kinematic.is_affected_by_forces());
        assert!(!RigidBodyType::Static.is_affected_by_forces());
    }

    #[test]
    fn test_rigidbody_type_can_move() {
        assert!(RigidBodyType::Dynamic.can_move());
        assert!(RigidBodyType::Kinematic.can_move());
        assert!(!RigidBodyType::Static.can_move());
    }

    #[test]
    fn test_rigidbody_type_responds_to_collisions() {
        assert!(RigidBodyType::Dynamic.responds_to_collisions());
        assert!(!RigidBodyType::Kinematic.responds_to_collisions());
        assert!(!RigidBodyType::Static.responds_to_collisions());
    }

    #[test]
    fn test_rigidbody_type_name() {
        assert_eq!(RigidBodyType::Dynamic.name(), "Dynamic");
        assert_eq!(RigidBodyType::Kinematic.name(), "Kinematic");
        assert_eq!(RigidBodyType::Static.name(), "Static");
    }

    #[test]
    fn test_rigidbody_type_display() {
        assert_eq!(format!("{}", RigidBodyType::Dynamic), "Dynamic");
        assert_eq!(format!("{}", RigidBodyType::Kinematic), "Kinematic");
        assert_eq!(format!("{}", RigidBodyType::Static), "Static");
    }

    // =========================================================================
    // RigidBody Construction Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_new_dynamic() {
        let body = RigidBody::new(RigidBodyType::Dynamic);
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.inverse_mass, 1.0);
        assert!(body.can_sleep());
        assert!(!body.is_sleeping());
    }

    #[test]
    fn test_rigidbody_new_kinematic() {
        let body = RigidBody::new(RigidBodyType::Kinematic);
        assert_eq!(body.body_type, RigidBodyType::Kinematic);
        assert_eq!(body.mass, 0.0);
        assert_eq!(body.inverse_mass, 0.0);
    }

    #[test]
    fn test_rigidbody_new_static() {
        let body = RigidBody::new(RigidBodyType::Static);
        assert_eq!(body.body_type, RigidBodyType::Static);
        assert_eq!(body.mass, 0.0);
        assert_eq!(body.inverse_mass, 0.0);
    }

    #[test]
    fn test_rigidbody_dynamic() {
        let body = RigidBody::dynamic();
        assert!(body.is_dynamic());
        assert!(!body.is_kinematic());
        assert!(!body.is_static());
    }

    #[test]
    fn test_rigidbody_kinematic() {
        let body = RigidBody::kinematic();
        assert!(!body.is_dynamic());
        assert!(body.is_kinematic());
        assert!(!body.is_static());
    }

    #[test]
    fn test_rigidbody_static_body() {
        let body = RigidBody::static_body();
        assert!(!body.is_dynamic());
        assert!(!body.is_kinematic());
        assert!(body.is_static());
    }

    #[test]
    fn test_rigidbody_default() {
        let body = RigidBody::default();
        assert!(body.is_dynamic());
        assert_eq!(body.mass, 1.0);
    }

    // =========================================================================
    // Builder Pattern Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_with_velocity() {
        let body = RigidBody::dynamic().with_velocity(Vec2::new(100.0, 50.0));
        assert_eq!(body.linear_velocity, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn test_rigidbody_with_angular_velocity() {
        let body = RigidBody::dynamic().with_angular_velocity(3.14);
        assert_eq!(body.angular_velocity, 3.14);
    }

    #[test]
    fn test_rigidbody_with_mass() {
        let body = RigidBody::dynamic().with_mass(2.0);
        assert_eq!(body.mass, 2.0);
        assert_eq!(body.inverse_mass, 0.5);
    }

    #[test]
    #[should_panic(expected = "Mass must be positive and finite")]
    fn test_rigidbody_with_mass_zero() {
        let _ = RigidBody::dynamic().with_mass(0.0);
    }

    #[test]
    #[should_panic(expected = "Mass must be positive and finite")]
    fn test_rigidbody_with_mass_negative() {
        let _ = RigidBody::dynamic().with_mass(-1.0);
    }

    #[test]
    fn test_rigidbody_with_damping() {
        let body = RigidBody::dynamic()
            .with_linear_damping(0.5)
            .with_angular_damping(0.3);
        assert_eq!(body.linear_damping, 0.5);
        assert_eq!(body.angular_damping, 0.3);
    }

    #[test]
    fn test_rigidbody_with_restitution() {
        let body = RigidBody::dynamic().with_restitution(0.8);
        assert_eq!(body.restitution, 0.8);
    }

    #[test]
    fn test_rigidbody_with_friction() {
        let body = RigidBody::dynamic().with_friction(0.7);
        assert_eq!(body.friction, 0.7);
    }

    #[test]
    fn test_rigidbody_with_gravity_scale() {
        let body = RigidBody::dynamic().with_gravity_scale(2.0);
        assert_eq!(body.gravity_scale, 2.0);
    }

    #[test]
    fn test_rigidbody_with_can_sleep() {
        let body1 = RigidBody::dynamic().with_can_sleep(true);
        assert!(body1.can_sleep());

        let body2 = RigidBody::dynamic().with_can_sleep(false);
        assert!(!body2.can_sleep());
    }

    #[test]
    fn test_rigidbody_with_continuous_cd() {
        let body1 = RigidBody::dynamic().with_continuous_cd(true);
        assert!(body1.has_continuous_cd());

        let body2 = RigidBody::dynamic().with_continuous_cd(false);
        assert!(!body2.has_continuous_cd());
    }

    #[test]
    fn test_rigidbody_with_fixed_rotation() {
        let body = RigidBody::dynamic()
            .with_angular_velocity(5.0)
            .with_fixed_rotation(true);

        assert!(body.has_fixed_rotation());
        assert_eq!(body.angular_velocity, 0.0);
        assert_eq!(body.inertia, 0.0);
        assert_eq!(body.inverse_inertia, 0.0);
    }

    #[test]
    fn test_rigidbody_builder_chaining() {
        let body = RigidBody::dynamic()
            .with_velocity(Vec2::new(100.0, 50.0))
            .with_mass(2.0)
            .with_restitution(0.8)
            .with_friction(0.5)
            .with_gravity_scale(1.5);

        assert_eq!(body.linear_velocity, Vec2::new(100.0, 50.0));
        assert_eq!(body.mass, 2.0);
        assert_eq!(body.restitution, 0.8);
        assert_eq!(body.friction, 0.5);
        assert_eq!(body.gravity_scale, 1.5);
    }

    // =========================================================================
    // Accessor Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_linear_speed() {
        let body = RigidBody::dynamic().with_velocity(Vec2::new(3.0, 4.0));
        assert_eq!(body.linear_speed(), 5.0);
    }

    #[test]
    fn test_rigidbody_linear_speed_squared() {
        let body = RigidBody::dynamic().with_velocity(Vec2::new(3.0, 4.0));
        assert_eq!(body.linear_speed_squared(), 25.0);
    }

    #[test]
    fn test_rigidbody_kinetic_energy() {
        let body = RigidBody::dynamic()
            .with_mass(2.0)
            .with_velocity(Vec2::new(3.0, 4.0));
        // KE = 0.5 * m * v² = 0.5 * 2.0 * 25.0 = 25.0
        assert_eq!(body.kinetic_energy(), 25.0);
    }

    // =========================================================================
    // Mutator Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_set_velocity() {
        let mut body = RigidBody::dynamic();
        body.set_velocity(Vec2::new(100.0, 50.0));
        assert_eq!(body.linear_velocity, Vec2::new(100.0, 50.0));
        assert!(!body.is_sleeping()); // Should wake
    }

    #[test]
    fn test_rigidbody_set_angular_velocity() {
        let mut body = RigidBody::dynamic();
        body.set_angular_velocity(3.14);
        assert_eq!(body.angular_velocity, 3.14);
        assert!(!body.is_sleeping()); // Should wake
    }

    #[test]
    fn test_rigidbody_set_mass() {
        let mut body = RigidBody::dynamic();
        body.set_mass(2.0);
        assert_eq!(body.mass, 2.0);
        assert_eq!(body.inverse_mass, 0.5);
    }

    #[test]
    #[should_panic(expected = "Cannot set mass on non-dynamic body")]
    fn test_rigidbody_set_mass_kinematic() {
        let mut body = RigidBody::kinematic();
        body.set_mass(2.0);
    }

    #[test]
    fn test_rigidbody_set_body_type() {
        let mut body = RigidBody::dynamic();
        body.set_body_type(RigidBodyType::Kinematic);

        assert!(body.is_kinematic());
        assert_eq!(body.inverse_mass, 0.0);
        assert_eq!(body.inverse_inertia, 0.0);
    }

    // =========================================================================
    // Physics Operations Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_apply_impulse() {
        let mut body = RigidBody::dynamic().with_mass(2.0);

        body.apply_impulse(Vec2::new(10.0, 0.0));
        // Δv = impulse / mass = 10.0 / 2.0 = 5.0
        assert_eq!(body.linear_velocity, Vec2::new(5.0, 0.0));
    }

    #[test]
    fn test_rigidbody_apply_impulse_kinematic() {
        let mut body = RigidBody::kinematic();
        let initial_velocity = body.linear_velocity;

        body.apply_impulse(Vec2::new(10.0, 0.0));
        // Should not affect kinematic body
        assert_eq!(body.linear_velocity, initial_velocity);
    }

    #[test]
    fn test_rigidbody_apply_angular_impulse() {
        let mut body = RigidBody::dynamic();
        body.apply_angular_impulse(5.0);
        // Δω = impulse / inertia = 5.0 / 1.0 = 5.0
        assert_eq!(body.angular_velocity, 5.0);
    }

    #[test]
    fn test_rigidbody_apply_angular_impulse_fixed_rotation() {
        let mut body = RigidBody::dynamic().with_fixed_rotation(true);

        body.apply_angular_impulse(5.0);
        // Should not affect body with fixed rotation
        assert_eq!(body.angular_velocity, 0.0);
    }

    #[test]
    fn test_rigidbody_apply_damping() {
        let mut body = RigidBody::dynamic()
            .with_velocity(Vec2::new(100.0, 0.0))
            .with_angular_velocity(10.0)
            .with_linear_damping(0.1)
            .with_angular_damping(0.1);

        body.apply_damping(0.1); // 0.1 seconds

        // Velocity should decrease
        assert!(body.linear_velocity.x < 100.0);
        assert!(body.angular_velocity < 10.0);
    }

    // =========================================================================
    // Sleep Management Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_sleep() {
        let mut body = RigidBody::dynamic()
            .with_velocity(Vec2::new(100.0, 50.0))
            .with_angular_velocity(5.0);

        body.sleep();

        assert!(body.is_sleeping());
        assert_eq!(body.linear_velocity, Vec2::zero());
        assert_eq!(body.angular_velocity, 0.0);
        assert_eq!(body.sleep_time, 0.0);
    }

    #[test]
    fn test_rigidbody_wake() {
        let mut body = RigidBody::dynamic();
        body.sleep();
        assert!(body.is_sleeping());

        body.wake();
        assert!(!body.is_sleeping());
        assert_eq!(body.sleep_time, 0.0);
    }

    #[test]
    fn test_rigidbody_update_sleep_time_below_threshold() {
        let mut body = RigidBody::dynamic().with_velocity(Vec2::new(1.0, 1.0));

        let should_sleep = body.update_sleep_time(0.1, 5.0, 0.1);
        assert!(should_sleep);
        assert!(body.sleep_time > 0.0);
    }

    #[test]
    fn test_rigidbody_update_sleep_time_above_threshold() {
        let mut body = RigidBody::dynamic().with_velocity(Vec2::new(100.0, 0.0));

        let should_sleep = body.update_sleep_time(0.1, 5.0, 0.1);
        assert!(!should_sleep);
        assert_eq!(body.sleep_time, 0.0);
    }

    #[test]
    fn test_rigidbody_update_sleep_time_cannot_sleep() {
        let mut body = RigidBody::dynamic()
            .with_can_sleep(false)
            .with_velocity(Vec2::new(1.0, 1.0));

        let should_sleep = body.update_sleep_time(0.1, 5.0, 0.1);
        assert!(!should_sleep);
        assert_eq!(body.sleep_time, 0.0);
    }

    // =========================================================================
    // Component and Display Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_is_component() {
        fn requires_component<T: Component>() {}
        requires_component::<RigidBody>();
    }

    #[test]
    fn test_rigidbody_display() {
        let body = RigidBody::dynamic().with_velocity(Vec2::new(100.0, 50.0));

        let display = format!("{}", body);
        assert!(display.contains("RigidBody"));
        assert!(display.contains("Dynamic"));
        assert!(display.contains("vel"));
        assert!(display.contains("mass"));
    }

    #[test]
    fn test_rigidbody_debug() {
        let body = RigidBody::dynamic();
        let debug = format!("{:?}", body);
        assert!(debug.contains("RigidBody"));
    }

    #[test]
    fn test_rigidbody_clone() {
        let body1 = RigidBody::dynamic()
            .with_velocity(Vec2::new(100.0, 50.0))
            .with_mass(2.0);

        let body2 = body1;
        assert_eq!(body1.linear_velocity, body2.linear_velocity);
        assert_eq!(body1.mass, body2.mass);
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    #[test]
    fn test_rigidbody_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<RigidBody>();
    }

    #[test]
    fn test_rigidbody_is_sync() {
        fn requires_sync<T: Sync>() {}
        requires_sync::<RigidBody>();
    }
}
