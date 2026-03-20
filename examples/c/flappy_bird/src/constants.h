#ifndef FLAPPY_CONSTANTS_H
#define FLAPPY_CONSTANTS_H

/* Game constants matching all other Flappy Bird implementations.
 * Source of truth: examples/rust/flappy_bird/src/constants.rs */

/* -- Screen ---------------------------------------------------------------- */
#define SCREEN_WIDTH         288.0f
#define SCREEN_HEIGHT        512.0f
#define BASE_HEIGHT          112.0f
#define TARGET_FPS           120.0f

/* -- Bird ------------------------------------------------------------------ */
#define BIRD_WIDTH           34.0f
#define BIRD_HEIGHT          24.0f
#define BIRD_START_X         (SCREEN_WIDTH / 4.0f)
#define BIRD_START_Y         (SCREEN_HEIGHT / 2.0f)

/* -- Physics --------------------------------------------------------------- */
#define GRAVITY              9.8f
#define JUMP_STRENGTH        (-3.5f)
#define JUMP_COOLDOWN        0.3f

/* -- Pipes ----------------------------------------------------------------- */
#define PIPE_SPEED           1.0f
#define PIPE_SPAWN_INTERVAL  1.5f
#define PIPE_GAP             100.0f
#define PIPE_COLLISION_WIDTH 60.0f
#define PIPE_IMAGE_WIDTH     52.0f
#define PIPE_IMAGE_HEIGHT    320.0f

/* -- Score ----------------------------------------------------------------- */
#define SCORE_DIGIT_WIDTH    24.0f
#define SCORE_DIGIT_HEIGHT   36.0f
#define SCORE_DIGIT_SPACING  30.0f

/* -- Animation ------------------------------------------------------------- */
#define ANIMATION_FRAME_DURATION 0.1f
#define ROTATION_SMOOTHING       0.03f

/* -- Rendering dimensions -------------------------------------------------- */
#define BACKGROUND_WIDTH     288.0f
#define BACKGROUND_HEIGHT    512.0f
#define BASE_SPRITE_WIDTH    336.0f

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

#endif
