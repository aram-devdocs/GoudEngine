/// Game constants matching the C#, Python, and TypeScript implementations exactly.
///
/// All values here are kept in sync with:
/// - `examples/csharp/flappy_goud/GameConstants.cs`
/// - `examples/python/flappy_bird.py`
/// - `examples/typescript/flappy_bird/`
// -- Screen -------------------------------------------------------------------
pub const SCREEN_WIDTH: f32 = 288.0;
pub const SCREEN_HEIGHT: f32 = 512.0;
pub const BASE_HEIGHT: f32 = 112.0;
pub const TARGET_FPS: f32 = 120.0;

// -- Bird ---------------------------------------------------------------------
pub const BIRD_WIDTH: f32 = 34.0;
pub const BIRD_HEIGHT: f32 = 24.0;
pub const BIRD_START_X: f32 = SCREEN_WIDTH / 4.0;
pub const BIRD_START_Y: f32 = SCREEN_HEIGHT / 2.0;

// -- Physics ------------------------------------------------------------------
pub const GRAVITY: f32 = 9.8;
pub const JUMP_STRENGTH: f32 = -3.5;
pub const JUMP_COOLDOWN: f32 = 0.3;

// -- Pipes --------------------------------------------------------------------
pub const PIPE_SPEED: f32 = 1.0;
pub const PIPE_SPAWN_INTERVAL: f32 = 1.5;
pub const PIPE_GAP: f32 = 100.0;
pub const PIPE_COLLISION_WIDTH: f32 = 60.0;
pub const PIPE_IMAGE_WIDTH: f32 = 52.0;
pub const PIPE_IMAGE_HEIGHT: f32 = 320.0;

// -- Score --------------------------------------------------------------------
pub const SCORE_DIGIT_WIDTH: f32 = 24.0;
pub const SCORE_DIGIT_HEIGHT: f32 = 36.0;
pub const SCORE_DIGIT_SPACING: f32 = 30.0;

// -- Animation ----------------------------------------------------------------
pub const ANIMATION_FRAME_DURATION: f32 = 0.1;
pub const ROTATION_SMOOTHING: f32 = 0.03;

// -- Rendering dimensions -----------------------------------------------------
pub const BACKGROUND_WIDTH: f32 = 288.0;
pub const BACKGROUND_HEIGHT: f32 = 512.0;
pub const BASE_SPRITE_WIDTH: f32 = 336.0;
