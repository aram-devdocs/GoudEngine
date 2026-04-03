package com.goudengine.flappybird

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.goudengine.core.EngineConfig
import com.goudengine.core.GoudEngine

/**
 * Android entry point for the Flappy Bird example.
 *
 * Demonstrates three input methods on Android:
 * - Touch: tap the screen to flap (primary mobile input)
 * - Keyboard: press Space to flap (useful in the Android emulator)
 * - Gamepad: A button to flap
 *
 * The engine renders into a native surface provided by GoudEngine.
 * Assets are loaded from the APK's assets/ directory.
 */
class MainActivity : AppCompatActivity() {

    private var gameManager: GameManager? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        GoudEngine.ensureLoaded()

        val game = EngineConfig.create()
            .setTitle("Flappy Bird - Android")
            .setSize(GameConstants.SCREEN_WIDTH, GameConstants.SCREEN_HEIGHT)
            .build()

        val manager = GameManager(game)
        gameManager = manager

        // Run the game loop on the render thread.
        // On Android the engine creates its own surface; the Activity acts as
        // the host lifecycle owner.
        Thread {
            try {
                while (!game.shouldClose()) {
                    manager.update(game)
                    manager.render(game)
                }
                game.destroy()
            } catch (t: Throwable) {
                android.util.Log.e("GoudEngine", "Game loop crashed", t)
            }
        }.start()
    }
}
