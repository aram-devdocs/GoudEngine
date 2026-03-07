# Dev Environment Setup

GoudEngine builds on Rust with SDK bindings for C#, Python, and TypeScript. The sections below cover system dependencies, toolchain installation, and verification.

## Prerequisites

| Tool | Version | Notes |
|---|---|---|
| Rust | stable (edition 2021) | Installed via rustup |
| .NET SDK | 8.0 | For C# SDK and examples |
| Python | 3.9+ (3.11 recommended) | For codegen scripts and Python SDK |
| Node.js | 16+ (20 recommended) | For TypeScript SDK |
| cbindgen | 0.29 | `cargo install cbindgen` |
| wasm-pack | latest | Only needed for TypeScript Web/WASM builds |
| cargo-deny | latest | `cargo install cargo-deny` |

Python and TypeScript SDK support is optional. Only Rust and the C# SDK are required for core development.

## System Dependencies

### macOS

Xcode Command Line Tools provide everything needed:

```sh
xcode-select --install
```

GLFW and OpenGL headers ship with macOS. Homebrew is not required for core development.

### Linux (Ubuntu/Debian)

```sh
sudo apt-get update
sudo apt-get install -y \
    build-essential cmake pkg-config \
    libgl1-mesa-dev libglu1-mesa-dev \
    libxrandr-dev libxinerama-dev libxcursor-dev \
    libxi-dev libxxf86vm-dev \
    libasound2-dev
```

Required to build GLFW and the audio subsystem.

> **Note:** `libglu1-mesa-dev` and `libxxf86vm-dev` are required by CI but are not installed by `install.sh`. Include them when setting up a fresh machine.

### Linux (Fedora)

```sh
sudo dnf install -y gcc gcc-c++ cmake pkgconfig \
    alsa-lib-devel libXrandr-devel libXinerama-devel \
    libXcursor-devel libXi-devel mesa-libGL-devel
```

### Linux (Arch)

```sh
sudo pacman -S --needed \
    base-devel cmake pkgconf alsa-lib \
    libxrandr libxinerama libxcursor libxi mesa
```

## Using install.sh

The repo includes `install.sh` to automate system dependency installation:

```sh
./install.sh
```

What it does:

1. Detects the OS (Linux distro or macOS)
2. Installs system libraries (OpenGL, X11, ALSA) via the native package manager
3. Installs .NET SDK 8.0 on Ubuntu/Debian if not already present
4. Installs Rust via rustup if `cargo` is not found
5. Installs cbindgen via `cargo install`

The script takes no flags. It runs unconditionally based on OS detection.

**Limitations:**

- Does not install Node.js, Python, wasm-pack, or cargo-deny
- Does not install `libglu1-mesa-dev` or `libxxf86vm-dev` (needed by CI)
- Does not cover .NET SDK installation on Fedora
- On macOS, relies on Xcode Command Line Tools being installed (prompts if missing)

Run the manual system dependency commands above even after running `install.sh` to match CI parity exactly.

## Step-by-Step Setup

1. **Clone the repo**

   ```sh
   git clone https://github.com/aram-devdocs/GoudEngine.git
   cd GoudEngine
   ```

2. **Install system dependencies** — run `./install.sh` or use the manual commands in the [System Dependencies](#system-dependencies) section above.

3. **Install Rust** (skip if `install.sh` already installed it — run `cargo --version` to check)

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
   source "$HOME/.cargo/env"
   ```

4. **Install Rust tools**

   ```sh
   cargo install cbindgen
   cargo install cargo-deny
   ```

5. **Verify the build**

   ```sh
   cargo check
   cargo build
   cargo test
   ```

6. **Install .NET SDK 8.0** (required for C# SDK work) — download from [dotnet.microsoft.com/download/dotnet/8.0](https://dotnet.microsoft.com/download/dotnet/8.0).

7. **Install Python 3.11+** (for codegen and Python SDK) — verify with `python3 --version`.

8. **Install Node.js 20** (for TypeScript SDK) — use [nvm](https://github.com/nvm-sh/nvm) or the [official installer](https://nodejs.org/en/download).

9. **(Optional) Install wasm-pack** (for TypeScript Web/WASM builds)

   ```sh
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

10. **Run a smoke test**

    ```sh
    cargo check
    cargo fmt --all -- --check
    cargo clippy -- -D warnings
    cargo test
    ```

## Verify Your Setup

All five commands must pass before pushing code:

```sh
cargo check                        # Type checks
cargo fmt --all -- --check         # Format check
cargo clippy -- -D warnings        # Lint check
cargo test                         # Unit tests
cargo deny check                   # Dependency audit
```

## Common Issues

### `glfw` build fails on Linux

Missing X11 or OpenGL dev libraries. Install the full set from the [Linux (Ubuntu/Debian)](#linux-ubuntudebian) section above.

### `libasound2-dev` not found (Ubuntu 24.04+)

Ubuntu 24.04 ships a transitional package. If `libasound2-dev` fails, try `libasound-dev` instead.

### `dotnet` command not found after install.sh

The .NET SDK install may need a new shell session to take effect. Close and reopen your terminal, or run `source ~/.bashrc`.

### `cargo test` fails with "failed to initialize any backend" or GL errors

Tests that need an OpenGL context fail without a display server. There are two approaches:

#### Option 1: NullBackend (recommended for unit/integration tests)

GoudEngine includes a headless `NullBackend` renderer that requires no GPU or display. Run tests that use it with:

```sh
cargo test --features headless --lib -- null
cargo test --test ffi_safety
cargo test --test integration
```

Use `goud_engine::test_helpers::init_test_context()` in your own tests to get a `NullBackend` instance.

#### Option 2: Xvfb (for tests that require a real GL context)

On headless Linux (CI, WSL without WSLg, SSH sessions), use Xvfb:

```sh
sudo apt-get install -y xvfb
xvfb-run -a cargo test
```

### Permission denied on install.sh

```sh
chmod +x install.sh
```

### cbindgen version mismatch

The project uses cbindgen 0.29. If you have an older version installed:

```sh
cargo install cbindgen --force
```

### Node.js native addon build fails (TypeScript SDK)

node-gyp requires a C++ compiler and Python 3:

```sh
# Ubuntu/Debian
sudo apt-get install -y g++ python3
# macOS — Xcode Command Line Tools covers this
```

## Optional Tools

| Tool | Purpose | Install |
|---|---|---|
| mdbook | Build and preview docs locally | `cargo install mdbook` |
| cargo-tarpaulin | Code coverage reports | `cargo install cargo-tarpaulin` |
| cargo-audit | Security vulnerability scanning | `cargo install cargo-audit` |
| GraphViz | Module dependency graph (`./graph.sh`) | `sudo apt install graphviz` / `brew install graphviz` |
| cargo-modules | Module dependency graph (`./graph.sh`) | `cargo install cargo-modules` |

## Next Steps

- [Development Guide](guide.md) for the day-to-day workflow
- [Building](building.md) for build commands and packaging
