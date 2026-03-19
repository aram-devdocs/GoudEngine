import com.goudengine.core.GoudGame
import com.goudengine.input.Key
import com.goudengine.types.Color

/**
 * Manages overall game state: menu, playing, game-over transitions.
 */
class GameManager(game: GoudGame) {
    private enum class State { MENU, PLAYING, GAME_OVER }

    private var state = State.MENU
    private val bird = Bird(game)
    private val pipes = mutableListOf<Pipe>()
    private val score = ScoreCounter()

    private var pipeTimer = 0f
    private var bgTexture = 0L
    private var baseTexture = 0L

    init {
        bgTexture = game.loadTexture("assets/sprites/background-day.png")
        baseTexture = game.loadTexture("assets/sprites/base.png")
    }

    fun update(game: GoudGame) {
        val dt = game.deltaTime()

        when (state) {
            State.MENU -> {
                if (game.isKeyJustPressed(Key.Space)) {
                    state = State.PLAYING
                }
            }
            State.PLAYING -> {
                bird.update(game, dt)
                updatePipes(game, dt)

                if (checkCollisions()) {
                    state = State.GAME_OVER
                }
            }
            State.GAME_OVER -> {
                if (game.isKeyJustPressed(Key.Space)) {
                    reset(game)
                }
            }
        }
    }

    fun render(game: GoudGame) {
        game.beginFrame()

        // Background
        game.drawSprite(
            bgTexture, 0f, 0f,
            GameConstants.SCREEN_WIDTH.toFloat(),
            GameConstants.SCREEN_HEIGHT.toFloat(),
            0f, Color.white()
        )

        // Pipes
        for (pipe in pipes) {
            pipe.render(game)
        }

        // Base
        game.drawSprite(
            baseTexture, 0f,
            (GameConstants.SCREEN_HEIGHT - GameConstants.BASE_HEIGHT).toFloat(),
            GameConstants.SCREEN_WIDTH.toFloat(),
            GameConstants.BASE_HEIGHT.toFloat(),
            0f, Color.white()
        )

        // Bird
        bird.render(game)

        game.endFrame()
    }

    private fun updatePipes(game: GoudGame, dt: Float) {
        pipeTimer += dt
        if (pipeTimer >= GameConstants.PIPE_SPAWN_INTERVAL) {
            pipeTimer = 0f
            pipes.add(Pipe.spawn(game))
        }

        val iter = pipes.iterator()
        while (iter.hasNext()) {
            val pipe = iter.next()
            pipe.update(dt)
            if (pipe.isOffScreen()) {
                iter.remove()
            } else if (!pipe.scored && pipe.x + GameConstants.PIPE_WIDTH < bird.x) {
                pipe.scored = true
                score.increment()
            }
        }
    }

    private fun checkCollisions(): Boolean {
        // Ground collision
        val groundY = GameConstants.SCREEN_HEIGHT - GameConstants.BASE_HEIGHT
        if (bird.y + GameConstants.BIRD_HEIGHT >= groundY) return true

        // Pipe collisions
        for (pipe in pipes) {
            if (pipe.collidesWith(bird.x, bird.y)) return true
        }

        return false
    }

    private fun reset(game: GoudGame) {
        bird.reset()
        pipes.clear()
        score.reset()
        pipeTimer = 0f
        state = State.PLAYING
    }
}
