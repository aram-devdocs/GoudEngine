# C++ SDK

The C++ SDK is a header-only C++17 layer in `namespace goud`.
It builds on the C SDK instead of calling raw Rust FFI exports directly.

The wrapper adds:

- `goud::Error` for last-error snapshots
- `goud::EngineConfig` with move-only ownership
- `goud::Context` and `goud::Engine` RAII wrappers with `noexcept` destructors
- optional `std::unique_ptr` and `std::shared_ptr` constructors where shared or transferred ownership is clearer than a raw handle

Build the native library first:

```bash
./build.sh --local --core-only --skip-csharp-sdk-build
```

Then include:

```cpp
#include <goud/goud.hpp>
```

The staged build copies the C wrapper header into the C++ include tree so the
packaged C++ SDK can stand on its own.
