import com.goudengine.core.GoudGame
import com.goudengine.input.Key

class GameManager(private val game: GoudGame) {
    private val scoreCounter = ScoreCounter()
    private val bird = Bird(game)
    private val pipes = mutableListOf<Pipe>()
    private var pipeSpawnTimer = 0f

    private var backgroundTextureId = 0L
    private var baseTextureId = 0L
    private var pipeTextureId = 0L

    companion object {
        const val BG_WIDTH = 288f
        const val BG_HEIGHT = 512f
        const val BASE_WIDTH = 336f
        const val BASE_HEIGHT = 112f
    }

    fun initialize() {
        backgroundTextureId = game.loadTexture("assets/sprites/background-day.png")
        baseTextureId = game.loadTexture("assets/sprites/base.png")
        pipeTextureId = game.loadTexture("assets/sprites/pipe-green.png")

        bird.initialize()
        scoreCounter.initialize(game)
    }

    fun start() {
        bird.reset()
        pipes.clear()
        scoreCounter.reset()
        pipeSpawnTimer = 0f
    }

    fun update(deltaTime: Float) {
        if (game.isKeyPressed(Key.Escape)) {
            game.requestClose()
            return
        }

        if (game.isKeyPressed(Key.R)) {
            resetGame()
            return
        }

        bird.update(deltaTime)

        // Ground collision
        if (bird.y + Bird.HEIGHT > GameConstants.SCREEN_HEIGHT) {
            resetGame()
            return
        }
        // Off-screen top
        if (bird.y < 0) {
            resetGame()
            return
        }

        // Update pipes and check collisions
        for (pipe in pipes) {
            pipe.update(deltaTime)
            if (pipe.collidesWithBird(bird.x, bird.y, Bird.WIDTH, Bird.HEIGHT)) {
                resetGame()
                return
            }
        }

        // Spawn new pipes
        pipeSpawnTimer += deltaTime
        if (pipeSpawnTimer > GameConstants.PIPE_SPAWN_INTERVAL) {
            pipeSpawnTimer = 0f
            pipes.add(Pipe(game))
        }

        // Remove off-screen pipes and score
        pipes.removeAll { pipe ->
            if (pipe.isOffScreen()) {
                scoreCounter.increment()
                true
            } else false
        }

        draw()
    }

    private fun draw() {
        // Background
        game.drawSprite(backgroundTextureId, BG_WIDTH / 2f, BG_HEIGHT / 2f, BG_WIDTH, BG_HEIGHT)

        // Score
        scoreCounter.draw(game)

        // Pipes
        for (pipe in pipes) {
            pipe.draw(pipeTextureId)
        }

        // Bird
        bird.draw()

        // Base
        game.drawSprite(
            baseTextureId,
            BASE_WIDTH / 2f,
            GameConstants.SCREEN_HEIGHT + BASE_HEIGHT / 2f,
            BASE_WIDTH,
            BASE_HEIGHT
        )
    }

    private fun resetGame() {
        scoreCounter.reset()
        start()
    }
}
