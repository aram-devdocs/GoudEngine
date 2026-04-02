# Xbox GDK Feasibility PoC -- Findings Report

**Issue**: #334 (parent: #135 Console Strategy)
**Date**: 2026-04-02
**Status**: Architecture validated, GDK SDK access required for runtime testing

## Executive Summary

GoudEngine's wgpu-based rendering architecture is structurally compatible with
Xbox GDK. The engine's `RenderBackend` trait abstraction, `PlatformBackend`
trait, and runtime backend selection already support plugging in new
platform/GPU combinations without modifying higher layers. A new `xbox-gdk`
feature flag enables Xbox-specific code paths while keeping them out of
standard builds.

**Verdict**: Feasible with caveats. The rendering path (wgpu DX12) aligns
well with Xbox's native GPU API. Blocking issues exist in audio, input, and
shader signing that require Xbox GDK SDK access to resolve.

## Architecture Integration

### What was built

- Feature flag: `xbox-gdk` in `goud_engine/Cargo.toml`
- Platform module: `XboxGdkPlatform` implementing `PlatformBackend`
- Window handle wrapper: `XboxWindowHandle` with `raw-window-handle` traits
  for wgpu DX12 surface creation
- wgpu initialization: `WgpuBackend::new_from_raw_handle()` forcing DX12
  backend
- Runtime integration: `(XboxGdk, Wgpu)` backend pair in `native_runtime.rs`

### What compiles today

`cargo check --features xbox-gdk` passes on macOS. The Xbox platform
constructor returns `BackendNotSupported` on non-MSVC targets, allowing
the full type system to be validated without the GDK SDK.

## wgpu DX12 on Xbox GDK

### Compatibility

wgpu v29 ships a DX12 backend (`wgpu::Backends::DX12`) that uses the
standard `d3d12.dll` API. Xbox GDK uses `d3d12_xs.dll` (Xbox Series) or
`d3d12_x.dll` (Xbox One), which are supersets of the desktop DX12 API with
console-specific extensions.

**Risk**: wgpu links against `d3d12.dll` at compile time. On Xbox, this
library does not exist -- only the `_xs`/`_x` variants are present. This
may require:
1. A wgpu build configuration that targets the Xbox DX12 variant
2. A thin shim that re-exports `d3d12_xs.dll` symbols as `d3d12.dll`
3. Contributing Xbox target support upstream to wgpu

**Severity**: High. This is the primary blocking issue.

### Surface Creation

The PoC implements `XboxWindowHandle` with `raw-window-handle` 0.6 traits
(via `wgpu::rwh::` re-exports). The HWND from Xbox GDK's `GameWindow` API
should work with wgpu's `SurfaceTarget::from()` since Xbox GDK supports the
Win32 HWND windowing model.

**Risk**: Low. The HWND path is well-tested in wgpu's Windows backend.

## Shader Pipeline

wgpu uses the naga shader compiler: WGSL source -> naga IR -> HLSL/DXIL
output. Xbox GDK requires DXIL bytecode, which naga can produce.

### DXIL Signing

Xbox retail builds require signed DXIL shaders. Development/devkit builds
allow unsigned DXIL. The PoC targets devkit mode.

For production, options include:
1. Using `dxcompiler.dll` (included in Xbox GDK) to compile and sign at
   build time
2. Pre-compiling shaders offline with `dxc.exe`
3. Contributing DXIL signing support to naga

**Severity**: Medium. Only affects retail builds, not the PoC.

### Shader Model

Xbox Series X/S supports Shader Model 6.6. naga targets SM 6.0 by default.
SM 6.0 is sufficient for the engine's current shader needs (no mesh shaders,
no ray tracing).

**Risk**: Low for current feature set.

## Windowing

Xbox GDK provides two windowing models:
1. **Win32 subset**: `CreateWindowExW` / `PeekMessage` / `DispatchMessage`
2. **GameWindow API**: `XGameWindowCreate` (higher-level, Xbox-specific)

Both provide an HWND. The PoC's `XboxGdkPlatform` is designed to use either
path. Xbox apps are always fullscreen; the platform backend correctly reports
this and rejects windowed mode.

**Risk**: Low. HWND acquisition is straightforward.

## Blocking Issues

| # | Issue | Severity | Mitigation |
|---|-------|----------|------------|
| 1 | wgpu links `d3d12.dll`, Xbox has `d3d12_xs.dll` | High | Shim DLL or upstream wgpu Xbox support |
| 2 | Audio: `rodio` uses CPAL which does not support Xbox audio APIs | High | Replace with Xbox `XAudio2` or `WASAPI` via FFI |
| 3 | Input: `winit` does not support Xbox controllers | Medium | Use Xbox `GameInput` API via FFI |
| 4 | DXIL signing for retail builds | Medium | Use GDK's `dxcompiler.dll` or offline compilation |
| 5 | GDK SDK not available in CI | Medium | Xbox builds manual-only until GDK CI licensing resolved |
| 6 | Networking: engine uses `std::net` which works on Xbox, but Xbox Live integration requires GDK APIs | Low | Defer to post-alpha |

## Audio Gap

`rodio` (the engine's audio backend) depends on CPAL, which uses CoreAudio
(macOS), WASAPI (Windows), or ALSA (Linux). Xbox GDK does not expose
WASAPI -- it uses XAudio2 or the lower-level `Windows.Media.Audio` API.

**Resolution**: Add an `xbox-audio` feature flag with an `XboxAudioBackend`
that calls XAudio2 via FFI. This is a separate work item (not in PoC scope).

## Input

The engine's `InputManager` maps keyboard/mouse/gamepad events from the
platform backend. Xbox GDK provides:
- `GameInput` API (recommended, supports all Xbox controllers)
- `XInputGetState` (legacy, supports standard gamepads)

The PoC's `XboxGdkPlatform::poll_events` is a stub. A full implementation
would call `GameInput` and map to the engine's input events.

**Resolution**: Extend `InputManager` with Xbox-specific gamepad mappings.
Separate work item.

## Build Toolchain

Xbox GDK compilation requires:
- Target: `x86_64-pc-windows-msvc`
- Environment: `GameDK` env var pointing to GDK install
- Linker search: `$(GameDK)\GRDK\GameKit\Lib\amd64`
- Link libraries: `xgameruntime.lib`

Rust cross-compilation from macOS to Windows MSVC is possible but non-trivial
(requires MSVC linker via `xwin` or a Windows CI runner).

## Certification Requirements

Xbox certification (XR requirements) that may affect engine architecture:
- **XR-015**: Must handle suspend/resume lifecycle events
- **XR-045**: Must support Quick Resume
- **XR-074**: Must not block the UI thread for >15 seconds
- **XR-130**: Must respect accessibility settings

These require engine-level hooks but do not block the rendering PoC.

## Recommended Next Steps

1. **Obtain Xbox GDK SDK access** and install on a Windows build machine
2. **Resolve DX12 linking** -- test whether wgpu's DX12 backend works with
   `d3d12_xs.dll` or needs a shim
3. **Implement actual window creation** in `XboxGdkPlatform::new_inner`
   using `CreateWindowExW`
4. **Test on devkit or GDK emulator** -- render a cleared framebuffer
5. **If successful**: plan audio, input, and certification work as separate
   issues for post-alpha
6. **If blocked on DX12 linking**: evaluate contributing Xbox support to
   wgpu upstream, or using raw DX12 as a backend (bypassing wgpu)
