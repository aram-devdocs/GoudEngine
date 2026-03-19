import com.goudengine.core.GoudGame
import com.goudengine.input.Key
import com.goudengine.types.Color

/**
 * The player-controlled bird.
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

        if (game.isKeyJustPressed(Key.Space) && jumpCooldown <= 0f) {
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
}
