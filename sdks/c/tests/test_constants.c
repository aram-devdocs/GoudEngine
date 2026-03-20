/* Test that all Flappy Bird game constants match the Rust source of truth.
 * Source of truth: examples/rust/flappy_bird/src/constants.rs */

#include <assert.h>
#include <math.h>
#include <stdio.h>

/* Include the C flappy bird constants */
#include "../../../examples/c/flappy_bird/src/constants.h"

/* Tolerance for floating-point comparisons */
#define FLOAT_EQ(a, b) (fabsf((a) - (b)) < 1e-6f)

int main(void) {
    /* Screen */
    assert(FLOAT_EQ(SCREEN_WIDTH, 288.0f));
    assert(FLOAT_EQ(SCREEN_HEIGHT, 512.0f));
    assert(FLOAT_EQ(BASE_HEIGHT, 112.0f));
    assert(FLOAT_EQ(TARGET_FPS, 120.0f));

    /* Bird */
    assert(FLOAT_EQ(BIRD_WIDTH, 34.0f));
    assert(FLOAT_EQ(BIRD_HEIGHT, 24.0f));
    assert(FLOAT_EQ(BIRD_START_X, 288.0f / 4.0f));
    assert(FLOAT_EQ(BIRD_START_Y, 512.0f / 2.0f));

    /* Physics */
    assert(FLOAT_EQ(GRAVITY, 9.8f));
    assert(FLOAT_EQ(JUMP_STRENGTH, -3.5f));
    assert(FLOAT_EQ(JUMP_COOLDOWN, 0.3f));

    /* Pipes */
    assert(FLOAT_EQ(PIPE_SPEED, 1.0f));
    assert(FLOAT_EQ(PIPE_SPAWN_INTERVAL, 1.5f));
    assert(FLOAT_EQ(PIPE_GAP, 100.0f));
    assert(FLOAT_EQ(PIPE_COLLISION_WIDTH, 60.0f));
    assert(FLOAT_EQ(PIPE_IMAGE_WIDTH, 52.0f));
    assert(FLOAT_EQ(PIPE_IMAGE_HEIGHT, 320.0f));

    /* Score */
    assert(FLOAT_EQ(SCORE_DIGIT_WIDTH, 24.0f));
    assert(FLOAT_EQ(SCORE_DIGIT_HEIGHT, 36.0f));
    assert(FLOAT_EQ(SCORE_DIGIT_SPACING, 30.0f));

    /* Animation */
    assert(FLOAT_EQ(ANIMATION_FRAME_DURATION, 0.1f));
    assert(FLOAT_EQ(ROTATION_SMOOTHING, 0.03f));

    /* Rendering dimensions */
    assert(FLOAT_EQ(BACKGROUND_WIDTH, 288.0f));
    assert(FLOAT_EQ(BACKGROUND_HEIGHT, 512.0f));
    assert(FLOAT_EQ(BASE_SPRITE_WIDTH, 336.0f));

    printf("test_constants: all assertions passed\n");
    return 0;
}
