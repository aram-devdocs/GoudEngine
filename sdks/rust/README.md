# GoudEngine Rust SDK

[![crates.io](https://img.shields.io/crates/v/goud-engine.svg)](https://crates.io/crates/goud-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

> **Alpha** -- This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

Rust SDK for GoudEngine. Links directly against the engine with zero FFI overhead.

## Installation

```bash
cargo add goud-engine
```

## Quick Start

```rust
use goudengine::*;

fn main() {
    let engine = Engine::new(800, 600, "My Game");
    engine.enable_blending();

    while !engine.should_close() {
        let dt = engine.poll_events();
        engine.begin_frame();
        engine.clear(0.2, 0.2, 0.2, 1.0);
        // game logic here
        engine.end_frame();
        engine.swap_buffers();
    }
}
```

## Documentation

See the [Getting Started guide](../../docs/src/getting-started/rust.md) for installation, project setup, and examples.

## Testing

```bash
cargo test
```

## Architecture

Unlike the other SDKs which call through FFI, this crate re-exports `goud_engine::sdk::*` directly with zero overhead. A separate crate lets downstream projects depend on `goud-engine` without pulling in FFI exports, codegen build scripts, or napi dependencies.

## Links

- [Full documentation](../../docs/src/getting-started/rust.md)
- [Examples](https://github.com/aram-devdocs/GoudEngine/tree/main/examples/rust)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
