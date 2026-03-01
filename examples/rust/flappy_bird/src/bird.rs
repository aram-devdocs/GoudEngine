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
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    initial_x: f32,
    initial_y: f32,
}

impl BirdAnimator {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            frame_index: 0,
            animation_time: 0.0,
            x,
            y,
            rotation: 0.0,
            initial_x: x,
            initial_y: y,
        }
    }

    pub fn update(&mut self, dt: f32, x: f32, y: f32, rotation: f32) {
        self.x = x;
        self.y = y;
        self.rotation = rotation;

        self.animation_time += dt;
        if self.animation_time >= ANIMATION_FRAME_DURATION {
            self.frame_index = (self.frame_index + 1) % 3;
            self.animation_time = 0.0;
        }
    }

    pub fn reset(&mut self) {
        self.frame_index = 0;
        self.animation_time = 0.0;
        self.x = self.initial_x;
        self.y = self.initial_y;
        self.rotation = 0.0;
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
            animator: BirdAnimator::new(x, y),
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
        self.animator
            .update(dt, self.x, self.y, self.movement.rotation);
    }

    /// Current rotation in degrees (for rendering).
    pub fn rotation_deg(&self) -> f32 {
        self.movement.rotation
    }
}
