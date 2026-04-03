package com.goudengine.flappybird

import com.goudengine.core.GoudGame
import com.goudengine.input.GamepadButton
import com.goudengine.input.Key
import com.goudengine.types.Color

/**
 * The player-controlled bird.
 *
 * Supports three input methods for flapping:
 * - Touch: tap anywhere on the screen (primary mobile input)
 * - Keyboard: press Space (useful in Android emulator)
 * - Gamepad: A button / South face button
 */
class Bird(game: GoudGame) {
    var x: Float = GameConstants.SCREEN_WIDTH / 3f
    var y: Float = GameConstants.SCREEN_HEIGHT / 2f
    private var velocity: Float = 0f
    private var jumpCooldown: Float = 0f
    private val animator = BirdAnimator(game)

    fun update(game: GoudGame, dt: Float) {
        jumpCooldown -= dt
        val scaledDt = dt * GameConstants.TARGET_FPS

        if (shouldFlap(game) && jumpCooldown <= 0f) {
            velocity = GameConstants.JUMP_STRENGTH * GameConstants.TARGET_FPS
            jumpCooldown = GameConstants.JUMP_COOLDOWN
        }

        velocity += GameConstants.GRAVITY * scaledDt
        y += velocity * dt

        animator.update(dt)
    }

    fun render(game: GoudGame) {
        val texture = animator.currentFrame()
        game.drawSprite(
            texture, x, y,
            GameConstants.BIRD_WIDTH.toFloat(),
            GameConstants.BIRD_HEIGHT.toFloat(),
            0f, Color.white()
        )
    }

    fun reset() {
        x = GameConstants.SCREEN_WIDTH / 3f
        y = GameConstants.SCREEN_HEIGHT / 2f
        velocity = 0f
        jumpCooldown = 0f
    }

    /**
     * Returns true if the player wants to flap via any supported input method.
     *
     * Input methods:
     * 1. Touch - tap anywhere (touchId 0 = primary finger)
     * 2. Keyboard - Space key (works in Android emulator)
     * 3. Gamepad - A/South button on gamepad 0
     */
    private fun shouldFlap(game: GoudGame): Boolean {
        // Touch input: primary mobile input method
        if (game.isTouchJustPressed(0)) return true

        // Keyboard input: useful for Android emulator testing
        if (game.isKeyJustPressed(Key.Space)) return true

        // Gamepad input: A/South button on gamepad 0.
        if (game.isGamepadButtonJustPressed(0, GamepadButton.South.value)) return true

        return false
    }
}
