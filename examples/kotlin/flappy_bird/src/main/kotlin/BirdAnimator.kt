import com.goudengine.core.GoudGame

class BirdAnimator(private val game: GoudGame) {
    private val frameTextures = mutableListOf<Long>()
    private var currentFrame = 0
    private var animationTime = 0f
    private val frameDuration = 0.1f

    private var currentX = 0f
    private var currentY = 0f
    private var currentRotation = 0f

    companion object {
        const val BIRD_WIDTH = 34f
        const val BIRD_HEIGHT = 24f
    }

    fun initialize() {
        frameTextures.add(game.loadTexture("assets/sprites/bluebird-downflap.png"))
        frameTextures.add(game.loadTexture("assets/sprites/bluebird-midflap.png"))
        frameTextures.add(game.loadTexture("assets/sprites/bluebird-upflap.png"))
    }

    fun update(deltaTime: Float, x: Float, y: Float, rotation: Float) {
        animationTime += deltaTime
        if (animationTime >= frameDuration) {
            currentFrame = (currentFrame + 1) % frameTextures.size
            animationTime = 0f
        }
        currentX = x
        currentY = y
        currentRotation = rotation
    }

    fun draw() {
        if (frameTextures.isEmpty()) return
        game.drawSprite(
            frameTextures[currentFrame],
            currentX + BIRD_WIDTH / 2f,
            currentY + BIRD_HEIGHT / 2f,
            BIRD_WIDTH,
            BIRD_HEIGHT,
            currentRotation * Math.PI.toFloat() / 180f
        )
    }

    fun reset() {
        currentFrame = 0
        animationTime = 0f
        currentRotation = 0f
    }
}
