import com.goudengine.core.GoudGame
import com.goudengine.types.Color
import kotlin.random.Random

/**
 * A pair of top/bottom pipes scrolling leftward.
 */
class Pipe private constructor(
    private val game: GoudGame,
    var x: Float,
    private val gapY: Float,
    private val pipeTexture: Long,
) {
    var scored = false

    fun update(dt: Float) {
        x = Movement.scrollLeft(x, GameConstants.PIPE_SPEED, dt)
    }

    fun render(game: GoudGame) {
        val pw = GameConstants.PIPE_WIDTH.toFloat()
        val gap = GameConstants.PIPE_GAP.toFloat()
        val topH = gapY
        val bottomY = gapY + gap
        val bottomH = GameConstants.SCREEN_HEIGHT.toFloat() - bottomY

        // Top pipe
        game.drawSprite(pipeTexture, x, 0f, pw, topH, 0f, Color.white())
        // Bottom pipe
        game.drawSprite(pipeTexture, x, bottomY, pw, bottomH, 0f, Color.white())
    }

    fun isOffScreen(): Boolean = x + GameConstants.PIPE_WIDTH < 0

    fun collidesWith(birdX: Float, birdY: Float): Boolean {
        val bw = GameConstants.BIRD_WIDTH.toFloat()
        val bh = GameConstants.BIRD_HEIGHT.toFloat()
        val pw = GameConstants.PIPE_WIDTH.toFloat()
        val gap = GameConstants.PIPE_GAP.toFloat()

        if (birdX + bw < x || birdX > x + pw) return false
        if (birdY < gapY || birdY + bh > gapY + gap) return true
        return false
    }

    companion object {
        fun spawn(game: GoudGame): Pipe {
            val minY = 50f
            val maxY = (GameConstants.SCREEN_HEIGHT - GameConstants.BASE_HEIGHT - GameConstants.PIPE_GAP - 50).toFloat()
            val gapY = Random.nextFloat() * (maxY - minY) + minY
            val texture = game.loadTexture("assets/sprites/pipe-green.png")
            return Pipe(game, GameConstants.SCREEN_WIDTH.toFloat(), gapY, texture)
        }
    }
}
