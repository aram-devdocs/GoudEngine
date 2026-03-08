//! Accessors, mutators, simulation control, and utility methods for [`PhysicsWorld`].

use std::time::Duration;

use super::resource::PhysicsWorld;
use crate::core::math::Vec2;

impl PhysicsWorld {
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

    /// Returns the maximum number of physics steps allowed per frame.
    #[inline]
    pub fn max_steps_per_frame(&self) -> u32 {
        self.max_steps_per_frame
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

        // Clamp accumulator to prevent spiral of death: if the frame was very
        // slow, limit the number of catch-up steps to max_steps_per_frame.
        self.time_accumulator = self
            .time_accumulator
            .min(self.timestep * self.max_steps_per_frame as f32);

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
