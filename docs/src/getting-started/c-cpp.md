# C/C++ SDK

## Overview

The C/C++ SDK provides a header-only layer over the GoudEngine native library.
The C header (`goud/goud.h`) exposes type aliases and inline wrapper functions
that return integer status codes.  The C++ header (`goud/goud.hpp`) adds RAII
wrappers with move semantics and `noexcept` destructors.

Both headers depend on the generated FFI header (`goud_engine.h`) and the
pre-built native library for your platform.

## Installation

### vcpkg

```bash
vcpkg install goud-engine
```

Then in your `CMakeLists.txt`:

```cmake
find_package(GoudEngine CONFIG REQUIRED)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

### Conan

```bash
conan install --requires=goud-engine/0.0.832
```

Then in your `CMakeLists.txt`:

```cmake
find_package(GoudEngine REQUIRED)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

### Manual (tarball)

Download the tarball for your platform from the
[GitHub Releases](https://github.com/aram-devdocs/GoudEngine/releases) page.
Each tarball contains `lib/`, `include/`, and `cmake/` directories.

Extract and point CMake at the tarball root:

```bash
tar -xzf goud-engine-v0.0.832-linux-x64.tar.gz
```

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyGame LANGUAGES CXX)

set(GOUD_ENGINE_ROOT "/path/to/goud-engine-v0.0.832-linux-x64")
list(APPEND CMAKE_PREFIX_PATH "${GOUD_ENGINE_ROOT}/cmake")

find_package(GoudEngine CONFIG REQUIRED)

add_executable(my_game main.cpp)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

## CMake Setup

Regardless of the installation method, the CMake usage is the same:

```cmake
find_package(GoudEngine CONFIG REQUIRED)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

This provides the `GoudEngine::GoudEngine` imported target with all necessary
include paths and link libraries.

## C Hello World

```c
#include <goud/goud.h>
#include <stdio.h>

int main(void) {
    goud_engine_config config = NULL;
    goud_context ctx;
    int status;

    status = goud_engine_config_init(&config);
    if (status != SUCCESS) {
        fprintf(stderr, "config init failed: %d\n", status);
        return 1;
    }

    goud_engine_config_set_title_utf8(config, "Hello C");
    goud_engine_config_set_window_size(config, 800, 600);

    status = goud_engine_create_checked(&config, &ctx);
    if (status != SUCCESS) {
        fprintf(stderr, "engine create failed: %d\n", status);
        return 1;
    }

    goud_color clear = {0.2f, 0.3f, 0.4f, 1.0f};

    while (!goud_window_should_close_checked(ctx)) {
        goud_window_poll_events_checked(ctx);
        goud_renderer_begin_frame(ctx);
        goud_renderer_clear_color(ctx, clear);
        goud_renderer_end_frame(ctx);
        goud_window_swap_buffers_checked(ctx);
    }

    goud_context_dispose(&ctx);
    return 0;
}
```

## C++ Hello World

```cpp
#include <goud/goud.hpp>

int main() {
    auto config = goud::EngineConfig::create();
    config.setTitle("Hello C++");
    config.setSize(800, 600);

    auto engine = goud::Engine::create(std::move(config));
    engine.enableBlending();

    goud_color clear{0.2f, 0.3f, 0.4f, 1.0f};

    while (!engine.shouldClose()) {
        engine.pollEvents();
        engine.context().beginFrame();
        engine.context().clear(clear);
        engine.context().endFrame();
        engine.swapBuffers();
    }

    return 0;
}
```

## Error Handling

### C

Use `goud_get_last_error` to retrieve structured error information:

```c
goud_error_info err;
int code = goud_get_last_error(&err);
if (code != SUCCESS) {
    fprintf(stderr, "[%s] %s: %s\n", err.subsystem, err.operation, err.message);
}
```

### C++

Use `goud::Error::last()`:

```cpp
auto err = goud::Error::last();
if (err) {
    std::fprintf(stderr, "[%s] %s: %s\n",
        err.subsystem().c_str(),
        err.operation().c_str(),
        err.message().c_str());
}
```

## Next Steps

- [API Reference (Doxygen)](../api/c-cpp/index.html)
- [C++ SDK README](https://github.com/aram-devdocs/GoudEngine/tree/main/sdks/cpp)
