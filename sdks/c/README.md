# GoudEngine C SDK

> **Alpha** -- This SDK is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues)

Header-only C layer over GoudEngine's generated `goud_engine.h` FFI surface.

## Installation

Build the native library, then include the staged headers:

```bash
cargo build --release
```

```c
#include <goud/goud.h>
```

## Quick Start

```c
#include <goud/goud.h>

int main(void) {
    goud_engine_config* config = goud_engine_config_create();
    goud_engine_config_set_title(config, "My Game");
    goud_engine_config_set_size(config, 800, 600);

    goud_context ctx = goud_engine_config_build(config);
    // game loop here
    goud_context_destroy(ctx);
    return 0;
}
```

## Documentation

See the [Getting Started guide](../../docs/src/getting-started/c-cpp.md) for installation, CMake integration, and examples.

## Architecture

This SDK is a header-only convenience layer -- all engine logic lives in Rust. `goud_engine.h` is the source of truth for the raw ABI. The C SDK adds `static inline` helpers for common workflows.

## Links

- [Full documentation](../../docs/src/getting-started/c-cpp.md)
- [License: MIT](https://github.com/aram-devdocs/GoudEngine/blob/main/LICENSE)
