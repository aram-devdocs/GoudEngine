import com.goudengine.core.GoudGame

class Pipe(private val game: GoudGame) {
    var x: Float = GameConstants.SCREEN_WIDTH.toFloat()
        private set
    val gapY: Float = (GameConstants.PIPE_GAP until (GameConstants.SCREEN_HEIGHT - GameConstants.PIPE_GAP)).random().toFloat()

    companion object {
        const val PIPE_WIDTH = 52f
        const val PIPE_HEIGHT = 320f
    }

    val topPipeY: Float get() = gapY - GameConstants.PIPE_GAP - PIPE_HEIGHT
    val bottomPipeY: Float get() = gapY + GameConstants.PIPE_GAP

    fun update(deltaTime: Float) {
        x -= GameConstants.PIPE_SPEED * deltaTime * GameConstants.TARGET_FPS
    }

    fun draw(pipeTextureId: Long) {
        // Top pipe (rotated 180 degrees)
        game.drawSprite(
            pipeTextureId,
            x + PIPE_WIDTH / 2f,
            topPipeY + PIPE_HEIGHT / 2f,
            PIPE_WIDTH,
            PIPE_HEIGHT,
            Math.PI.toFloat()
        )
        // Bottom pipe
        game.drawSprite(
            pipeTextureId,
            x + PIPE_WIDTH / 2f,
            bottomPipeY + PIPE_HEIGHT / 2f,
            PIPE_WIDTH,
            PIPE_HEIGHT,
            0f
        )
    }

    fun isOffScreen(): Boolean = x + GameConstants.PIPE_COLLISION_WIDTH < 0

    fun collidesWithBird(birdX: Float, birdY: Float, birdW: Float, birdH: Float): Boolean {
        return checkAABB(birdX, birdY, birdW, birdH, x, topPipeY, PIPE_WIDTH, PIPE_HEIGHT) ||
            checkAABB(birdX, birdY, birdW, birdH, x, bottomPipeY, PIPE_WIDTH, PIPE_HEIGHT)
    }

    private fun checkAABB(
        x1: Float, y1: Float, w1: Float, h1: Float,
        x2: Float, y2: Float, w2: Float, h2: Float
    ): Boolean {
        return x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2
    }
}
