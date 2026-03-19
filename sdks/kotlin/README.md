# GoudEngine Kotlin SDK

> **Alpha** -- This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

Kotlin/JVM bindings for GoudEngine using JNI. Published to Maven Central as `com.goudengine:goud-engine-kotlin`.

## Architecture

```
sdks/kotlin/
  build.gradle.kts              # Gradle build with native library bundling
  settings.gradle.kts           # Project settings
  gradle/                       # Gradle wrapper
  docs/
    getting-started.md          # Quick-start guide
  src/main/java/com/goudengine/
    internal/                   # JNI native method declarations (generated)
      GoudGameNative.java       # Game lifecycle JNI bindings
      GoudContextNative.java    # Headless context JNI bindings
      AudioNative.java          # Audio JNI bindings
      NetworkNative.java        # Networking JNI bindings
      ...
  src/main/kotlin/com/goudengine/
    core/                       # Wrapper classes
      GoudEngine.kt             # Native library loader
      GoudGame.kt               # Game window and loop
      GoudContext.kt            # Headless context
      EngineConfig.kt           # Builder for game configuration
      EntityHandle.kt           # ECS entity handle
      Audio.kt                  # Audio playback
      Network.kt                # Networking (UDP, TCP, WebSocket)
      PhysicsWorld2D.kt         # Physics simulation
      AnimationController.kt    # Sprite animation
      Errors.kt                 # Error types (generated from schema)
      Coroutines.kt             # Kotlin coroutine integration
    components/                 # ECS component types
      Transform2D.kt            # 2D position, rotation, scale
      Sprite.kt                 # Sprite rendering
      Text.kt                   # Text rendering
      SpriteAnimator.kt         # Animation state machine
    types/                      # Value types
      Vec2.kt, Vec3.kt          # Vector math
      Color.kt                  # RGBA color
      Rect.kt                   # Rectangle
      Mat3x3.kt                 # 3x3 matrix
    input/                      # Input enums
      Key.kt                    # Keyboard key constants
      MouseButton.kt            # Mouse button constants
    animation/                  # Animation enums
    physics/                    # Physics enums and types
    network/                    # Network protocol enums
  src/test/kotlin/com/goudengine/
    Vec2Test.kt, Vec3Test.kt    # Vector math tests
    ColorTest.kt                # Color tests
    Mat3x3Test.kt               # Matrix tests
    ErrorsTest.kt               # Error type tests
    ...
```

## JNI Architecture

The Kotlin SDK communicates with the Rust engine through the Java Native Interface (JNI):

1. **Java native declarations** (`src/main/java/.../internal/`) define `native` methods that map to Rust FFI functions.
2. **Kotlin wrapper classes** (`src/main/kotlin/.../core/`) provide idiomatic Kotlin APIs on top of the raw JNI calls.
3. **Native library loading** -- `GoudEngine.ensureLoaded()` loads `libgoud_engine.dylib` / `.so` / `.dll` from the jar's `native/` resources or from the system library path.

The Gradle build task (`buildNative`) compiles the Rust library and bundles it into the jar automatically.

## Installation

### From Source

```bash
# Build the native Rust library
cargo build --release -p goud-engine-core

# Generate SDK code
python3 codegen/gen_kotlin.py

# Build the jar
cd sdks/kotlin
./gradlew build --no-daemon
```

### Gradle Dependency (from Maven Central, when published)

```kotlin
dependencies {
    implementation("com.goudengine:goud-engine-kotlin:0.0.832")
}
```

## Quick Start

```kotlin
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

    val texture = game.loadTexture("assets/sprites/player.png")

    while (!game.shouldClose()) {
        game.beginFrame()

        game.drawSprite(texture, 400f, 300f, 64f, 64f, 0f, Color.white())

        game.endFrame()
    }

    game.destroy()
}
```

## Features

- 2D and 3D rendering with runtime renderer selection
- Entity Component System (ECS) with Transform2D, Sprite, Text
- Physics simulation (Rapier 2D) with rigid bodies and colliders
- Audio playback with per-channel volume and spatial audio
- Text rendering with TrueType fonts
- Sprite animation with state machine controller
- Networking (UDP, TCP, WebSocket)
- Kotlin coroutine integration for async operations
- Input handling (keyboard, mouse)
- Debugger runtime (snapshots, capture, replay, metrics)

## Build Commands

```bash
# Build everything (native + jar)
cd sdks/kotlin && ./gradlew build --no-daemon

# Run tests
cd sdks/kotlin && ./gradlew test --no-daemon

# Generate API docs
cd sdks/kotlin && ./gradlew dokkaHtml --no-daemon

# Run the Flappy Bird example
./dev.sh --sdk kotlin --game flappy_bird
```

## Code Generation

All JNI native declarations and most wrapper classes are auto-generated by `codegen/gen_kotlin.py` from the unified schema (`codegen/goud_sdk.schema.json`). Do not hand-edit generated files.

Regenerate:

```bash
python3 codegen/gen_kotlin.py
```

## Testing

```bash
cd sdks/kotlin
./gradlew test --no-daemon
```

Tests cover value types (Vec2, Vec3, Color, Mat3x3, Rect), error types, entity handles, enum parity, and coroutine helpers. Tests that require a live engine context are excluded from CI.

## Platform Support

| Platform | Library | Status |
|----------|---------|--------|
| macOS (x64/ARM64) | libgoud_engine.dylib | Supported |
| Linux (x64) | libgoud_engine.so | Supported |
| Windows (x64) | goud_engine.dll | Experimental |

Requires JDK 17 or later.
