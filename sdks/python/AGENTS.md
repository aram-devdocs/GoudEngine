# sdks/python/ — Python SDK

## Purpose

Python bindings for GoudEngine using ctypes-based FFI.

## Structure

- `goudengine/` — Python package with bindings and wrapper classes
- `test_bindings.py` — SDK test suite
- `README.md` — Python SDK documentation

## Running

```bash
python3 sdks/python/test_bindings.py                    # Run tests
./dev.sh --sdk python --game python_demo                # Run demo
./dev.sh --sdk python --game flappy_bird                # Run Flappy Bird
```

## Patterns

- ctypes loads the native `.dylib` (macOS) or `.so` (Linux)
- Function signatures declared with `argtypes` and `restype`
- Python classes wrap ctypes calls with Pythonic snake_case API
- MUST stay in parity with C# SDK

## After FFI Changes

1. Update `goudengine/generated/_ffi.py` with new ctypes function signatures
2. Add wrapper methods to appropriate Python classes
3. Run `test_bindings.py` to verify

## Anti-Patterns

- NEVER implement logic in Python — call FFI functions
- NEVER use Python types where ctypes C-compatible types are needed
- NEVER skip updating Python when C# SDK is updated (parity requirement)
