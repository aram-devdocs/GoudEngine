use crate::constants::*;

// =============================================================================
// Movement (mirrors Movement.cs)
// =============================================================================

/// Physics-based vertical movement: gravity, jumping, rotation.
pub struct Movement {
    pub velocity: f32,
    pub rotation: f32,
    gravity: f32,
    jump_strength: f32,
    jump_cooldown_timer: f32,
}

impl Movement {
    pub fn new(gravity: f32, jump_strength: f32) -> Self {
        Self {
            velocity: 0.0,
            rotation: 0.0,
            gravity,
            jump_strength,
            jump_cooldown_timer: 0.0,
        }
    }

    /// Applies gravity to velocity and ticks the jump cooldown.
    pub fn apply_gravity(&mut self, dt: f32) {
        self.velocity += self.gravity * dt * TARGET_FPS;
        self.jump_cooldown_timer = (self.jump_cooldown_timer - dt).max(0.0);
    }

    /// Attempts a jump if the cooldown has elapsed.
    pub fn try_jump(&mut self) {
        if self.jump_cooldown_timer <= 0.0 {
            self.velocity = self.jump_strength * TARGET_FPS;
            self.jump_cooldown_timer = JUMP_COOLDOWN;
        }
    }

    /// Updates the Y position and smoothly adjusts rotation.
    pub fn update_position(&mut self, position_y: &mut f32, dt: f32) {
        *position_y += self.velocity * dt;
        let target_rotation = (self.velocity * 3.0).clamp(-45.0, 45.0);
        self.rotation += (target_rotation - self.rotation) * ROTATION_SMOOTHING;
    }
}

// =============================================================================
// BirdAnimator (mirrors BirdAnimator.cs)
// =============================================================================

/// Cycles through wing-flap textures at a fixed cadence.
pub struct BirdAnimator {
    pub frame_index: usize,
    animation_time: f32,
}

impl BirdAnimator {
    pub fn new() -> Self {
        Self {
            frame_index: 0,
            animation_time: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.animation_time += dt;
        if self.animation_time >= ANIMATION_FRAME_DURATION {
            self.frame_index = (self.frame_index + 1) % 3;
            self.animation_time = 0.0;
        }
    }

    pub fn reset(&mut self) {
        self.frame_index = 0;
        self.animation_time = 0.0;
    }
}

// =============================================================================
// Bird (mirrors Bird.cs)
// =============================================================================

/// The player-controlled bird: movement physics plus animation state.
pub struct Bird {
    pub x: f32,
    pub y: f32,
    movement: Movement,
    pub animator: BirdAnimator,
}

impl Bird {
    pub fn new() -> Self {
        let x = BIRD_START_X;
        let y = BIRD_START_Y;
        Self {
            x,
            y,
            movement: Movement::new(GRAVITY, JUMP_STRENGTH),
            animator: BirdAnimator::new(),
        }
    }

    pub fn reset(&mut self) {
        self.x = BIRD_START_X;
        self.y = BIRD_START_Y;
        self.movement.velocity = 0.0;
        self.movement.rotation = 0.0;
        self.animator.reset();
    }

    /// Updates bird state for one frame. `jump_pressed` is true when the
    /// player presses Space or clicks the left mouse button.
    pub fn update(&mut self, dt: f32, jump_pressed: bool) {
        if jump_pressed {
            self.movement.try_jump();
        }
        self.movement.apply_gravity(dt);
        self.movement.update_position(&mut self.y, dt);
        self.animator.update(dt);
    }

    /// Current rotation in degrees (for rendering).
    pub fn rotation_deg(&self) -> f32 {
        self.movement.rotation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- Movement tests -------------------------------------------------------

    #[test]
    fn test_gravity_increases_velocity() {
        let mut mov = Movement::new(GRAVITY, JUMP_STRENGTH);
        assert!((mov.velocity - 0.0).abs() < f32::EPSILON);
        let dt = 1.0 / 120.0;
        mov.apply_gravity(dt);
        assert!(
            mov.velocity > 0.0,
            "gravity should increase velocity downward"
        );
    }

    #[test]
    fn test_jump_sets_negative_velocity() {
        let mut mov = Movement::new(GRAVITY, JUMP_STRENGTH);
        mov.try_jump();
        assert!(
            mov.velocity < 0.0,
            "jump should set a negative (upward) velocity"
        );
    }

    #[test]
    fn test_jump_cooldown_prevents_double_jump() {
        let mut mov = Movement::new(GRAVITY, JUMP_STRENGTH);
        mov.try_jump();
        let first_velocity = mov.velocity;
        // Apply a tiny gravity tick so velocity changes
        mov.apply_gravity(1.0 / 120.0);
        mov.try_jump(); // should be blocked by cooldown
                        // Velocity should NOT be reset to jump strength
        assert!(
            (mov.velocity - first_velocity).abs() > f32::EPSILON,
            "second jump should be blocked by cooldown"
        );
    }

    #[test]
    fn test_jump_allowed_after_cooldown_expires() {
        let mut mov = Movement::new(GRAVITY, JUMP_STRENGTH);
        mov.try_jump();
        // Tick enough time to expire the cooldown
        let dt = JUMP_COOLDOWN + 0.01;
        mov.apply_gravity(dt);
        mov.try_jump();
        let expected = JUMP_STRENGTH * TARGET_FPS;
        assert!(
            (mov.velocity - expected).abs() < f32::EPSILON,
            "jump should succeed after cooldown expires"
        );
    }

    #[test]
    fn test_update_position_changes_y() {
        let mut mov = Movement::new(GRAVITY, JUMP_STRENGTH);
        mov.velocity = -5.0;
        let mut y = 100.0;
        let dt = 1.0 / 120.0;
        mov.update_position(&mut y, dt);
        assert!(y < 100.0, "negative velocity should move bird upward");
    }

    #[test]
    fn test_rotation_clamped_to_range() {
        let mut mov = Movement::new(GRAVITY, JUMP_STRENGTH);
        mov.velocity = 1000.0; // extreme downward
        let mut y = 100.0;
        // Run many updates to let rotation converge
        for _ in 0..10000 {
            mov.update_position(&mut y, 1.0 / 120.0);
        }
        assert!(
            mov.rotation <= 45.0 + 0.01,
            "rotation should not exceed 45 degrees, got {}",
            mov.rotation
        );
    }

    // -- BirdAnimator tests ---------------------------------------------------

    #[test]
    fn test_animator_starts_at_frame_zero() {
        let anim = BirdAnimator::new();
        assert_eq!(anim.frame_index, 0);
    }

    #[test]
    fn test_animator_cycles_frames() {
        let mut anim = BirdAnimator::new();
        // Advance past one full frame duration
        anim.update(ANIMATION_FRAME_DURATION + 0.001);
        assert_eq!(anim.frame_index, 1);
        anim.update(ANIMATION_FRAME_DURATION + 0.001);
        assert_eq!(anim.frame_index, 2);
        anim.update(ANIMATION_FRAME_DURATION + 0.001);
        assert_eq!(anim.frame_index, 0, "frame_index should wrap around to 0");
    }

    #[test]
    fn test_animator_no_advance_before_duration() {
        let mut anim = BirdAnimator::new();
        anim.update(ANIMATION_FRAME_DURATION * 0.5);
        assert_eq!(
            anim.frame_index, 0,
            "frame should not advance before duration elapses"
        );
    }

    #[test]
    fn test_animator_reset_clears_state() {
        let mut anim = BirdAnimator::new();
        anim.update(ANIMATION_FRAME_DURATION + 0.001);
        assert_eq!(anim.frame_index, 1);
        anim.reset();
        assert_eq!(anim.frame_index, 0);
    }

    // -- Bird integration tests -----------------------------------------------

    #[test]
    fn test_bird_starts_at_expected_position() {
        let bird = Bird::new();
        assert!((bird.x - BIRD_START_X).abs() < f32::EPSILON);
        assert!((bird.y - BIRD_START_Y).abs() < f32::EPSILON);
    }

    #[test]
    fn test_bird_falls_under_gravity() {
        let mut bird = Bird::new();
        let start_y = bird.y;
        // Run several frames without jumping
        for _ in 0..10 {
            bird.update(1.0 / 120.0, false);
        }
        assert!(bird.y > start_y, "bird should fall downward under gravity");
    }

    #[test]
    fn test_bird_jump_moves_upward() {
        let mut bird = Bird::new();
        // Let bird fall a bit first so the jump effect is clear
        for _ in 0..5 {
            bird.update(1.0 / 120.0, false);
        }
        let y_before_jump = bird.y;
        bird.update(1.0 / 120.0, true); // jump
                                        // Run one more frame to let the negative velocity take effect
        bird.update(1.0 / 120.0, false);
        assert!(
            bird.y < y_before_jump,
            "bird should move upward after jumping"
        );
    }

    #[test]
    fn test_bird_reset_restores_start_position() {
        let mut bird = Bird::new();
        for _ in 0..20 {
            bird.update(1.0 / 120.0, false);
        }
        bird.reset();
        assert!((bird.x - BIRD_START_X).abs() < f32::EPSILON);
        assert!((bird.y - BIRD_START_Y).abs() < f32::EPSILON);
        assert!((bird.rotation_deg() - 0.0).abs() < f32::EPSILON);
    }
}
