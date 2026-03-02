# GoudEngine

GoudEngine is a Rust game engine with multi-language SDK support. All game logic lives in Rust. SDKs for C#, Python, TypeScript, and Rust provide thin wrappers over the FFI boundary.

## SDK Support

| SDK | Package | Backend |
|-----|---------|---------|
| C# | [NuGet](https://www.nuget.org/packages/GoudEngine) | DllImport (P/Invoke) |
| Python | [PyPI](https://pypi.org/project/goudengine/) | ctypes |
| TypeScript | [npm](https://www.npmjs.com/package/goudengine) | napi-rs (Node.js) + wasm-bindgen (Web) |
| Rust | [crates.io](https://crates.io/crates/goud-engine) | Direct linking (no FFI) |

## Quick Links

- **New to GoudEngine?** Start with a getting-started guide for your language: [C#](getting-started/csharp.md), [Python](getting-started/python.md), [Rust](getting-started/rust.md), or [TypeScript](getting-started/typescript.md).
- **Building from source?** See the [Building](development/building.md) and [Development Guide](development/guide.md).
- **Understanding the internals?** Read the [SDK-First Architecture](architecture/sdk-first.md) document.

## Status

GoudEngine is in alpha. APIs change frequently. [Report issues on GitHub](https://github.com/aram-devdocs/GoudEngine/issues).
