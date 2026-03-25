# Console Porting Guide

This guide covers the public side of a console port. It assumes you are an NDA
partner building the platform layer in a private repo. The public repository
provides the engine core, the generated C header, and the trait contracts you
need to plug in your own renderer.

## Scope

This document covers:

- downloading the static partner artifact
- linking the static library into a console build
- wiring platform hooks into the engine
- keeping the public and NDA-only boundaries clean

This document does not cover console SDK setup, certification tooling, or
private graphics API calls.

## Get the Partner Artifact

Each release publishes console-partner archives alongside the normal desktop
tarballs:

- `goud-engine-console-v<version>-linux-x64.tar.gz`
- `goud-engine-console-v<version>-osx-x64.tar.gz`
- `goud-engine-console-v<version>-osx-arm64.tar.gz`
- `goud-engine-console-v<version>-win-x64.tar.gz`

Extract the archive for the host machine you build on. The layout is fixed:

| Path | Linux/macOS | Windows |
|---|---|---|
| `lib/` | `lib/libgoud_engine.a` | `lib/goud_engine.lib` |
| `include/` | `include/goud_engine.h` | `include/goud_engine.h` |

The archive is minimal on purpose. It contains the static engine library and
the generated public header, nothing else.

## Link the Static Library

Point your console build at the extracted `include/` directory and link the
static library from `lib/`.

Generic example:

```text
INCLUDE_DIR=/path/to/goud-engine-console-v<version>-osx-arm64/include
LIB_DIR=/path/to/goud-engine-console-v<version>-osx-arm64/lib
```

- Add `INCLUDE_DIR` to the compiler include path.
- Add the library file from `LIB_DIR` to the linker inputs.
- Keep any platform SDK libraries in your private build scripts.

If your host toolchain is Windows, link `goud_engine.lib`. On Linux and macOS,
link `libgoud_engine.a`.

## Public Integration Points

The public repo already gives you three stable seams:

1. The generated C ABI in `goud_engine.h`
2. The provider traits in Rust, especially `RenderProvider`
3. The platform abstractions used by the engine runtime

Most console ports need a thin private crate or module that does four jobs:

- creates the platform window or surface
- owns the swap chain and command submission path
- implements `RenderProvider`
- feeds platform events into the engine loop

For the renderer contract, use
[Console Render Backend Contract](../architecture/console-render-provider.md).

## Porting Checklist

### Rendering

- Implement `RenderProvider` in a private backend crate.
- Translate engine descriptors into platform pipeline, buffer, texture, and
  render-target objects.
- Keep presentation inside `end_frame()`.
- Handle resize and lost-surface recovery in `resize()` and your private
  backend state.

### Platform hooks

- Create the console window, surface, or presentation target before
  `ProviderLifecycle::init()`.
- Feed platform input through the engine's input path instead of bypassing it.
- Keep thread-affinity rules in the private layer if the SDK requires them.

### Build and packaging

- Keep the public header untouched. Regenerate it from Rust when the public ABI
  changes.
- Treat the static library and header as a matched pair from the same release.
- Version your private integration layer separately from the public engine
  release if needed.

## Certification Watch List

These are common review areas. The details vary by platform, but the categories
do not.

### Memory

- avoid frame-to-frame leaks in transient GPU resources
- release buffers, textures, and render targets in a predictable place
- document any private allocator or pool usage that must be sized per title

### Threading

- keep SDK calls on the threads required by the platform
- do not move window or presentation ownership between threads unless the SDK
  says it is safe
- make fence and queue shutdown deterministic

### Audio

- verify suspend and resume behavior
- verify sample-rate and channel-layout expectations for the platform
- keep audio teardown separate from graphics teardown so one failure does not
  mask the other

### Public and private boundary

- keep NDA headers, libraries, and build files out of the public repo
- do not add private handles or SDK structs to public FFI signatures
- document platform assumptions in your private integration repo, not here

## What Stays Out of the Public Repo

Keep these items private:

- console SDK headers and libs
- device and swap-chain wrapper code
- certification scripts and checklists with platform-specific detail
- performance captures, debug markers, and SDK validation layers

The public repo should stay limited to trait contracts, the generated C header,
and release artifacts that are safe to publish.

## Suggested Bring-Up Order

1. Link the static library and confirm the title boots.
2. Stand up a `RenderProvider` that can clear the screen and present.
3. Add buffer, texture, and shader creation.
4. Add sprite draws, then text, then mesh and particle paths.
5. Fill in diagnostics after the frame loop is stable.

## Related Docs

- [Console Render Backend Contract](../architecture/console-render-provider.md)
- [Provider System](../architecture/providers.md)
- [C/C++ SDK](../getting-started/c-cpp.md)
