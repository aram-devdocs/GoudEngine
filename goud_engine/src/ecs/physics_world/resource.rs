//! Physics world resource for managing physics simulation.
//!
//! The `PhysicsWorld` resource is the central coordinator for all physics operations
//! in the game engine. It manages the physics simulation state, configuration, and
//! provides systems with access to physics parameters.
//!
//! # Architecture
//!
//! The physics system follows a modular design:
//!
//! - **PhysicsWorld**: Global resource managing simulation state
//! - **RigidBody**: Component for dynamic/kinematic/static physics objects
//! - **Collider**: Component defining collision shapes
//! - **Physics Systems**: Systems that update physics state each frame
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::{World, physics_world::PhysicsWorld};
//! use goud_engine::core::math::Vec2;
//!
//! let mut world = World::new();
//!
//! // Create physics world with custom gravity
//! let physics = PhysicsWorld::new()
//!     .with_gravity(Vec2::new(0.0, -980.0))  // 980 pixels/s^2 downward
//!     .with_timestep(1.0 / 60.0)  // 60 Hz fixed timestep
//!     .with_iterations(8, 3);  // 8 velocity, 3 position iterations
//!
//! world.insert_resource(physics);
//!
//! // Access in systems
//! // fn physics_system(physics: Res<PhysicsWorld>) { ... }
//! ```
//!
//! # Simulation Loop
//!
//! Physics simulation uses a fixed timestep accumulator pattern:
//!
//! 1. Accumulate frame delta time
//! 2. Step simulation in fixed increments while accumulator >= timestep
//! 3. Interpolate visual state for smooth rendering
//!
//! This ensures deterministic physics regardless of frame rate.
//!
//! # Collision Detection
//!
//! The physics world uses a two-phase collision detection approach:
//!
//! - **Broad Phase**: Spatial partitioning (spatial hash or AABB tree)
//! - **Narrow Phase**: Precise collision tests (SAT, GJK, or simple shapes)
//!
//! # Thread Safety
//!
//! PhysicsWorld is `Send + Sync` and can be accessed from parallel systems.
//! Mutable access requires `ResMut<PhysicsWorld>`.

use crate::core::math::Vec2;

// =============================================================================
// PhysicsWorld Resource
// =============================================================================

/// Global physics simulation resource.
///
/// `PhysicsWorld` manages the physics simulation state and configuration.
/// It stores global physics parameters like gravity, timestep, and solver
/// iterations.
///
/// # Examples
///
/// ```
/// use goud_engine::ecs::physics_world::PhysicsWorld;
/// use goud_engine::core::math::Vec2;
///
/// // Create with defaults
/// let mut physics = PhysicsWorld::new();
///
/// // Configure with builder pattern
/// let physics = PhysicsWorld::new()
///     .with_gravity(Vec2::new(0.0, -980.0))
///     .with_timestep(1.0 / 120.0)  // 120 Hz
///     .with_iterations(10, 4);
///
/// // Access configuration
/// println!("Gravity: {:?}", physics.gravity());
/// println!("Timestep: {}", physics.timestep());
/// ```
#[derive(Clone, Debug)]
pub struct PhysicsWorld {
    /// Gravity acceleration vector (pixels/s^2).
    /// Default: (0.0, -980.0) - downward at 980 pixels/s^2 (similar to 9.8 m/s^2).
    pub(super) gravity: Vec2,

    /// Fixed timestep for physics simulation (seconds).
    /// Default: 1/60 (60 Hz) - 16.67ms per step.
    pub(super) timestep: f32,

    /// Number of velocity solver iterations per step.
    /// Higher values improve constraint stability but increase CPU cost.
    /// Default: 8 iterations.
    pub(super) velocity_iterations: u32,

    /// Number of position solver iterations per step.
    /// Higher values reduce positional drift but increase CPU cost.
    /// Default: 3 iterations.
    pub(super) position_iterations: u32,

    /// Accumulated time since last physics step (seconds).
    /// Used for fixed timestep accumulator pattern.
    pub(super) time_accumulator: f32,

    /// Total elapsed simulation time (seconds).
    /// Incremented by timestep on each physics step.
    pub(super) simulation_time: f64,

    /// Total number of physics steps executed.
    pub(super) step_count: u64,

    /// Whether physics simulation is paused.
    /// When true, physics systems skip updates.
    pub(super) paused: bool,

    /// Time scale multiplier for simulation speed.
    /// 1.0 = normal speed, 0.5 = half speed, 2.0 = double speed.
    /// Range: [0.0, 10.0] for safety.
    pub(super) time_scale: f32,

    /// Whether to use sleeping optimization for idle bodies.
    /// When true, bodies that haven't moved recently are excluded from simulation.
    pub(super) sleep_enabled: bool,

    /// Linear velocity threshold for sleep (pixels/s).
    /// Bodies moving slower than this may sleep.
    pub(super) sleep_linear_threshold: f32,

    /// Angular velocity threshold for sleep (radians/s).
    /// Bodies rotating slower than this may sleep.
    pub(super) sleep_angular_threshold: f32,

    /// Time a body must be below thresholds before sleeping (seconds).
    pub(super) sleep_time_threshold: f32,

    /// Maximum number of physics steps allowed per frame.
    /// Prevents the "spiral of death" where a slow frame causes many physics
    /// steps, which makes the next frame even slower, and so on.
    /// Default: 8 steps per frame.
    pub(super) max_steps_per_frame: u32,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsWorld {
    // =========================================================================
    // Construction
    // =========================================================================

    /// Creates a new physics world with default settings.
    ///
    /// Default configuration:
    /// - Gravity: (0.0, -980.0) pixels/s^2 (downward)
    /// - Timestep: 1/60 seconds (60 Hz)
    /// - Velocity iterations: 8
    /// - Position iterations: 3
    /// - Time scale: 1.0 (normal speed)
    /// - Sleep enabled: true
    /// - Sleep thresholds: 5.0 px/s linear, 0.1 rad/s angular
    /// - Sleep time: 0.5 seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let physics = PhysicsWorld::new();
    /// assert_eq!(physics.timestep(), 1.0 / 60.0);
    /// assert_eq!(physics.velocity_iterations(), 8);
    /// ```
    pub fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, -980.0),
            timestep: 1.0 / 60.0,
            velocity_iterations: 8,
            position_iterations: 3,
            time_accumulator: 0.0,
            simulation_time: 0.0,
            step_count: 0,
            paused: false,
            time_scale: 1.0,
            sleep_enabled: true,
            sleep_linear_threshold: 5.0,
            sleep_angular_threshold: 0.1,
            sleep_time_threshold: 0.5,
            max_steps_per_frame: 8,
        }
    }

    /// Creates a physics world with no gravity (space simulation).
    ///
    /// Useful for top-down games or space games where gravity doesn't apply.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let physics = PhysicsWorld::zero_gravity();
    /// assert_eq!(physics.gravity(), Vec2::zero());
    /// ```
    pub fn zero_gravity() -> Self {
        Self {
            gravity: Vec2::zero(),
            ..Self::new()
        }
    }

    // =========================================================================
    // Builder Pattern Configuration
    // =========================================================================

    /// Sets the gravity vector.
    ///
    /// # Arguments
    ///
    /// * `gravity` - Acceleration vector in pixels/s^2
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    /// use goud_engine::core::math::Vec2;
    ///
    /// let physics = PhysicsWorld::new()
    ///     .with_gravity(Vec2::new(0.0, -490.0));  // Half earth gravity
    /// ```
    pub fn with_gravity(mut self, gravity: Vec2) -> Self {
        self.gravity = gravity;
        self
    }

    /// Sets the fixed timestep for simulation.
    ///
    /// # Arguments
    ///
    /// * `timestep` - Time in seconds per physics step (e.g., 1.0/60.0 for 60 Hz)
    ///
    /// # Panics
    ///
    /// Panics if `timestep` is not positive and finite.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let physics = PhysicsWorld::new()
    ///     .with_timestep(1.0 / 120.0);  // 120 Hz for fast-paced games
    /// ```
    pub fn with_timestep(mut self, timestep: f32) -> Self {
        assert!(
            timestep > 0.0 && timestep.is_finite(),
            "Timestep must be positive and finite"
        );
        self.timestep = timestep;
        self
    }

    /// Sets the solver iteration counts.
    ///
    /// # Arguments
    ///
    /// * `velocity_iterations` - Velocity constraint solver iterations (typically 6-10)
    /// * `position_iterations` - Position constraint solver iterations (typically 2-4)
    ///
    /// Higher iteration counts improve stability and accuracy but increase CPU cost.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// // High-precision simulation
    /// let physics = PhysicsWorld::new()
    ///     .with_iterations(10, 4);
    ///
    /// // Fast but less stable
    /// let physics = PhysicsWorld::new()
    ///     .with_iterations(4, 2);
    /// ```
    pub fn with_iterations(mut self, velocity_iterations: u32, position_iterations: u32) -> Self {
        self.velocity_iterations = velocity_iterations;
        self.position_iterations = position_iterations;
        self
    }

    /// Sets the time scale for simulation speed.
    ///
    /// # Arguments
    ///
    /// * `time_scale` - Multiplier for simulation speed (0.0 to 10.0)
    ///   - 1.0 = normal speed
    ///   - 0.5 = half speed (slow motion)
    ///   - 2.0 = double speed (fast forward)
    ///
    /// # Panics
    ///
    /// Panics if `time_scale` is negative or greater than 10.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let physics = PhysicsWorld::new()
    ///     .with_time_scale(0.5);  // Slow motion
    /// ```
    pub fn with_time_scale(mut self, time_scale: f32) -> Self {
        assert!(
            (0.0..=10.0).contains(&time_scale),
            "Time scale must be in range [0.0, 10.0]"
        );
        self.time_scale = time_scale;
        self
    }

    /// Configures sleeping optimization.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable sleeping
    /// * `linear_threshold` - Linear velocity threshold (pixels/s)
    /// * `angular_threshold` - Angular velocity threshold (radians/s)
    /// * `time_threshold` - Time below thresholds before sleeping (seconds)
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let physics = PhysicsWorld::new()
    ///     .with_sleep_config(true, 2.0, 0.05, 1.0);  // More aggressive sleeping
    /// ```
    pub fn with_sleep_config(
        mut self,
        enabled: bool,
        linear_threshold: f32,
        angular_threshold: f32,
        time_threshold: f32,
    ) -> Self {
        self.sleep_enabled = enabled;
        self.sleep_linear_threshold = linear_threshold;
        self.sleep_angular_threshold = angular_threshold;
        self.sleep_time_threshold = time_threshold;
        self
    }

    /// Sets the maximum number of physics steps allowed per frame.
    ///
    /// This prevents the "spiral of death" where a slow frame causes many
    /// physics steps, making the next frame even slower, and so on.
    ///
    /// # Arguments
    ///
    /// * `max_steps` - Maximum steps per frame (must be >= 1)
    ///
    /// # Panics
    ///
    /// Panics if `max_steps` is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let physics = PhysicsWorld::new()
    ///     .with_max_steps_per_frame(4);  // Limit to 4 steps per frame
    /// assert_eq!(physics.max_steps_per_frame(), 4);
    /// ```
    pub fn with_max_steps_per_frame(mut self, max_steps: u32) -> Self {
        assert!(max_steps >= 1, "max_steps_per_frame must be at least 1");
        self.max_steps_per_frame = max_steps;
        self
    }
}
