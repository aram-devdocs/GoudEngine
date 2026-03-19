import com.goudengine.core.GoudGame
import com.goudengine.input.Key
import com.goudengine.input.MouseButton

class Bird(private val game: GoudGame) {
    private val movement = Movement(GameConstants.GRAVITY, GameConstants.JUMP_STRENGTH)
    private val animator = BirdAnimator(game)

    var x: Float = GameConstants.SCREEN_WIDTH / 4f
        private set
    var y: Float = GameConstants.SCREEN_HEIGHT / 2f
        private set

    companion object {
        const val WIDTH = 34f
        const val HEIGHT = 24f
    }

    fun initialize() {
        animator.initialize()
    }

    fun reset() {
        x = GameConstants.SCREEN_WIDTH / 4f
        y = GameConstants.SCREEN_HEIGHT / 2f
        movement.velocity = 0f
        animator.reset()
    }

    fun update(deltaTime: Float): Boolean {
        var didFlap = false

        if (game.isKeyPressed(Key.Space) || game.isMouseButtonPressed(MouseButton.Left)) {
            didFlap = movement.tryJump(deltaTime)
        }

        movement.applyGravity(deltaTime)
        y = movement.updatePosition(y, deltaTime)
        animator.update(deltaTime, x, y, movement.rotation)

        return didFlap
    }

    fun draw() {
        animator.draw()
    }
}
