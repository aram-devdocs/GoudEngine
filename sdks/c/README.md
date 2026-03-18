# C SDK

The C SDK is a header-only layer over the generated `goud_engine.h` surface.
It keeps the raw ABI available and adds a small set of `static inline` helpers
for common native workflows:

- status-code helpers with `goud_get_last_error`
- explicit ownership helpers for `goud_context` and `goud_engine_config`
- representative wrappers for entity, asset, renderer, input, and audio calls

Build the native library first:

```bash
./build.sh --local --core-only --skip-csharp-sdk-build
```

After that, include the staged headers from `sdks/c/include/`:

```c
#include <goud/goud.h>
```

`goud_engine.h` remains the source of truth for the raw ABI. The C SDK only
adds convenience wrappers on top of it.
