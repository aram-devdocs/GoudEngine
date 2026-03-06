# Adding a New Language Target

This guide walks through adding a new language binding (e.g., Lua, Go, Java) to GoudEngine. The existing generators are the best reference: `codegen/gen_csharp.py` for a complete, mature implementation and `codegen/gen_ts_web.py` for a simpler one.

Before starting, read [SDK-First Architecture](sdk-first.md) to understand the pipeline.

---

## Step 1: Understand the Schema

Read the two source-of-truth files:

- `codegen/goud_sdk.schema.json` — all types, enums, and tool methods that every SDK must expose
- `codegen/ffi_mapping.json` — the C ABI function names and signatures behind each schema method

The schema's `types` section defines value types (`Color`, `Vec2`, `Transform2D`) with fields and factory constructors. The `enums` section defines `Key`, `MouseButton`, and similar. The `tools` section defines `GoudGame` with a constructor, destructor, lifecycle methods (`beginFrame`, `endFrame`), and all game methods.

Your generator reads both files and emits code for each of these sections.

---

## Step 2: Add Type Mappings

Open `codegen/sdk_common.py` and add a type mapping table for your language. The existing tables show the pattern:

```python
# Example for a hypothetical language "Lua" (Lua has no static types,
# so this would map to annotation strings or documentation stubs)
LUA_TYPES = {
    "f32": "number", "f64": "number",
    "u8": "integer", "u16": "integer", "u32": "integer", "u64": "integer",
    "i8": "integer", "i16": "integer", "i32": "integer", "i64": "integer",
    "bool": "boolean", "string": "string", "void": "nil",
}
```

The schema uses type names like `f32`, `bool`, `string`, and composite names like `Color` or `Transform2D`. Your mapping table handles the primitives; the generator handles the composite types by looking them up in the schema.

Also add a ctypes-equivalent mapping if your language calls the native library through a C interop layer that requires explicit type annotations.

---

## Step 3: Create the Generator

Create `codegen/gen_<lang>.py`. The generator must produce at minimum:

1. **FFI declarations** — how to call the C functions from your language (e.g., `DllImport` in C#, `ctypes.CDLL` in Python, napi-rs bindings in TypeScript/Node)
2. **Value type wrappers** — classes or structs for `Color`, `Vec2`, `Vec3`, `Rect`, `Transform2D`, `Sprite`
3. **Enum definitions** — `Key` and `MouseButton` with their numeric values
4. **Tool wrappers** — the `GoudGame` class with all methods from `schema["tools"]["GoudGame"]`

### Generator structure

```python
#!/usr/bin/env python3
"""Generates the <Lang> SDK from the universal schema."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping,
    to_pascal, to_snake, write_generated,
)

OUT = SDKS_DIR / "<lang>"
schema = load_schema()
mapping = load_ffi_mapping()


def gen_types():
    # Emit value types from schema["types"]
    ...


def gen_enums():
    # Emit enums from schema["enums"]
    ...


def gen_game():
    # Emit GoudGame wrapper from schema["tools"]["GoudGame"]
    # and mapping["tools"]["GoudGame"]
    ...


if __name__ == "__main__":
    print("Generating <Lang> SDK...")
    gen_types()
    gen_enums()
    gen_game()
    print("<Lang> SDK generation complete.")
```

All output files MUST start with the `HEADER_COMMENT` constant so readers know not to edit them:

```python
lines = [f"// {HEADER_COMMENT}", ""]
```

Use `write_generated(path, content)` to write output files — it creates parent directories automatically and prints a status line.

### Mapping methods to FFI calls

For each method in `schema["tools"]["GoudGame"]["methods"]`, look up the corresponding entry in `mapping["tools"]["GoudGame"]["methods"]` to get the FFI function name:

```python
for method_name, method_def in schema["tools"]["GoudGame"]["methods"].items():
    ffi_fn = mapping["tools"]["GoudGame"]["methods"][method_name]["ffi"]
    # ffi_fn is a string like "goud_renderer_draw_sprite"
```

The `ffi_mapping.json` `lifecycle` section covers `beginFrame` and `endFrame`, which call multiple FFI functions in sequence. Check `mapping["tools"]["GoudGame"]["lifecycle"]` for those.

---

## Step 4: Add to codegen.sh

Add a step to `codegen.sh` between the existing generator steps and the final validation:

```bash
echo "║ [N/8] Generating <Lang> SDK..."
python3 codegen/gen_<lang>.py
```

Update the step count in the surrounding echo messages.

---

## Step 5: Create the SDK Directory

Create `sdks/<lang>/` with the package manifest for your language's package manager:

```
sdks/<lang>/
├── <package manifest>   # e.g., go.mod, build.gradle, rockspec
├── README.md
└── generated/           # files emitted by gen_<lang>.py
```

The SDK directory must contain at minimum one working example of how a game calls `GoudGame`. See `sdks/csharp/` and `sdks/python/` for the expected structure.

---

## Step 6: Add Tests

Add a test file that verifies the generated bindings load and the FFI calls round-trip correctly. At minimum, test:

- Library loads without error
- `GoudGame` constructor (or equivalent) initializes
- A type factory (e.g., `Color.red()`) returns the expected values

For Python the equivalent is `sdks/python/test_bindings.py`. Run it with:

```bash
python3 sdks/python/test_bindings.py
```

Add a corresponding command to the project `AGENTS.md` Essential Commands section.

---

## Step 7: Add an Example

Port an existing example to your new language. The simplest starting point is `examples/csharp/hello_ecs/`, which demonstrates ECS basics without physics or complex rendering.

For a fuller parity test, port `examples/csharp/flappy_goud/` — the Python SDK already has a matching `examples/python/flappy_bird.py`, so you can compare those two implementations side by side.

Place the example under `examples/<lang>/`.

---

## Checklist

Before merging a new language target:

- [ ] `codegen/sdk_common.py` has a type map for the new language
- [ ] `codegen/gen_<lang>.py` exists and runs without error
- [ ] All generated files begin with `HEADER_COMMENT`
- [ ] `codegen.sh` includes the generator step
- [ ] `sdks/<lang>/` has a package manifest
- [ ] `./codegen.sh` runs end-to-end cleanly
- [ ] Test file exists and passes
- [ ] At least one example game exists under `examples/<lang>/`
- [ ] `AGENTS.md` Essential Commands lists how to run the new SDK

---

## Reference: Existing Generators

| File | Target | Notes |
|---|---|---|
| `codegen/gen_csharp.py` | .NET 8 | Most complete; handles struct marshaling, builder pattern, `DllImport` |
| `codegen/gen_python.py` | Python 3 | Uses `ctypes`; reference for dynamic-type languages |
| `codegen/gen_ts_node.py` | TypeScript (Node) | Uses napi-rs; generates both Rust glue and TypeScript wrapper |
| `codegen/gen_ts_web.py` | TypeScript (Web) | Smallest generator; wraps a WASM module, no FFI declarations needed |

`gen_ts_web.py` is the simplest because it targets WASM — there are no ctypes or DllImport declarations. It wraps a pre-built WASM module handle directly. If your target also has a managed runtime that handles memory, this may be the closest analogue.

`gen_csharp.py` is the most detailed example of FFI struct mapping, null checking, and builder construction. Read it before writing a generator for a statically typed, natively-compiled language.
