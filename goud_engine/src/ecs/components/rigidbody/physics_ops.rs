//! Physics operations, sleep management, and trait impls for [`RigidBody`].

use crate::core::math::Vec2;
use crate::ecs::Component;

use super::body::RigidBody;

// =============================================================================
// Physics Operations
// =============================================================================

impl RigidBody {
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
        self.set_sleeping(true);
        self.linear_velocity = Vec2::zero();
        self.angular_velocity = 0.0;
        self.reset_sleep_time();
    }

    /// Wakes the body from sleep.
    pub fn wake(&mut self) {
        self.set_sleeping(false);
        self.reset_sleep_time();
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
            self.reset_sleep_time();
            return false;
        }

        // Check if motion is below thresholds
        let below_threshold = self.linear_speed_squared() < linear_threshold * linear_threshold
            && self.angular_velocity.abs() < angular_threshold;

        if below_threshold {
            self.increment_sleep_time(dt);
            true
        } else {
            self.reset_sleep_time();
            false
        }
    }
}

// =============================================================================
// Trait Implementations
// =============================================================================

/// ECS component trait implementation.
impl Component for RigidBody {}

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
