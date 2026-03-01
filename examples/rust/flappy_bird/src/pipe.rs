use crate::constants::*;
use rand::Rng;

/// A pair of pipes (top and bottom) forming an obstacle.
pub struct Pipe {
    pub x: f32,
    pub gap_y: f32,
}

impl Pipe {
    /// Creates a new pipe pair at the right edge of the screen with a
    /// random gap position (matching the C#/Python implementations).
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let gap_y = rng.gen_range(PIPE_GAP as i32..(SCREEN_HEIGHT - PIPE_GAP) as i32) as f32;
        Self {
            x: SCREEN_WIDTH,
            gap_y,
        }
    }

    /// Y coordinate of the top of the top pipe sprite.
    pub fn top_pipe_y(&self) -> f32 {
        self.gap_y - PIPE_GAP - PIPE_IMAGE_HEIGHT
    }

    /// Y coordinate of the top of the bottom pipe sprite.
    pub fn bottom_pipe_y(&self) -> f32 {
        self.gap_y + PIPE_GAP
    }

    /// Moves the pipe leftward by `PIPE_SPEED * dt * TARGET_FPS`.
    pub fn update(&mut self, dt: f32) {
        self.x -= PIPE_SPEED * dt * TARGET_FPS;
    }

    /// Returns true when the pipe has scrolled completely off-screen.
    pub fn is_off_screen(&self) -> bool {
        self.x + PIPE_COLLISION_WIDTH < 0.0
    }

    /// AABB collision test against the bird bounds.
    pub fn collides_with_bird(&self, bird_x: f32, bird_y: f32, bird_w: f32, bird_h: f32) -> bool {
        // Top pipe
        if aabb_overlap(
            bird_x,
            bird_y,
            bird_w,
            bird_h,
            self.x,
            self.top_pipe_y(),
            PIPE_IMAGE_WIDTH,
            PIPE_IMAGE_HEIGHT,
        ) {
            return true;
        }
        // Bottom pipe
        aabb_overlap(
            bird_x,
            bird_y,
            bird_w,
            bird_h,
            self.x,
            self.bottom_pipe_y(),
            PIPE_IMAGE_WIDTH,
            PIPE_IMAGE_HEIGHT,
        )
    }
}

/// Axis-Aligned Bounding Box overlap test.
fn aabb_overlap(x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
    x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_starts_at_right_edge() {
        let pipe = Pipe::new();
        assert!((pipe.x - SCREEN_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn test_pipe_moves_left() {
        let mut pipe = Pipe::new();
        let start_x = pipe.x;
        pipe.update(1.0 / 120.0);
        assert!(pipe.x < start_x);
    }

    #[test]
    fn test_pipe_gap_in_valid_range() {
        for _ in 0..100 {
            let pipe = Pipe::new();
            assert!(pipe.gap_y >= PIPE_GAP);
            assert!(pipe.gap_y < SCREEN_HEIGHT - PIPE_GAP);
        }
    }

    #[test]
    fn test_aabb_overlap_basic() {
        assert!(aabb_overlap(0.0, 0.0, 10.0, 10.0, 5.0, 5.0, 10.0, 10.0));
        assert!(!aabb_overlap(0.0, 0.0, 5.0, 5.0, 10.0, 10.0, 5.0, 5.0));
    }

    #[test]
    fn test_pipe_off_screen_after_long_travel() {
        let mut pipe = Pipe::new();
        for _ in 0..10000 {
            pipe.update(1.0 / 60.0);
        }
        assert!(pipe.is_off_screen());
    }
}
