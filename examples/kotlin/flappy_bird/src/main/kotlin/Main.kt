import com.goudengine.core.EngineConfig
import com.goudengine.physics.PhysicsBackend2D

fun main() {
    val config = EngineConfig.create()
        .setSize(GameConstants.SCREEN_WIDTH, GameConstants.SCREEN_HEIGHT + GameConstants.BASE_HEIGHT)
        .setTitle("Flappy Bird Clone")
        .setPhysicsBackend2D(PhysicsBackend2D.Simple)

    val game = config.build()
    val manager = GameManager(game)

    manager.initialize()
    manager.start()

    while (!game.shouldClose()) {
        game.beginFrame(0.4f, 0.7f, 0.9f, 1.0f) // Sky blue background
        manager.update(game.deltaTime())
        game.endFrame()
    }

    game.destroy()
}
