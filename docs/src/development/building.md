# Building GoudEngine

> **Alpha** — GoudEngine is under active development. APIs change frequently. [Report issues](https://github.com/aram-devdocs/GoudEngine/issues) · [Contact](mailto:aram.devdocs@gmail.com)

## Core Build Commands

```sh
cargo build                 # Debug build
cargo build --release       # Release build
cargo test                  # Run all tests
cargo test -- --nocapture   # Tests with output
```

## Release Build

`build.sh` compiles the engine in release mode and copies the native library into the SDK output directories:

```sh
./build.sh
./build.sh --release        # Explicit release flag
```

After running, the compiled `.dylib` / `.so` / `.dll` is placed in the SDK staging directories. The build also refreshes `codegen/generated/goud_engine.h`, copies that header into the C# and Python package staging paths, and regenerates the C# `NativeMethods.g.cs` surface.

## Packaging

`package.sh` creates a NuGet package from the built artifacts:

```sh
./package.sh                # Package to nuget_package_output/
./package.sh --local        # Package and push to local NuGet feed
```

The local NuGet feed is at `$HOME/nuget-local`. To consume it in an example project:

```sh
./dev.sh --game <game> --local
# or manually:
dotnet add package GoudEngine --version <version> --source $HOME/nuget-local
```

## SDK Tests

```sh
# C# SDK tests
dotnet test sdks/csharp.tests/

# Python SDK tests
python3 sdks/python/test_bindings.py

# TypeScript SDK tests
cd sdks/typescript && npm test
```

## Module Dependency Graph

Generate a visual graph of module dependencies:

```sh
./graph.sh
```

This creates `docs/diagrams/module_graph.png` and `.pdf` using `cargo modules` and GraphViz.
