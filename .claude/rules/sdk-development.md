---
globs:
  - "sdks/**"
---

# SDK Development Rules

SDKs are thin wrappers over FFI. They translate Rust's C-ABI functions into idiomatic APIs for each target language.

## Thin Wrapper Principle

- SDKs call FFI functions — they NEVER implement game logic, math, physics, or rendering
- If you find logic in an SDK that should be in Rust, move it to Rust and expose via FFI
- DRY validation: search for duplicate logic between Rust core and SDK code

## Feature Parity

Every FFI export MUST have wrappers in BOTH:
- **C# SDK** (`sdks/GoudEngine/`) — DllImport declarations + wrapper classes
- **Python SDK** (`sdks/python/goud_engine/`) — ctypes declarations in `bindings.py` + wrapper classes

After adding new FFI functions, verify parity across both SDKs.

## Naming Conventions

| Language | Methods | Properties | Files |
|---|---|---|---|
| C# | PascalCase | PascalCase | PascalCase.cs |
| Python | snake_case | snake_case | snake_case.py |

## Testing

After SDK changes, run both:
- `dotnet test sdks/GoudEngine.Tests/` (C#)
- `python3 sdks/python/test_bindings.py` (Python)

## C# Specifics

- Targets .NET 8.0
- Uses `DllImport` for native library loading
- NuGet packaging via `./package.sh`
- Local NuGet feed at `$HOME/nuget-local`

## Python Specifics

- Uses `ctypes` for FFI bindings
- `bindings.py` declares all ctypes signatures (`argtypes`, `restype`)
- Handles library loading for macOS (`.dylib`) and Linux (`.so`)
