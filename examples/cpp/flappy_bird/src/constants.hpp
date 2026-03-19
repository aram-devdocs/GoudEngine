#ifndef FLAPPY_CONSTANTS_HPP
#define FLAPPY_CONSTANTS_HPP

/// Game constants matching all other Flappy Bird implementations.
/// Source of truth: examples/rust/flappy_bird/src/constants.rs
namespace flappy {

// -- Screen -------------------------------------------------------------------
constexpr float SCREEN_WIDTH        = 288.0f;
constexpr float SCREEN_HEIGHT       = 512.0f;
constexpr float BASE_HEIGHT         = 112.0f;
constexpr float TARGET_FPS          = 120.0f;

// -- Bird ---------------------------------------------------------------------
constexpr float BIRD_WIDTH          = 34.0f;
constexpr float BIRD_HEIGHT         = 24.0f;
constexpr float BIRD_START_X        = SCREEN_WIDTH / 4.0f;
constexpr float BIRD_START_Y        = SCREEN_HEIGHT / 2.0f;

// -- Physics ------------------------------------------------------------------
constexpr float GRAVITY             = 9.8f;
constexpr float JUMP_STRENGTH       = -3.5f;
constexpr float JUMP_COOLDOWN       = 0.3f;

// -- Pipes --------------------------------------------------------------------
constexpr float PIPE_SPEED          = 1.0f;
constexpr float PIPE_SPAWN_INTERVAL = 1.5f;
constexpr float PIPE_GAP            = 100.0f;
constexpr float PIPE_COLLISION_WIDTH = 60.0f;
constexpr float PIPE_IMAGE_WIDTH    = 52.0f;
constexpr float PIPE_IMAGE_HEIGHT   = 320.0f;

// -- Score --------------------------------------------------------------------
constexpr float SCORE_DIGIT_WIDTH   = 24.0f;
constexpr float SCORE_DIGIT_HEIGHT  = 36.0f;
constexpr float SCORE_DIGIT_SPACING = 30.0f;

// -- Animation ----------------------------------------------------------------
constexpr float ANIMATION_FRAME_DURATION = 0.1f;
constexpr float ROTATION_SMOOTHING       = 0.03f;

// -- Rendering dimensions -----------------------------------------------------
constexpr float BACKGROUND_WIDTH    = 288.0f;
constexpr float BACKGROUND_HEIGHT   = 512.0f;
constexpr float BASE_SPRITE_WIDTH   = 336.0f;

}  // namespace flappy

#endif
