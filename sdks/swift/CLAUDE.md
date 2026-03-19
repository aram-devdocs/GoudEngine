# Swift SDK

Thin Swift wrapper over the GoudEngine C FFI. All game logic lives in Rust.

## Package layout

```
sdks/swift/
  Package.swift                      # SwiftPM manifest
  Sources/
    CGoudEngine/                     # System library target (C header)
      include/
        module.modulemap
        goud_engine.h                # Auto-copied C header
      shim.c
    GoudEngine/
      generated/                     # All .g.swift files from codegen
  Tests/
    GoudEngineTests/
```

## Codegen

All generated `.g.swift` files are produced by `codegen/gen_swift.py`.
Never hand-edit files inside `Sources/GoudEngine/generated/`.

Regenerate:
```bash
python3 codegen/gen_swift.py
```

## FFI conventions

- C header is imported via the `CGoudEngine` system library target.
- Tool classes (`GoudGame`, `EngineConfig`, etc.) use `deinit` with an `_alive` guard to prevent double-free.
- `GoudContextId` fields are accessed via `._0` (C struct import convention).
- String parameters are passed through `withCString { ptr in ... }`.
- Enum raw values use SCREAMING_SNAKE_CASE to match the C header.

## Adding as an SPM dependency

Projects that consume the Swift SDK add a local package dependency pointing at
`sdks/swift/`. The native library (`libgoud_engine.dylib` / `.so`) must be
built first via `cargo build --release`.

```swift
// In your game's Package.swift
dependencies: [
    .package(path: "../../sdks/swift"),  // adjust relative path
],
targets: [
    .executableTarget(
        name: "MyGame",
        dependencies: [
            .product(name: "GoudEngine", package: "GoudEngine"),
        ],
        linkerSettings: [
            .unsafeFlags(["-L", "../../target/release"]),
        ]
    ),
]
```

Set `GOUD_ENGINE_LIB_DIR` to override the library search path
(defaults to `../../target/release` relative to the SDK `Package.swift`).

## Testing

```bash
cd sdks/swift && swift test
```

Tests that require a live engine context are not included in the generated
test suite; only pure-Swift value type, enum, and error tests are generated.
