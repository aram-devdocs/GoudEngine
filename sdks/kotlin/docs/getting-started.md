# Getting Started with the GoudEngine Kotlin SDK

## Prerequisites

- JDK 17 or later (Temurin recommended)
- Gradle 8.5+ (wrapper included in `sdks/kotlin/`)
- The GoudEngine native library (`libgoud_engine.dylib` / `.so` / `.dll`)

## Setup

1. Clone the repository and build the native library:

```bash
cargo build --release -p goud-engine-core
```

2. Generate the Kotlin SDK:

```bash
python3 codegen/gen_kotlin.py
```

3. Build the SDK jar:

```bash
cd sdks/kotlin
./gradlew build --no-daemon
```

## Your First Window

```kotlin
import com.goudengine.core.EngineConfig
import com.goudengine.core.GoudEngine
import com.goudengine.core.GoudGame

fun main() {
    GoudEngine.ensureLoaded()

    val game = EngineConfig.create()
        .title("Hello GoudEngine")
        .width(800)
        .height(600)
        .build()

    while (!game.shouldClose()) {
        game.beginFrame()
        // draw here
        game.endFrame()
    }

    game.destroy()
}
```

## Loading Textures

```kotlin
val texture = game.loadTexture("assets/sprites/player.png")
```

## Drawing Sprites

```kotlin
import com.goudengine.types.Color

game.drawSprite(texture, x, y, width, height, rotation, Color.white())
```

## Handling Input

```kotlin
import com.goudengine.input.Key
import com.goudengine.input.MouseButton

if (game.isKeyJustPressed(Key.Space.value)) {
    // jump
}

if (game.isMouseButtonPressed(MouseButton.Left.value)) {
    val pos = game.getMousePosition()
    // use pos.x, pos.y
}
```

## Value Types

The SDK provides Kotlin data classes for common types:

```kotlin
import com.goudengine.types.Vec2
import com.goudengine.types.Vec3
import com.goudengine.types.Color
import com.goudengine.types.Rect

val position = Vec2(100f, 200f)
val direction = position.normalize()
val distance = position.distance(Vec2.zero())

val color = Color.rgb(1f, 0f, 0f) // red
val transparent = color.withAlpha(0.5f)

val bounds = Rect(0f, 0f, 100f, 50f)
val inside = bounds.contains(Vec2(50f, 25f)) // true
```

## Example Game

See `examples/kotlin/flappy_bird/` for a complete Flappy Bird implementation
that mirrors the C# and Python examples:

```bash
./dev.sh --sdk kotlin --game flappy_bird
```

## API Documentation

Generate HTML docs with Dokka:

```bash
cd sdks/kotlin
./gradlew dokkaHtml --no-daemon
# output in build/dokka/html/
```
