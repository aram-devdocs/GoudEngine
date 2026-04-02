import com.goudengine.core.EngineConfig
import com.goudengine.core.GoudEngine
import com.goudengine.types.Color

fun main() {
    GoudEngine.ensureLoaded()

    val game = EngineConfig.create()
        .title("Hello GoudEngine")
        .width(800)
        .height(600)
        .build()

    val texture = game.loadTexture("assets/player.png")

    while (!game.shouldClose()) {
        game.beginFrame()
        game.drawSprite(texture, 400f, 300f, 64f, 64f, 0f, Color.white())
        game.endFrame()
    }

    game.destroy()
}
