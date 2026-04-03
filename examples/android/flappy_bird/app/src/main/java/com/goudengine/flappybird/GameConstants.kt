package com.goudengine.flappybird

/**
 * Game constants matching the canonical C# flappy_goud values.
 * Source of truth: examples/rust/flappy_bird/src/constants.rs
 */
object GameConstants {
    const val TARGET_FPS: Int = 120
    const val BASE_HEIGHT: Int = 112

    const val SCREEN_WIDTH: Int = 288
    const val SCREEN_HEIGHT: Int = 512
    const val GRAVITY: Float = 9.8f
    const val JUMP_STRENGTH: Float = -3.5f
    const val JUMP_COOLDOWN: Float = 0.30f

    const val PIPE_SPEED: Float = 1.0f
    const val PIPE_SPAWN_INTERVAL: Float = 1.5f
    const val PIPE_WIDTH: Int = 60
    const val PIPE_GAP: Int = 100

    const val BIRD_WIDTH: Int = 34
    const val BIRD_HEIGHT: Int = 24
}
