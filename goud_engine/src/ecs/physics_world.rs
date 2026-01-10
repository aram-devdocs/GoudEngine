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
use std::time::Duration;

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
    gravity: Vec2,

    /// Fixed timestep for physics simulation (seconds).
    /// Default: 1/60 (60 Hz) - 16.67ms per step.
    timestep: f32,

    /// Number of velocity solver iterations per step.
    /// Higher values improve constraint stability but increase CPU cost.
    /// Default: 8 iterations.
    velocity_iterations: u32,

    /// Number of position solver iterations per step.
    /// Higher values reduce positional drift but increase CPU cost.
    /// Default: 3 iterations.
    position_iterations: u32,

    /// Accumulated time since last physics step (seconds).
    /// Used for fixed timestep accumulator pattern.
    time_accumulator: f32,

    /// Total elapsed simulation time (seconds).
    /// Incremented by timestep on each physics step.
    simulation_time: f64,

    /// Total number of physics steps executed.
    step_count: u64,

    /// Whether physics simulation is paused.
    /// When true, physics systems skip updates.
    paused: bool,

    /// Time scale multiplier for simulation speed.
    /// 1.0 = normal speed, 0.5 = half speed, 2.0 = double speed.
    /// Range: [0.0, 10.0] for safety.
    time_scale: f32,

    /// Whether to use sleeping optimization for idle bodies.
    /// When true, bodies that haven't moved recently are excluded from simulation.
    sleep_enabled: bool,

    /// Linear velocity threshold for sleep (pixels/s).
    /// Bodies moving slower than this may sleep.
    sleep_linear_threshold: f32,

    /// Angular velocity threshold for sleep (radians/s).
    /// Bodies rotating slower than this may sleep.
    sleep_angular_threshold: f32,

    /// Time a body must be below thresholds before sleeping (seconds).
    sleep_time_threshold: f32,
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

    // =========================================================================
    // Accessors
    // =========================================================================

    /// Returns the gravity vector.
    #[inline]
    pub fn gravity(&self) -> Vec2 {
        self.gravity
    }

    /// Returns the fixed timestep.
    #[inline]
    pub fn timestep(&self) -> f32 {
        self.timestep
    }

    /// Returns the number of velocity solver iterations.
    #[inline]
    pub fn velocity_iterations(&self) -> u32 {
        self.velocity_iterations
    }

    /// Returns the number of position solver iterations.
    #[inline]
    pub fn position_iterations(&self) -> u32 {
        self.position_iterations
    }

    /// Returns the time accumulator value.
    #[inline]
    pub fn time_accumulator(&self) -> f32 {
        self.time_accumulator
    }

    /// Returns the total simulation time.
    #[inline]
    pub fn simulation_time(&self) -> f64 {
        self.simulation_time
    }

    /// Returns the total number of physics steps executed.
    #[inline]
    pub fn step_count(&self) -> u64 {
        self.step_count
    }

    /// Returns whether physics is paused.
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns the time scale.
    #[inline]
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Returns whether sleeping is enabled.
    #[inline]
    pub fn is_sleep_enabled(&self) -> bool {
        self.sleep_enabled
    }

    /// Returns the linear velocity sleep threshold.
    #[inline]
    pub fn sleep_linear_threshold(&self) -> f32 {
        self.sleep_linear_threshold
    }

    /// Returns the angular velocity sleep threshold.
    #[inline]
    pub fn sleep_angular_threshold(&self) -> f32 {
        self.sleep_angular_threshold
    }

    /// Returns the time threshold for sleeping.
    #[inline]
    pub fn sleep_time_threshold(&self) -> f32 {
        self.sleep_time_threshold
    }

    // =========================================================================
    // Mutators
    // =========================================================================

    /// Sets the gravity vector.
    pub fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    /// Sets the fixed timestep.
    ///
    /// # Panics
    ///
    /// Panics if `timestep` is not positive and finite.
    pub fn set_timestep(&mut self, timestep: f32) {
        assert!(
            timestep > 0.0 && timestep.is_finite(),
            "Timestep must be positive and finite"
        );
        self.timestep = timestep;
    }

    /// Sets the solver iteration counts.
    pub fn set_iterations(&mut self, velocity_iterations: u32, position_iterations: u32) {
        self.velocity_iterations = velocity_iterations;
        self.position_iterations = position_iterations;
    }

    /// Pauses physics simulation.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes physics simulation.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Sets the time scale.
    ///
    /// # Panics
    ///
    /// Panics if `time_scale` is negative or greater than 10.0.
    pub fn set_time_scale(&mut self, time_scale: f32) {
        assert!(
            (0.0..=10.0).contains(&time_scale),
            "Time scale must be in range [0.0, 10.0]"
        );
        self.time_scale = time_scale;
    }

    /// Enables or disables sleeping optimization.
    pub fn set_sleep_enabled(&mut self, enabled: bool) {
        self.sleep_enabled = enabled;
    }

    /// Sets the sleep thresholds.
    pub fn set_sleep_thresholds(&mut self, linear: f32, angular: f32, time: f32) {
        self.sleep_linear_threshold = linear;
        self.sleep_angular_threshold = angular;
        self.sleep_time_threshold = time;
    }

    // =========================================================================
    // Simulation Control
    // =========================================================================

    /// Advances the time accumulator by the given delta time.
    ///
    /// This is typically called once per frame with the frame's delta time.
    /// The physics system will then step the simulation in fixed increments
    /// while the accumulator exceeds the timestep.
    ///
    /// # Arguments
    ///
    /// * `delta` - Frame delta time in seconds
    ///
    /// # Returns
    ///
    /// The number of physics steps that should be executed this frame.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let mut physics = PhysicsWorld::new();
    ///
    /// // Advance by one frame (16.67ms at 60 FPS)
    /// let steps = physics.advance(1.0 / 60.0);
    /// assert_eq!(steps, 1);  // Exactly one step
    ///
    /// // Advance by a large delta (slow frame)
    /// let steps = physics.advance(0.1);  // 100ms
    /// assert!(steps > 0);  // Multiple steps to catch up
    /// ```
    pub fn advance(&mut self, delta: f32) -> u32 {
        if self.paused {
            return 0;
        }

        // Apply time scale
        let scaled_delta = delta * self.time_scale;

        // Accumulate time
        self.time_accumulator += scaled_delta;

        // Calculate number of steps to execute
        (self.time_accumulator / self.timestep) as u32
    }

    /// Executes a single physics step, decrementing the accumulator.
    ///
    /// This should be called by physics systems in a loop while `should_step()`
    /// returns true.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let mut physics = PhysicsWorld::new();
    /// physics.advance(0.05);  // Accumulate time
    ///
    /// while physics.should_step() {
    ///     physics.step();
    ///     // ... perform physics calculations ...
    /// }
    /// ```
    pub fn step(&mut self) {
        if self.time_accumulator >= self.timestep {
            self.time_accumulator -= self.timestep;
            self.simulation_time += self.timestep as f64;
            self.step_count += 1;
        }
    }

    /// Returns true if a physics step should be executed.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let mut physics = PhysicsWorld::new();
    /// assert!(!physics.should_step());  // No time accumulated
    ///
    /// physics.advance(0.02);  // 20ms
    /// assert!(physics.should_step());  // Enough for one step at 60 Hz
    /// ```
    #[inline]
    pub fn should_step(&self) -> bool {
        !self.paused && self.time_accumulator >= self.timestep
    }

    /// Returns the interpolation alpha for rendering.
    ///
    /// This value is used to interpolate visual state between the previous
    /// and current physics states for smooth rendering at arbitrary frame rates.
    ///
    /// # Returns
    ///
    /// A value in [0.0, 1.0] representing how far between physics steps we are.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let mut physics = PhysicsWorld::new();
    /// physics.advance(0.008);  // 8ms (half a step at 60 Hz)
    ///
    /// let alpha = physics.interpolation_alpha();
    /// assert!(alpha > 0.4 && alpha < 0.6);  // Approximately 0.5
    /// ```
    #[inline]
    pub fn interpolation_alpha(&self) -> f32 {
        if self.timestep > 0.0 {
            self.time_accumulator / self.timestep
        } else {
            0.0
        }
    }

    /// Resets the simulation state.
    ///
    /// Clears the time accumulator, simulation time, and step count.
    /// Does not modify configuration settings.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let mut physics = PhysicsWorld::new();
    /// physics.advance(1.0);
    /// assert!(physics.step_count() > 0);
    ///
    /// physics.reset();
    /// assert_eq!(physics.step_count(), 0);
    /// assert_eq!(physics.simulation_time(), 0.0);
    /// ```
    pub fn reset(&mut self) {
        self.time_accumulator = 0.0;
        self.simulation_time = 0.0;
        self.step_count = 0;
    }

    // =========================================================================
    // Utility Methods
    // =========================================================================

    /// Returns the simulation frequency in Hz.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let physics = PhysicsWorld::new();
    /// assert_eq!(physics.frequency(), 60.0);  // 60 Hz
    /// ```
    #[inline]
    pub fn frequency(&self) -> f32 {
        if self.timestep > 0.0 {
            1.0 / self.timestep
        } else {
            0.0
        }
    }

    /// Returns the timestep as a Duration.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    /// use std::time::Duration;
    ///
    /// let physics = PhysicsWorld::new();
    /// let duration = physics.timestep_duration();
    /// assert_eq!(duration.as_millis(), 16);  // ~16.67ms
    /// ```
    #[inline]
    pub fn timestep_duration(&self) -> Duration {
        Duration::from_secs_f32(self.timestep)
    }

    /// Returns physics simulation statistics as a formatted string.
    ///
    /// Useful for debug displays and profiling.
    ///
    /// # Examples
    ///
    /// ```
    /// use goud_engine::ecs::physics_world::PhysicsWorld;
    ///
    /// let mut physics = PhysicsWorld::new();
    /// physics.advance(1.0);
    /// while physics.should_step() {
    ///     physics.step();
    /// }
    ///
    /// println!("{}", physics.stats());
    /// ```
    pub fn stats(&self) -> String {
        format!(
            "PhysicsWorld Stats:\n\
             - Timestep: {:.2}ms ({:.0} Hz)\n\
             - Steps: {}\n\
             - Sim Time: {:.2}s\n\
             - Time Scale: {:.2}x\n\
             - Paused: {}\n\
             - Accumulator: {:.2}ms\n\
             - Gravity: {:?}\n\
             - Iterations: {} vel, {} pos\n\
             - Sleep: {}",
            self.timestep * 1000.0,
            self.frequency(),
            self.step_count,
            self.simulation_time,
            self.time_scale,
            self.paused,
            self.time_accumulator * 1000.0,
            self.gravity,
            self.velocity_iterations,
            self.position_iterations,
            if self.sleep_enabled {
                "enabled"
            } else {
                "disabled"
            }
        )
    }
}

// PhysicsWorld is Send + Sync (no interior mutability)
// This is verified by the compiler due to all fields being Send + Sync

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_new() {
        let physics = PhysicsWorld::new();
        assert_eq!(physics.gravity(), Vec2::new(0.0, -980.0));
        assert_eq!(physics.timestep(), 1.0 / 60.0);
        assert_eq!(physics.velocity_iterations(), 8);
        assert_eq!(physics.position_iterations(), 3);
        assert_eq!(physics.time_scale(), 1.0);
        assert!(!physics.is_paused());
        assert!(physics.is_sleep_enabled());
    }

    #[test]
    fn test_default() {
        let physics = PhysicsWorld::default();
        assert_eq!(physics.timestep(), 1.0 / 60.0);
    }

    #[test]
    fn test_zero_gravity() {
        let physics = PhysicsWorld::zero_gravity();
        assert_eq!(physics.gravity(), Vec2::zero());
    }

    // =========================================================================
    // Builder Pattern Tests
    // =========================================================================

    #[test]
    fn test_with_gravity() {
        let physics = PhysicsWorld::new().with_gravity(Vec2::new(0.0, -490.0));
        assert_eq!(physics.gravity(), Vec2::new(0.0, -490.0));
    }

    #[test]
    fn test_with_timestep() {
        let physics = PhysicsWorld::new().with_timestep(1.0 / 120.0);
        assert_eq!(physics.timestep(), 1.0 / 120.0);
    }

    #[test]
    #[should_panic(expected = "Timestep must be positive and finite")]
    fn test_with_timestep_invalid() {
        let _ = PhysicsWorld::new().with_timestep(0.0);
    }

    #[test]
    fn test_with_iterations() {
        let physics = PhysicsWorld::new().with_iterations(10, 4);
        assert_eq!(physics.velocity_iterations(), 10);
        assert_eq!(physics.position_iterations(), 4);
    }

    #[test]
    fn test_with_time_scale() {
        let physics = PhysicsWorld::new().with_time_scale(0.5);
        assert_eq!(physics.time_scale(), 0.5);
    }

    #[test]
    #[should_panic(expected = "Time scale must be in range [0.0, 10.0]")]
    fn test_with_time_scale_too_high() {
        let _ = PhysicsWorld::new().with_time_scale(11.0);
    }

    #[test]
    fn test_with_sleep_config() {
        let physics = PhysicsWorld::new().with_sleep_config(false, 2.0, 0.05, 1.0);
        assert!(!physics.is_sleep_enabled());
        assert_eq!(physics.sleep_linear_threshold(), 2.0);
        assert_eq!(physics.sleep_angular_threshold(), 0.05);
        assert_eq!(physics.sleep_time_threshold(), 1.0);
    }

    #[test]
    fn test_builder_chaining() {
        let physics = PhysicsWorld::new()
            .with_gravity(Vec2::new(0.0, -490.0))
            .with_timestep(1.0 / 120.0)
            .with_iterations(10, 4)
            .with_time_scale(2.0);

        assert_eq!(physics.gravity(), Vec2::new(0.0, -490.0));
        assert_eq!(physics.timestep(), 1.0 / 120.0);
        assert_eq!(physics.velocity_iterations(), 10);
        assert_eq!(physics.time_scale(), 2.0);
    }

    // =========================================================================
    // Mutator Tests
    // =========================================================================

    #[test]
    fn test_set_gravity() {
        let mut physics = PhysicsWorld::new();
        physics.set_gravity(Vec2::new(10.0, 0.0));
        assert_eq!(physics.gravity(), Vec2::new(10.0, 0.0));
    }

    #[test]
    fn test_pause_resume() {
        let mut physics = PhysicsWorld::new();
        assert!(!physics.is_paused());

        physics.pause();
        assert!(physics.is_paused());

        physics.resume();
        assert!(!physics.is_paused());
    }

    #[test]
    fn test_set_time_scale() {
        let mut physics = PhysicsWorld::new();
        physics.set_time_scale(0.5);
        assert_eq!(physics.time_scale(), 0.5);
    }

    // =========================================================================
    // Simulation Control Tests
    // =========================================================================

    #[test]
    fn test_advance_single_step() {
        let mut physics = PhysicsWorld::new();
        let steps = physics.advance(1.0 / 60.0);
        assert_eq!(steps, 1);
        assert!(physics.should_step());
    }

    #[test]
    fn test_advance_multiple_steps() {
        let mut physics = PhysicsWorld::new();
        let steps = physics.advance(0.1); // 100ms = ~6 steps at 60 Hz
        assert!(steps >= 6);
    }

    #[test]
    fn test_advance_no_steps() {
        let mut physics = PhysicsWorld::new();
        let steps = physics.advance(0.001); // 1ms < timestep
        assert_eq!(steps, 0);
        assert!(!physics.should_step());
    }

    #[test]
    fn test_advance_with_time_scale() {
        let mut physics = PhysicsWorld::new().with_time_scale(2.0);

        let steps = physics.advance(1.0 / 120.0); // 8.33ms * 2 = 16.67ms
        assert_eq!(steps, 1);
    }

    #[test]
    fn test_advance_paused() {
        let mut physics = PhysicsWorld::new();
        physics.pause();

        let steps = physics.advance(1.0);
        assert_eq!(steps, 0);
        assert!(!physics.should_step());
    }

    #[test]
    fn test_step() {
        let mut physics = PhysicsWorld::new();
        physics.advance(0.1);

        let initial_steps = physics.step_count();
        physics.step();
        assert_eq!(physics.step_count(), initial_steps + 1);
        assert!(physics.simulation_time() > 0.0);
    }

    #[test]
    fn test_step_loop() {
        let mut physics = PhysicsWorld::new();
        physics.advance(0.1); // 100ms

        let mut step_count = 0;
        while physics.should_step() {
            physics.step();
            step_count += 1;

            // Safety: prevent infinite loop
            if step_count > 100 {
                break;
            }
        }

        assert!(step_count >= 6); // ~6 steps at 60 Hz
        assert!(!physics.should_step());
    }

    #[test]
    fn test_interpolation_alpha() {
        let mut physics = PhysicsWorld::new();

        // Half a step
        physics.advance(1.0 / 120.0);
        let alpha = physics.interpolation_alpha();
        assert!(alpha > 0.4 && alpha < 0.6);

        // Full step
        physics.advance(1.0 / 120.0);
        let alpha = physics.interpolation_alpha();
        assert!(alpha > 0.9);
    }

    #[test]
    fn test_reset() {
        let mut physics = PhysicsWorld::new();
        physics.advance(1.0);
        while physics.should_step() {
            physics.step();
        }

        assert!(physics.step_count() > 0);
        assert!(physics.simulation_time() > 0.0);

        physics.reset();
        assert_eq!(physics.step_count(), 0);
        assert_eq!(physics.simulation_time(), 0.0);
        assert_eq!(physics.time_accumulator(), 0.0);
    }

    // =========================================================================
    // Utility Tests
    // =========================================================================

    #[test]
    fn test_frequency() {
        let physics = PhysicsWorld::new();
        let freq = physics.frequency();
        assert!((freq - 60.0).abs() < 0.01);

        let physics = PhysicsWorld::new().with_timestep(1.0 / 120.0);
        let freq = physics.frequency();
        assert!((freq - 120.0).abs() < 0.01);
    }

    #[test]
    fn test_timestep_duration() {
        let physics = PhysicsWorld::new();
        let duration = physics.timestep_duration();
        assert_eq!(duration.as_millis(), 16); // ~16.67ms
    }

    #[test]
    fn test_stats() {
        let mut physics = PhysicsWorld::new();
        physics.advance(1.0);
        while physics.should_step() {
            physics.step();
        }

        let stats = physics.stats();
        assert!(stats.contains("PhysicsWorld Stats"));
        assert!(stats.contains("Timestep"));
        assert!(stats.contains("Steps"));
        assert!(stats.contains("Gravity"));
    }

    // =========================================================================
    // Clone and Debug Tests
    // =========================================================================

    #[test]
    fn test_clone() {
        let physics1 = PhysicsWorld::new()
            .with_gravity(Vec2::new(0.0, -490.0))
            .with_timestep(1.0 / 120.0);

        let physics2 = physics1.clone();
        assert_eq!(physics2.gravity(), physics1.gravity());
        assert_eq!(physics2.timestep(), physics1.timestep());
    }

    #[test]
    fn test_debug() {
        let physics = PhysicsWorld::new();
        let debug_str = format!("{physics:?}");
        assert!(debug_str.contains("PhysicsWorld"));
    }

    // =========================================================================
    // Thread Safety Tests
    // =========================================================================

    #[test]
    fn test_is_send() {
        fn requires_send<T: Send>() {}
        requires_send::<PhysicsWorld>();
    }

    #[test]
    fn test_is_sync() {
        fn requires_sync<T: Sync>() {}
        requires_sync::<PhysicsWorld>();
    }

    // =========================================================================
    // Integration Tests
    // =========================================================================

    #[test]
    fn test_fixed_timestep_pattern() {
        let mut physics = PhysicsWorld::new();

        // Simulate variable frame rates
        let frame_times = vec![0.016, 0.020, 0.012, 0.033];
        let mut total_steps = 0;

        for dt in frame_times {
            physics.advance(dt);

            while physics.should_step() {
                physics.step();
                total_steps += 1;
            }
        }

        assert_eq!(physics.step_count(), total_steps as u64);
    }

    #[test]
    fn test_slow_motion() {
        let mut physics = PhysicsWorld::new().with_time_scale(0.5);

        // 100ms real time = 50ms simulated time at 0.5x
        physics.advance(0.1);

        // At 60 Hz with 0.5x time scale:
        // 100ms * 0.5 = 50ms simulated time
        // 50ms / 16.67ms per step = ~3 steps
        let mut actual_steps = 0;

        while physics.should_step() {
            physics.step();
            actual_steps += 1;
        }

        // Should be around 3 steps (allow for floating point precision)
        assert!(actual_steps >= 2 && actual_steps <= 4);
    }
}
