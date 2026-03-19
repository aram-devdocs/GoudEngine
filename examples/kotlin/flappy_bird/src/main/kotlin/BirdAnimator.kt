import com.goudengine.core.GoudGame

/**
 * Cycles through bird wing animation frames.
 */
class BirdAnimator(game: GoudGame) {
    private val frames: LongArray
    private var frameIndex = 0
    private var timer = 0f
    private val frameDuration = 0.15f

    init {
        frames = longArrayOf(
            game.loadTexture("assets/sprites/yellowbird-downflap.png"),
            game.loadTexture("assets/sprites/yellowbird-midflap.png"),
            game.loadTexture("assets/sprites/yellowbird-upflap.png"),
        )
    }

    fun update(dt: Float) {
        timer += dt
        if (timer >= frameDuration) {
            timer = 0f
            frameIndex = (frameIndex + 1) % frames.size
        }
    }

    fun currentFrame(): Long = frames[frameIndex]
}
