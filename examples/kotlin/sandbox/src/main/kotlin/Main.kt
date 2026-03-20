import com.goudengine.core.EngineConfig
import com.goudengine.core.GoudEngine
import com.goudengine.core.GoudGame
import com.goudengine.input.Key
import com.goudengine.types.Color
import kotlin.math.sin

/**
 * GoudEngine Sandbox -- Kotlin SDK
 *
 * A simplified interactive sandbox demonstrating core engine features:
 *   - Window creation and configuration
 *   - Texture loading and sprite drawing
 *   - Colored quad drawing
 *   - Keyboard and mouse input
 *   - Mode switching (1/2/3 keys)
 *
 * Run with: ./dev.sh --sdk kotlin --game sandbox
 *
 * Note: This example follows the same API patterns as the Kotlin Flappy Bird
 * example, using beginFrame()/endFrame()/deltaTime() which require the Kotlin
 * SDK frame-loop methods to be fully wired.
 */

const val WINDOW_WIDTH = 1280
const val WINDOW_HEIGHT = 720
const val MOVE_SPEED = 220.0f

enum class Mode(val label: String) {
    MODE_2D("2D"),
    MODE_3D("3D"),
    MODE_HYBRID("Hybrid"),
}

fun main() {
    GoudEngine.ensureLoaded()

    val game = EngineConfig.create()
        .setTitle("GoudEngine Sandbox - Kotlin")
        .setSize(WINDOW_WIDTH, WINDOW_HEIGHT)
        .build()

    // Load textures (shared assets from C# flappy_goud)
    val background = game.loadTexture("assets/sprites/background-day.png")
    val sprite = game.loadTexture("assets/sprites/bluebird-midflap.png")
    println("Textures loaded.")

    var playerX = 250.0f
    var playerY = 300.0f
    var angle = 0.0f
    var currentMode = Mode.MODE_2D

    println("Starting sandbox...")
    println("  WASD/Arrows: move sprite")
    println("  1/2/3: switch mode (2D / 3D / Hybrid)")
    println("  Escape: quit")

    while (!game.shouldClose()) {
        // Note: beginFrame/endFrame/deltaTime are not yet in the auto-generated
        // Kotlin SDK. This example is written for forward compatibility, matching
        // the pattern used in the Kotlin flappy_bird example.
        val dt = 0.016f // placeholder until deltaTime() is wired
        angle += dt

        // Input: quit
        if (game.isKeyJustPressed(Key.Escape)) {
            game.requestClose()
        }

        // Input: mode switching
        if (game.isKeyJustPressed(Key.Digit1)) {
            currentMode = Mode.MODE_2D
            println("Mode: 2D")
        }
        if (game.isKeyJustPressed(Key.Digit2)) {
            currentMode = Mode.MODE_3D
            println("Mode: 3D")
        }
        if (game.isKeyJustPressed(Key.Digit3)) {
            currentMode = Mode.MODE_HYBRID
            println("Mode: Hybrid")
        }

        // Input: movement
        if (game.isKeyPressed(Key.A) || game.isKeyPressed(Key.Left)) {
            playerX -= MOVE_SPEED * dt
        }
        if (game.isKeyPressed(Key.D) || game.isKeyPressed(Key.Right)) {
            playerX += MOVE_SPEED * dt
        }
        if (game.isKeyPressed(Key.W) || game.isKeyPressed(Key.Up)) {
            playerY -= MOVE_SPEED * dt
        }
        if (game.isKeyPressed(Key.S) || game.isKeyPressed(Key.Down)) {
            playerY += MOVE_SPEED * dt
        }

        // Rendering per mode
        when (currentMode) {
            Mode.MODE_2D -> {
                game.drawSprite(
                    background, 0f, 0f,
                    WINDOW_WIDTH.toFloat(), WINDOW_HEIGHT.toFloat(),
                    0f, Color.white()
                )
                game.drawSprite(
                    sprite, playerX, playerY,
                    64f, 64f,
                    angle * 0.25f, Color.white()
                )
                game.drawQuad(920f, 260f, 180f, 40f, Color(0.20f, 0.55f, 0.95f, 0.80f))
            }
            Mode.MODE_3D -> {
                game.drawQuad(
                    WINDOW_WIDTH / 2f, WINDOW_HEIGHT / 2f,
                    WINDOW_WIDTH.toFloat(), WINDOW_HEIGHT.toFloat(),
                    Color(0.05f, 0.08f, 0.12f, 1.0f)
                )
                game.drawQuad(
                    WINDOW_WIDTH / 2f, WINDOW_HEIGHT / 2f - 40f,
                    400f, 60f,
                    Color(0.20f, 0.55f, 0.95f, 0.60f)
                )
            }
            Mode.MODE_HYBRID -> {
                game.drawQuad(
                    WINDOW_WIDTH / 2f, WINDOW_HEIGHT / 2f,
                    WINDOW_WIDTH.toFloat(), WINDOW_HEIGHT.toFloat(),
                    Color(0.08f, 0.17f, 0.24f, 1.0f)
                )
                game.drawSprite(
                    sprite, playerX, playerY,
                    72f, 72f,
                    angle * 0.25f, Color.white()
                )
                game.drawQuad(920f, 260f, 180f, 40f, Color(0.20f, 0.55f, 0.95f, 0.62f))
            }
        }

        // Mouse marker
        val mouse = game.getMousePosition()
        game.drawQuad(mouse.x, mouse.y, 14f, 14f, Color(0.95f, 0.85f, 0.20f, 0.95f))

        // Mode badge
        game.drawQuad(
            WINDOW_WIDTH / 2f, 20f, 200f, 30f,
            Color(0.20f, 0.55f, 0.95f, 0.84f)
        )

        // Oscillating decoration
        val bobY = 600f + 20f * sin(angle * 2.0).toFloat()
        game.drawQuad(100f, bobY, 60f, 60f, Color(0.90f, 0.30f, 0.40f, 0.75f))
    }

    game.destroy()
    println("Sandbox closed.")
}
