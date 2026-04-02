# Getting Started

Pick the language you know best. Every SDK has the same capabilities -- the engine runs in Rust, and your SDK calls into it.

| Language | Best for | Install |
|----------|----------|---------|
| **[Rust](rust.md)** | Maximum performance, engine contributions | `cargo add goud-engine` |
| **[C#](csharp.md)** | Unity-like workflow, .NET ecosystem | `dotnet add package GoudEngine` |
| **[Python](python.md)** | Rapid prototyping, scripting | `pip install goudengine` |
| **[TypeScript](typescript.md)** | Web games (WASM), desktop via Node.js | `npm install goudengine` |
| **[C](c-cpp.md)** | Minimal overhead, embedded systems | Header-only |
| **[C++](c-cpp.md)** | RAII wrappers, existing C++ projects | CMake / vcpkg / Conan |
| **[Go](go.md)** | Simple concurrency, Go-native projects | `go get github.com/aram-devdocs/GoudEngine/sdks/go` |
| **[Kotlin](kotlin.md)** | JVM ecosystem, Android (future) | Gradle: `io.github.aram-devdocs:goudengine` |
| **[Swift](swift.md)** | Apple platforms, SwiftPM projects | Swift Package Manager |
| **[Lua](lua.md)** | Embedded scripting, mod support | `luarocks install goudengine` or embedded runner |

## What you get

Each guide walks you through the same steps:

1. **Prerequisites** -- what to install
2. **Install** -- one command to get the SDK
3. **Hello World** -- open a window
4. **Draw a Sprite** -- load and render an image
5. **Handle Input** -- respond to keyboard and mouse
6. **Run Examples** -- try the included demo games
7. **Next Steps** -- where to go from here

## How it works

All 10 SDKs are thin wrappers over the same Rust engine. Your game logic calls SDK functions, which call into Rust via FFI. This means:

- Identical behavior across all languages
- Bugs fixed once in Rust, fixed everywhere
- New features available in all SDKs simultaneously via codegen
