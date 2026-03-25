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

Every FFI export MUST have wrappers in all SDK languages under `sdks/`. Run `./codegen.sh` to regenerate all bindings and `python codegen/validate_coverage.py` to check coverage.

## Naming Conventions

| Language | Methods | Properties | Files |
|---|---|---|---|
| C# | PascalCase | PascalCase | PascalCase.cs |
| Python | snake_case | snake_case | snake_case.py |

## Testing

After SDK changes, run `./codegen.sh` to regenerate all bindings, then verify:
- `dotnet test sdks/csharp.tests/` (C#)
- `python3 sdks/python/test_bindings.py` (Python)
- `cd sdks/typescript && npm test` (TypeScript)

## C# Specifics

- Targets .NET 8.0
- Uses `DllImport` for native library loading
- NuGet packaging via `./package.sh`
- Local NuGet feed at `$HOME/nuget-local`

## Python Specifics

- Uses `ctypes` for FFI bindings
- `generated/_ffi.py` declares all ctypes signatures (`argtypes`, `restype`)
- Handles library loading for macOS (`.dylib`) and Linux (`.so`)

## Error Types

Error types are generated from the `errors` section of `codegen/goud_sdk.schema.json`.
Never hand-write error classes. The codegen generates:
- C#: `generated/Core/Errors.g.cs`
- Python: `generated/_errors.py`
- TypeScript: `src/generated/errors.g.ts`
