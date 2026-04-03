# Nintendo Switch Vulkan Feasibility PoC -- Findings Report

**Issue**: #341 (parent: #135 Console Strategy)
**Date**: 2026-04-02
**Status**: Architecture validated, Nintendo SDK access required for runtime testing

## Executive Summary

GoudEngine's wgpu-based rendering architecture is structurally compatible with
Nintendo Switch's Vulkan 1.1 support. The engine's trait abstraction and runtime
backend selection support plugging in a Switch platform without modifying higher
layers. A `switch-vulkan` feature flag enables Switch-specific paths.

**Verdict**: Conditionally feasible. wgpu's Vulkan backend aligns with Switch's
Vulkan 1.1 support, but significant blockers exist around the proprietary SDK,
custom Rust target, and NVN-to-Vulkan translation layer performance.

## Architecture Integration

The following components were added under the `switch-vulkan` feature flag:

| Component | File | Purpose |
|-----------|------|---------|
| `SwitchVulkanPlatform` | `libs/platform/switch_vulkan_platform.rs` | `PlatformBackend` impl for Switch `nn::vi` windowing |
| `SwitchWindowHandle` | `libs/graphics/backend/wgpu_backend/switch_surface.rs` | `raw-window-handle` wrapper for wgpu surface creation |
| `switch_init` | `libs/graphics/backend/wgpu_backend/switch_init.rs` | wgpu Vulkan initialization from Switch handle |
| `WindowBackendKind::SwitchVulkan` | `libs/platform/mod.rs` | Enum variant for runtime backend selection |
| native_runtime arms | `libs/platform/native_runtime.rs` | Detection, validation, construction for Switch pair |

All components compile and type-check on macOS (x86_64 and aarch64). The
constructors return `BackendNotSupported` on non-aarch64 targets, allowing
cross-platform CI to verify the code without a Switch devkit.

## wgpu Vulkan on Nintendo Switch

### Compatibility

Nintendo Switch supports Vulkan 1.1 via a translation layer over NVN (Nintendo's
proprietary low-level API). wgpu's Vulkan backend targets Vulkan 1.0+ and should
work, but:
- The Switch's Vulkan driver has known conformance gaps
- Some Vulkan extensions used by wgpu may not be available
- Performance overhead from NVN translation is unknown

### Surface Creation

Switch uses `nn::vi` for display/layer management. The Vulkan surface is created
from an `nn::vi::Layer` handle, not from a standard window handle. This requires
a custom `raw-window-handle` implementation.

Risk: Medium. No standard windowing model -- requires manual `RawWindowHandle`
construction from `nn::vi` native window pointers.

## Shader Pipeline

GoudEngine uses naga for shader translation (GLSL -> WGSL -> SPIR-V). The Switch
Vulkan driver accepts SPIR-V, so the existing pipeline should work without
modification. However:
- naga's SPIR-V output should be tested against Switch's Vulkan conformance
- Some SPIR-V features may not be supported by the Switch driver
- Shader compilation performance on Switch hardware is unknown

## Blocking Issues

| # | Issue | Severity | Mitigation |
|---|-------|----------|------------|
| 1 | No official Rust target for Switch (`aarch64-nintendo-switch-none`) | High | Custom target JSON + tier 3 LLVM support |
| 2 | NintendoSDK is NDA-protected, not publicly available | High | Requires Nintendo developer program membership |
| 3 | Vulkan loader: Switch uses `nn::gfx` loader, not standard `libvulkan.so` | High | Custom Vulkan loader integration in wgpu |
| 4 | Audio: `nn::audio` API, not ALSA/PulseAudio | High | Custom audio backend via FFI |
| 5 | Input: `nn::hid` for Joy-Con/Pro Controller | Medium | Extend InputManager with Switch mappings |
| 6 | Performance: NVN-to-Vulkan translation overhead unknown | Medium | Benchmark once SDK available |
| 7 | File I/O: `nn::fs` for ROM/save data access | Medium | Custom asset loader path |

## Audio/Input/Filesystem Gaps

### Audio
The Switch uses `nn::audio` for sound output. GoudEngine's current audio backend
(rodio on desktop) would need a Switch-specific implementation wrapping `nn::audio`
via FFI. The `AudioManager` trait abstraction exists but has no Switch backend.

### Input
Switch input uses `nn::hid` for Joy-Con and Pro Controller support. The engine's
`InputManager` would need:
- Joy-Con button mapping (including gyro/accelerometer)
- Handheld vs. docked controller detection
- Touch screen support in handheld mode

### Filesystem
Switch uses `nn::fs` for ROM access and save data. The engine's `AssetServer`
would need a custom loader path that reads from the ROM filesystem via `nn::fs`
instead of standard `std::fs` operations.

## Build Toolchain

- Target: `aarch64-nintendo-switch-none` (custom target JSON)
- SDK: NintendoSDK (requires Nintendo developer program membership)
- Linker: Nintendo toolchain (part of SDK)
- Sysroot: SDK sysroot for aarch64
- Rust: Nightly required for custom target support

## Certification Requirements

Nintendo Lot Check requirements affecting engine:
- Must handle HOME button suspend/resume correctly
- Must support handheld/docked mode switching (720p to 1080p resolution change)
- Must meet performance requirements (30fps minimum)
- Must handle controller disconnect/reconnect gracefully
- Must support save data management via `nn::account` + `nn::fs`

## Estimated Development Effort

| Phase | Work | Weeks |
|-------|------|-------|
| SDK setup + custom Rust target | | 2-3 |
| Vulkan surface + basic rendering | | 3-4 |
| Audio + input integration | | 2-3 |
| File I/O + asset loading | | 1-2 |
| Performance optimization | | 2-3 |
| Lot Check compliance | | 2-3 |
| **Total** | | **12-18** |

## Recommended Next Steps

1. Join Nintendo developer program and obtain NintendoSDK
2. Create custom Rust target JSON for `aarch64-nintendo-switch`
3. Test wgpu Vulkan backend on Switch devkit
4. If Vulkan works: plan audio, input, filesystem as separate issues
5. If Vulkan blocked: evaluate NVN backend as alternative (requires writing custom wgpu backend)
