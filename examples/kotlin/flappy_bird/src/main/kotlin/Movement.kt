import kotlin.math.max
import kotlin.math.min

class Movement(
    private val gravity: Float,
    private val jumpStrength: Float
) {
    var velocity: Float = 0f
    var rotation: Float = 0f
        private set

    private var jumpCooldownTimer: Float = 0f

    private companion object {
        const val ROTATION_SMOOTHING = 0.03f
    }

    fun applyGravity(deltaTime: Float) {
        velocity += gravity * deltaTime * GameConstants.TARGET_FPS
        jumpCooldownTimer -= max(0f, deltaTime)
    }

    fun tryJump(deltaTime: Float): Boolean {
        if (jumpCooldownTimer <= 0f) {
            velocity = jumpStrength * GameConstants.TARGET_FPS
            jumpCooldownTimer = GameConstants.JUMP_COOLDOWN
            return true
        }
        return false
    }

    fun updatePosition(positionY: Float, deltaTime: Float): Float {
        val newY = positionY + velocity * deltaTime
        val targetRotation = min(45f, max(-45f, velocity * 3f))
        rotation += (targetRotation - rotation) * ROTATION_SMOOTHING
        return newY
    }
}
