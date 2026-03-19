import com.goudengine.core.GoudGame

class ScoreCounter {
    var score: Int = 0
        private set

    private val digitTextures = LongArray(10)

    private var xOffset = 0f
    private var yOffset = 50f

    companion object {
        const val DIGIT_WIDTH = 24f
        const val DIGIT_HEIGHT = 36f
        const val DIGIT_SPACING = 30f
    }

    fun initialize(game: GoudGame) {
        for (i in 0 until 10) {
            digitTextures[i] = game.loadTexture("assets/sprites/$i.png")
        }
        xOffset = GameConstants.SCREEN_WIDTH / 2f - 30f
    }

    fun increment() { score++ }
    fun reset() { score = 0 }

    fun draw(game: GoudGame) {
        val scoreString = score.toString()
        for ((i, ch) in scoreString.withIndex()) {
            val digit = ch - '0'
            game.drawSprite(
                digitTextures[digit],
                xOffset + i * DIGIT_SPACING + DIGIT_WIDTH / 2f,
                yOffset + DIGIT_HEIGHT / 2f,
                DIGIT_WIDTH,
                DIGIT_HEIGHT
            )
        }
    }
}
