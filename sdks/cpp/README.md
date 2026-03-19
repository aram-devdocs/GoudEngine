# C++ SDK

The C++ SDK is a header-only C++17 layer in `namespace goud`.
It builds on the C SDK instead of calling raw Rust FFI exports directly.

The wrapper adds:

- `goud::Error` for last-error snapshots
- `goud::EngineConfig` with move-only ownership
- `goud::Context` and `goud::Engine` RAII wrappers with `noexcept` destructors
- optional `std::unique_ptr` and `std::shared_ptr` constructors where shared or transferred ownership is clearer than a raw handle

## Package Manager Installation

### vcpkg

```bash
vcpkg install goud-engine
```

Then in your `CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyGame LANGUAGES CXX)

find_package(GoudEngine CONFIG REQUIRED)

add_executable(my_game main.cpp)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

### Conan

```bash
conan install --requires=goud-engine/<VERSION>
```

Then in your `CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyGame LANGUAGES CXX)

find_package(GoudEngine REQUIRED)

add_executable(my_game main.cpp)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

## Prerequisites

Build the native library first:

```bash
cargo build --release
```

## CMake Integration

### Using `find_package`

```cmake
cmake_minimum_required(VERSION 3.14)
project(MyGame LANGUAGES CXX)

set(GOUD_ENGINE_ROOT "/path/to/GoudEngine")
list(APPEND CMAKE_MODULE_PATH "${GOUD_ENGINE_ROOT}/sdks/cpp/cmake")

find_package(GoudEngine REQUIRED)

add_executable(my_game main.cpp)
target_link_libraries(my_game PRIVATE GoudEngine::GoudEngine)
```

If `GOUD_ENGINE_ROOT` is not set, the find module falls back to the repo root
relative to `sdks/cpp/cmake/`.

### Using `add_subdirectory`

```cmake
add_subdirectory(path/to/GoudEngine/sdks/cpp)
target_link_libraries(my_game PRIVATE GoudEngine)
```

### Building tests and examples

```bash
cmake -B build -DGOUD_BUILD_TESTS=ON -DGOUD_BUILD_EXAMPLES=ON sdks/cpp
cmake --build build
ctest --test-dir build
```

## Meson Integration

### As a subproject

Place `sdks/cpp/subprojects/goud-engine.wrap` in your project's `subprojects/`
directory, then:

```meson
goud_dep = dependency('goud-engine-cpp', fallback: ['goud-engine', 'goud_engine_dep'])
executable('my_game', 'main.cpp', dependencies: goud_dep)
```

### Direct dependency

```meson
goud_sub = subproject('goud-engine')
goud_dep = goud_sub.get_variable('goud_engine_dep')
```

## Quick Start

```cpp
#include <goud/goud.hpp>

int main() {
    auto config = goud::EngineConfig::create();
    config.setTitle("My Game");
    config.setSize(800, 600);

    auto engine = goud::Engine::create(std::move(config));
    engine.enableBlending();

    goud_color clear_color{0.2f, 0.3f, 0.4f, 1.0f};

    while (!engine.shouldClose()) {
        engine.pollEvents();
        engine.context().beginFrame();
        engine.context().clear(clear_color);
        engine.context().endFrame();
        engine.swapBuffers();
    }
    return 0;
}
```

## Documentation

API reference generated with Doxygen is available at the
[C/C++ API docs](https://aram-devdocs.github.io/GoudEngine/api/c-cpp/) page.
