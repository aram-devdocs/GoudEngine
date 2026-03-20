# Getting Started — Swift SDK

> **Alpha** — GoudEngine is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

## Prerequisites

- Swift 5.9+ (ships with Xcode 15+, or install via [swift.org](https://www.swift.org/install/))
- macOS 13+ (Ventura or later)
- [Rust toolchain](https://rustup.rs/) — needed to build the native engine library

## Building the Native Library

The Swift SDK wraps the native Rust engine through its C FFI. Build the engine first:

```bash
git clone https://github.com/aram-devdocs/GoudEngine.git
cd GoudEngine
cargo build --release
```

This produces `target/release/libgoud_engine.dylib` (macOS) which the Swift package links against.

## Installation via SPM

Add GoudEngine as a local Swift Package Manager dependency. In your project's `Package.swift`:

```swift
// swift-tools-version: 5.9

import Foundation
import PackageDescription

// Point to the built native library.
// Override with GOUD_ENGINE_LIB_DIR for custom layouts.
let libSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_LIB_DIR"]
    ?? "../../target/release"

let package = Package(
    name: "MyGame",
    platforms: [
        .macOS(.v13),
    ],
    dependencies: [
        .package(path: "../../sdks/swift"),  // adjust relative path to GoudEngine repo
    ],
    targets: [
        .executableTarget(
            name: "MyGame",
            dependencies: [
                .product(name: "GoudEngine", package: "swift"),
            ],
            path: "Sources/MyGame",
            linkerSettings: [
                .unsafeFlags(["-L", libSearchPath]),
            ]
        ),
    ]
)
```

The `linkerSettings` tell the Swift compiler where to find `libgoud_engine.dylib`. Adjust the path to match your project layout relative to the GoudEngine repository root.

## First Project

Create `Sources/MyGame/main.swift` with a minimal window that closes on Escape:

```swift
import GoudEngine

let config = EngineConfig()
config
    .setSize(width: 800, height: 600)
    .setTitle(title: "My First Game")

let game = config.build()

while !game.shouldClose() {
    game.beginFrame(r: 0.2, g: 0.2, b: 0.2, a: 1.0)

    if game.isKeyPressed(key: .ESCAPE) {
        game.requestClose()
    }

    game.endFrame()
}
```

`beginFrame` clears the screen to the given color and prepares the frame. `endFrame` swaps buffers and polls events. `game.deltaTime` gives seconds since the last frame -- use it to keep movement frame-rate independent.

Build and run:

```bash
swift build && swift run
```

## Drawing a Sprite

Load textures once before the loop. Drawing happens between `beginFrame` and `endFrame`.

```swift
import GoudEngine

let config = EngineConfig()
config.setSize(width: 800, height: 600).setTitle(title: "Sprite Demo")
let game = config.build()

let texture = game.loadTexture(path: "assets/player.png")

while !game.shouldClose() {
    game.beginFrame(r: 0.1, g: 0.1, b: 0.1, a: 1.0)

    game.drawSprite(
        textureId: texture,
        x: 100, y: 100,
        width: 64, height: 64,
        rotation: 0,
        color: Color.white()
    )

    if game.isKeyPressed(key: .ESCAPE) {
        game.requestClose()
    }

    game.endFrame()
}
```

Put your image files in an `assets/` folder next to your project. The path is relative to the working directory when you run the executable.

## Handling Input

`isKeyPressed` returns true every frame the key is held. Use it for movement. For one-shot actions, track state yourself.

```swift
var x: Float = 400
var y: Float = 300
let speed: Float = 200

while !game.shouldClose() {
    game.beginFrame()
    let dt = game.deltaTime

    if game.isKeyPressed(key: .W) { y -= speed * dt }
    if game.isKeyPressed(key: .S) { y += speed * dt }
    if game.isKeyPressed(key: .A) { x -= speed * dt }
    if game.isKeyPressed(key: .D) { x += speed * dt }

    if game.isMouseButtonPressed(button: .LEFT) {
        // click action
    }

    let mouseX = game.mouseX()
    let mouseY = game.mouseY()

    game.endFrame()
}
```

## Running the Flappy Bird Example

The repository includes a complete Flappy Bird clone in Swift that mirrors the C# `flappy_goud` example for parity testing:

```bash
cd GoudEngine
cargo build --release
./dev.sh --sdk swift --game flappy_bird
```

Source is in [`examples/swift/flappy_bird/`](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/swift/flappy_bird/).

## Next Steps

- [Swift SDK source](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/swift/) — package layout and generated code
- [Swift examples source](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/swift/) — complete game source code
- [Build Your First Game](../guides/build-your-first-game.md) — end-to-end minimal game walkthrough
- [Example Showcase](../guides/showcase.md) — current cross-language parity matrix
- [SDK-first architecture](../architecture/sdk-first.md) — how the engine layers fit together
- [Development guide](../development/guide.md) — building from source, version management, git hooks
- Other getting started guides: [C#](csharp.md) · [Python](python.md) · [TypeScript](typescript.md) · [Rust](rust.md) · [Go](go.md) · [Kotlin](kotlin.md) · [Lua](lua.md)
