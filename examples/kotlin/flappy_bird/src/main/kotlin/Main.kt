import com.goudengine.core.EngineConfig
import com.goudengine.core.GoudEngine
import com.goudengine.core.GoudGame

/**
 * Kotlin Flappy Bird -- mirrors the C# flappy_goud example.
 *
 * Run with: ./dev.sh --sdk kotlin --game flappy_bird
 */
fun main() {
    GoudEngine.ensureLoaded()

    val game = EngineConfig.create()
        .setTitle("Flappy Bird - Kotlin")
        .setSize(GameConstants.SCREEN_WIDTH, GameConstants.SCREEN_HEIGHT)
        .build()

    val manager = GameManager(game)

    while (!game.shouldClose()) {
        manager.update(game)
        manager.render(game)
    }

    game.destroy()
}
