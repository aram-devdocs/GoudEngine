# Getting Started with GoudEngine Kotlin SDK

This guide walks you through setting up a Kotlin/JVM project that uses GoudEngine to create a game window, load textures, handle input, and render sprites.

## Prerequisites

- **JDK 17** or later (Temurin recommended)
- **Gradle 8.5+** (or use the wrapper included in the SDK)
- **Rust toolchain** (for building the native engine library)
- **GoudEngine** cloned and built (`cargo build --release`)

## Project Setup

Add the GoudEngine Kotlin SDK as a dependency in your `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.goudengine:goud-engine-kotlin:0.0.832")
}
```

If you're working from the repo, you can reference the SDK JAR directly:

```kotlin
dependencies {
    implementation(files("path/to/sdks/kotlin/build/libs/goud-engine-kotlin-0.0.832.jar"))
}
```

Make sure the native library (`libgoud_engine.dylib` on macOS, `libgoud_engine.so` on Linux, `goud_engine.dll` on Windows) is on `java.library.path`.

## Creating Your First Window

```kotlin
import com.goudengine.core.EngineConfig

fun main() {
    val config = EngineConfig.create()
        .setSize(800, 600)
        .setTitle("My First GoudEngine Game")

    val game = config.build()

    while (!game.shouldClose()) {
        game.beginFrame(0.2f, 0.3f, 0.3f, 1.0f) // Dark teal background
        // Your game logic here
        game.endFrame()
    }

    game.destroy()
}
```

## Loading Textures

```kotlin
val textureId = game.loadTexture("assets/sprites/player.png")
```

Textures are loaded once and referenced by their handle (a `Long` value). Use `drawSprite` to render them each frame.

## Rendering Sprites

```kotlin
// Draw at center position (x, y) with size (width, height)
game.drawSprite(textureId, 100f, 100f, 64f, 64f)

// Draw with rotation (radians)
game.drawSprite(textureId, 200f, 200f, 64f, 64f, 1.57f)
```

`drawSprite` uses center-based positioning, meaning the coordinates specify the center of the sprite.

## Handling Input

```kotlin
import com.goudengine.input.Key
import com.goudengine.input.MouseButton

// Check keyboard
if (game.isKeyPressed(Key.Space)) {
    // Space bar was pressed this frame
}

if (game.isKeyDown(Key.Left)) {
    // Left arrow is held down
}

// Check mouse
if (game.isMouseButtonPressed(MouseButton.Left)) {
    // Left mouse button was clicked this frame
}
```

## Using Value Types

The SDK provides familiar math types:

```kotlin
import com.goudengine.types.Vec2
import com.goudengine.types.Color

val position = Vec2(100f, 200f)
val velocity = Vec2(1f, 0f).scale(5f)
val newPos = position.add(velocity)

val red = Color.rgb(1f, 0f, 0f)
val semiTransparent = red.withAlpha(0.5f)
```

## Full Example

Check out the Flappy Bird example in `examples/kotlin/flappy_bird/` for a complete game demonstrating sprites, animation, collision detection, and scoring. Run it with:

```bash
./dev.sh --sdk kotlin --game flappy_bird
```

## API Reference

Generate the full API documentation with Dokka:

```bash
cd sdks/kotlin
./gradlew dokkaHtml
```

The generated HTML documentation will be at `build/dokka/html/index.html`.
