# Cross-Platform Deployment

This guide covers shipping GoudEngine projects for macOS, Linux, Windows, and Web.

## Supported deployment targets

- macOS (`osx-x64`, `osx-arm64`)
- Linux (`linux-x64`)
- Windows (`win-x64`)
- Web (TypeScript + WASM)

## Toolchain requirements by platform

### macOS

- Rust toolchain
- .NET SDK 8+
- Python 3.11+
- Node.js 20+

Gotcha:

- On Apple Silicon, verify both `osx-x64` and `osx-arm64` runtime payloads.

```bash
file sdks/csharp/runtimes/osx-x64/native/libgoud_engine.dylib
file sdks/csharp/runtimes/osx-arm64/native/libgoud_engine.dylib
```

### Linux

- Rust toolchain
- Python 3.11+
- Node.js 20+
- GLFW/OpenGL runtime packages

Gotcha:

- Native runtime failures are usually missing system GL dependencies.

### Windows

- Rust toolchain
- .NET SDK 8+
- Python 3.11+
- Node.js 20+

Gotcha:

- Confirm native DLL placement in RID output folders before packaging.

### Web (TypeScript WASM)

- Node.js 20+
- Browser serving over HTTP

Gotchas:

- Do not run from `file://`.
- Publish loader JS and `.wasm` together.

## Local release build flow

```bash
./build.sh --release
./package.sh --local
./dev.sh --game flappy_goud --local
```

This validates a local release package install path before CI publishing.

## CI/CD deployment example (GitHub Actions)

This is a concrete workflow shape for build + artifact publish.

```yaml
name: deploy-matrix

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-dotnet@v4
        with:
          dotnet-version: '8.0.x'
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: cargo check
      - run: cargo test --workspace --quiet
      - run: python3 sdks/python/test_bindings.py
      - run: dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal
      - run: cd sdks/typescript && npm ci && npm test
      - run: bash scripts/check-generated-artifacts.sh
      - run: PATH="$HOME/.cargo/bin:$HOME/.dotnet/tools:$PATH" bash scripts/clean-room-regenerate.sh --docs

  package-desktop:
    needs: verify
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            artifact: linux-x64
          - os: macos-latest
            artifact: macos-universal
          - os: windows-latest
            artifact: win-x64
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: ./build.sh --release
      - uses: actions/upload-artifact@v4
        with:
          name: goudengine-${{ matrix.artifact }}
          path: |
            target/release
            sdks/csharp/runtimes
            sdks/nuget_package_output

  package-web:
    needs: verify
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: cd sdks/typescript && npm ci && npm run build:web
      - uses: actions/upload-artifact@v4
        with:
          name: goudengine-web
          path: |
            sdks/typescript/dist/web
            examples/typescript/flappy_bird/web
            examples/typescript/feature_lab/web
```

For this repository, canonical release automation remains:

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/docs.yml`

Use the example workflow above as the portable template, and treat the checked-in workflow files as the source of truth for this repository's actual release pipeline.

## WASM deployment checklist

- Host `goud_engine_bg.wasm` and generated JS in the same release payload.
- Serve all assets over HTTP(S).
- Keep import-map paths stable across local and deployed environments.

## Pre-release checklist

- `cargo check`
- `cargo test --workspace --quiet`
- `python3 sdks/python/test_bindings.py`
- `dotnet test sdks/csharp.tests/GoudEngine.Tests.csproj -v minimal`
- `(cd sdks/typescript && npm test)`
- `bash scripts/check-generated-artifacts.sh`
- `PATH="$HOME/.cargo/bin:$HOME/.dotnet/tools:$PATH" bash scripts/clean-room-regenerate.sh --docs`

Do not publish if any item fails.
