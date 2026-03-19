#include <catch2/catch_test_macros.hpp>
#include "constants.hpp"

TEST_CASE("Game constants match Rust source of truth", "[constants]") {
    REQUIRE(flappy::SCREEN_WIDTH == 288.0f);
    REQUIRE(flappy::SCREEN_HEIGHT == 512.0f);
    REQUIRE(flappy::BASE_HEIGHT == 112.0f);
    REQUIRE(flappy::TARGET_FPS == 120.0f);

    REQUIRE(flappy::BIRD_WIDTH == 34.0f);
    REQUIRE(flappy::BIRD_HEIGHT == 24.0f);
    REQUIRE(flappy::BIRD_START_X == 288.0f / 4.0f);
    REQUIRE(flappy::BIRD_START_Y == 512.0f / 2.0f);

    REQUIRE(flappy::GRAVITY == 9.8f);
    REQUIRE(flappy::JUMP_STRENGTH == -3.5f);
    REQUIRE(flappy::JUMP_COOLDOWN == 0.3f);

    REQUIRE(flappy::PIPE_SPEED == 1.0f);
    REQUIRE(flappy::PIPE_SPAWN_INTERVAL == 1.5f);
    REQUIRE(flappy::PIPE_GAP == 100.0f);
    REQUIRE(flappy::PIPE_COLLISION_WIDTH == 60.0f);
    REQUIRE(flappy::PIPE_IMAGE_WIDTH == 52.0f);
    REQUIRE(flappy::PIPE_IMAGE_HEIGHT == 320.0f);

    REQUIRE(flappy::SCORE_DIGIT_WIDTH == 24.0f);
    REQUIRE(flappy::SCORE_DIGIT_HEIGHT == 36.0f);
    REQUIRE(flappy::SCORE_DIGIT_SPACING == 30.0f);

    REQUIRE(flappy::ANIMATION_FRAME_DURATION == 0.1f);
    REQUIRE(flappy::ROTATION_SMOOTHING == 0.03f);

    REQUIRE(flappy::BACKGROUND_WIDTH == 288.0f);
    REQUIRE(flappy::BACKGROUND_HEIGHT == 512.0f);
    REQUIRE(flappy::BASE_SPRITE_WIDTH == 336.0f);
}
