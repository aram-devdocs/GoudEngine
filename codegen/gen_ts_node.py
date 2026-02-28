#!/usr/bin/env python3
"""Generates the complete TypeScript Desktop (Node.js) SDK from the universal schema.

Produces:
  sdks/typescript/native/src/types.g.rs      -- Vec2, Vec3, Color, Rect + factory fns
  sdks/typescript/native/src/components.g.rs -- Transform2DData, SpriteData + factory fns
  sdks/typescript/native/src/entity.g.rs     -- Entity napi class
  sdks/typescript/native/src/game.g.rs       -- GameConfig, GoudGame class
  sdks/typescript/native/src/lib.rs          -- module declarations
  sdks/typescript/src/types/engine.g.ts      -- IGoudGame interface
  sdks/typescript/src/types/input.g.ts       -- Key, MouseButton enums
  sdks/typescript/src/types/math.g.ts        -- Color, Vec2, Vec3, Rect classes
  sdks/typescript/src/node/index.g.ts        -- GoudGame wrapper with run() loop
  sdks/typescript/src/index.g.ts             -- Entry point
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping,
    to_pascal, to_snake, to_camel, write_generated, TYPESCRIPT_TYPES,
)

TS = SDKS_DIR / "typescript"
GEN = TS / "src" / "generated"
NATIVE_SRC = TS / "native" / "src"
schema = load_schema()
mapping = load_ffi_mapping()


# Methods that already exist on the napi-rs NativeGoudGame class.
# New schema methods not in this set will use (this.native as any) until the
# native binding is extended.
NATIVE_KNOWN_METHODS = {
    "shouldClose", "close", "destroy", "beginFrame", "endFrame",
    "loadTexture", "destroyTexture", "drawSprite", "drawQuad",
    "isKeyPressed", "isKeyJustPressed", "isKeyJustReleased",
    "isMouseButtonPressed", "isMouseButtonJustPressed", "isMouseButtonJustReleased",
    "getMousePosition", "getMouseDelta", "getScrollDelta",
    "spawnEmpty", "spawnBatch", "despawn", "entityCount", "isAlive",
    "addTransform2D", "getTransform2D", "setTransform2D", "hasTransform2D", "removeTransform2D",
    "addSprite", "getSprite", "hasSprite", "removeSprite",
    "addName", "getName", "hasName", "removeName",
}

IFACE_TYPES = {
    "Entity": "IEntity",
    "Transform2D": "ITransform2DData",
    "Sprite": "ISpriteData",
    "Vec2": "IVec2",
    "Vec3": "IVec3",
    "Color": "IColor",
    "Rect": "IRect",
    "RenderStats": "IRenderStats",
    "Contact": "IContact",
    "Entity[]": "IEntity[]",
}


def ts_type(t: str) -> str:
    base = t.rstrip("?")
    mapped = TYPESCRIPT_TYPES.get(base, base)
    if t.endswith("?"):
        return f"{mapped} | null"
    return mapped


def ts_iface_type(t: str) -> str:
    """Map schema type to TypeScript interface type for IGoudGame."""
    base = t.rstrip("?")
    mapped = IFACE_TYPES.get(base, TYPESCRIPT_TYPES.get(base, base))
    if t.endswith("?"):
        return f"{mapped} | null"
    return mapped


# ── engine.g.ts (IGoudGame interface) ───────────────────────────────

def gen_interface():
    tool = schema["tools"]["GoudGame"]
    lines = [f"// {HEADER_COMMENT}", ""]

    lines.append("export interface IVec2 { x: number; y: number; }")
    lines.append("export interface IVec3 { x: number; y: number; z: number; }")
    lines.append("export interface IColor { r: number; g: number; b: number; a: number; }")
    lines.append("export interface IRect { x: number; y: number; width: number; height: number; }")
    lines.append("export interface IRenderStats { drawCalls: number; triangles: number; textureBinds: number; }")
    lines.append("export interface IContact { pointX: number; pointY: number; normalX: number; normalY: number; penetration: number; }")
    lines.append("")

    lines.append("export interface IEntity {")
    lines.append("  readonly index: number;")
    lines.append("  readonly generation: number;")
    lines.append("  readonly isPlaceholder: boolean;")
    lines.append("  toBits(): bigint;")
    lines.append("}")
    lines.append("")

    lines.append("export interface ITransform2DData {")
    for f in schema["types"]["Transform2D"]["fields"]:
        lines.append(f"  {to_camel(f['name'])}: number;")
    lines.append("}")
    lines.append("")

    lines.append("export interface ISpriteData {")
    for f in schema["types"]["Sprite"]["fields"]:
        fn = to_camel(f["name"])
        ft = f["type"]
        if ft == "Color":
            lines.append(f"  {fn}: IColor;")
        elif ft == "bool":
            lines.append(f"  {fn}: boolean;")
        elif ft in ("u64", "f32"):
            lines.append(f"  {fn}: number;")
        else:
            lines.append(f"  {fn}: {ts_iface_type(ft)};")
    lines.append("}")
    lines.append("")

    lines.append("export interface IGoudGame {")
    for prop in tool["properties"]:
        lines.append(f"  readonly {to_camel(prop['name'])}: {ts_iface_type(prop['type'])};")

    for method in tool["methods"]:
        mn = to_camel(method["name"])
        params = method.get("params", [])
        ret = method.get("returns", "void")

        # Determine which params can be optional: a param is only optional if it has
        # a default AND all params that follow it also have defaults (TypeScript rule).
        has_default = [p.get("default") is not None for p in params]
        can_be_optional = []
        trailing_all_defaulted = True
        for i in range(len(params) - 1, -1, -1):
            if not has_default[i]:
                trailing_all_defaulted = False
            can_be_optional.insert(0, trailing_all_defaulted and has_default[i])

        param_strs = []
        for i, p in enumerate(params):
            pn = to_camel(p["name"])
            pt = p["type"]
            opt = "?" if can_be_optional[i] else ""
            if pt == "callback(f32)":
                param_strs.append(f"{pn}: (dt: number) => void")
            elif pt in schema["types"]:
                ts_t = ts_iface_type(pt)
                param_strs.append(f"{pn}{opt}: {ts_t}")
            elif pt in schema["enums"]:
                param_strs.append(f"{pn}: number")
            else:
                ts_t = ts_iface_type(pt)
                param_strs.append(f"{pn}{opt}: {ts_t}")

        sig = ", ".join(param_strs)
        ts_ret = ts_iface_type(ret)
        if method.get("async"):
            lines.append(f"  {mn}({sig}): Promise<{ts_ret}>;")
        else:
            lines.append(f"  {mn}({sig}): {ts_ret};")

    lines.append("}")
    lines.append("")
    write_generated(GEN / "types" / "engine.g.ts", "\n".join(lines))


# ── input.g.ts ──────────────────────────────────────────────────────

def gen_input():
    lines = [f"// {HEADER_COMMENT}", ""]

    for enum_name, enum_def in schema["enums"].items():
        lines.append(f"export enum {enum_name} {{")
        for vname, vval in enum_def["values"].items():
            lines.append(f"  {vname} = {vval},")
        lines.append("}")
        lines.append("")

    write_generated(GEN / "types" / "input.g.ts", "\n".join(lines))


# ── math.g.ts ───────────────────────────────────────────────────────

def gen_math():
    lines = [f"// {HEADER_COMMENT}", "", "import type { IColor, IVec2, IVec3, IRect } from './engine.g';", ""]

    for type_name in ("Color", "Vec2", "Vec3", "Rect"):
        type_def = schema["types"][type_name]
        fields = type_def.get("fields", [])
        iface = f"I{type_name}"

        lines.append(f"export class {type_name} implements {iface} {{")

        ctor_params = ", ".join(f"public {to_camel(f['name'])}: number" for f in fields)
        lines.append(f"  constructor({ctor_params}) {{}}")
        lines.append("")

        for factory in type_def.get("factories", []):
            fn = to_camel(factory["name"])
            fargs = factory.get("args", [])
            val = factory.get("value")
            if val and not fargs:
                val_str = ", ".join(str(v) for v in val)
                lines.append(f"  static {fn}(): {type_name} {{ return new {type_name}({val_str}); }}")
            elif fargs:
                arg_str = ", ".join(f"{a['name']}: number" for a in fargs)
                pass_str = ", ".join(a["name"] for a in fargs)
                if fn == "fromHex":
                    lines.append(f"  static {fn}({arg_str}): {type_name} {{")
                    lines.append(f"    return new {type_name}(((hex >> 16) & 0xff) / 255, ((hex >> 8) & 0xff) / 255, (hex & 0xff) / 255, 1);")
                    lines.append("  }")
                elif fn == "rgb":
                    lines.append(f"  static {fn}({arg_str}): {type_name} {{ return new {type_name}({pass_str}, 1); }}")
                else:
                    lines.append(f"  static {fn}({arg_str}): {type_name} {{ return new {type_name}({pass_str}); }}")
        lines.append("")

        for meth in type_def.get("methods", []):
            mn = to_camel(meth["name"])
            if mn == "add":
                lines.append(f"  {mn}(other: {type_name}): {type_name} {{ return new {type_name}(this.x + other.x, this.y + other.y); }}")
            elif mn == "sub":
                lines.append(f"  {mn}(other: {type_name}): {type_name} {{ return new {type_name}(this.x - other.x, this.y - other.y); }}")
            elif mn == "scale":
                lines.append(f"  {mn}(s: number): {type_name} {{ return new {type_name}(this.x * s, this.y * s); }}")
            elif mn == "length":
                lines.append(f"  {mn}(): number {{ return Math.sqrt(this.x * this.x + this.y * this.y); }}")
            elif mn == "normalize":
                lines.append(f"  {mn}(): {type_name} {{ const l = this.length(); return l === 0 ? {type_name}.zero() : new {type_name}(this.x / l, this.y / l); }}")
            elif mn == "dot":
                lines.append(f"  {mn}(other: {type_name}): number {{ return this.x * other.x + this.y * other.y; }}")
            elif mn == "distance":
                lines.append(f"  {mn}(other: {type_name}): number {{ return this.sub(other).length(); }}")
            elif mn == "lerp":
                lines.append(f"  {mn}(other: {type_name}, t: number): {type_name} {{ return new {type_name}(this.x + (other.x - this.x) * t, this.y + (other.y - this.y) * t); }}")
            elif mn == "withAlpha":
                lines.append(f"  {mn}(a: number): {type_name} {{ return new {type_name}(this.r, this.g, this.b, a); }}")
            elif mn == "contains":
                lines.append(f"  {mn}(p: IVec2): boolean {{ return p.x >= this.x && p.x <= this.x + this.width && p.y >= this.y && p.y <= this.y + this.height; }}")
            elif mn == "intersects":
                lines.append(f"  {mn}(o: IRect): boolean {{ return this.x < o.x + o.width && this.x + this.width > o.x && this.y < o.y + o.height && this.y + this.height > o.y; }}")

        lines.append("}")
        lines.append("")

    write_generated(GEN / "types" / "math.g.ts", "\n".join(lines))


# ── node/index.g.ts ─────────────────────────────────────────────────

def gen_node_wrapper():
    tool = schema["tools"]["GoudGame"]
    tm = mapping["tools"]["GoudGame"]

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "import {",
        "  GoudGame as NativeGoudGame,",
        "  Entity as NativeEntity,",
        "  type GameConfig,",
        "} from '../../../index';",
        "",
        "import type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact } from '../types/engine.g';",
        "import { Color, Vec2, Vec3 } from '../types/math.g';",
        "export { Color, Vec2, Vec3 } from '../types/math.g';",
        "export { Key, MouseButton } from '../types/input.g';",
        "export type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact } from '../types/engine.g';",
        "",
        "export class GoudGame implements IGoudGame {",
        "  private native: NativeGoudGame;",
        "",
        "  constructor(config?: { width?: number; height?: number; title?: string }) {",
        "    this.native = new NativeGoudGame(config as GameConfig);",
        "  }",
        "",
        "  static async create(config?: { width?: number; height?: number; title?: string }): Promise<GoudGame> {",
        "    return new GoudGame(config);",
        "  }",
        "",
    ]

    for prop in tool["properties"]:
        pn = to_camel(prop["name"])
        lines.append(f"  get {pn}(): number {{ return this.native.{pn}; }}")
    lines.append("")

    for method in tool["methods"]:
        mn = to_camel(method["name"])
        mm = tm["methods"].get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")

        # Determine which params can be optional (TypeScript: optional only at tail).
        has_default = [p.get("default") is not None for p in params]
        can_be_optional = []
        trailing_all_defaulted = True
        for i in range(len(params) - 1, -1, -1):
            if not has_default[i]:
                trailing_all_defaulted = False
            can_be_optional.insert(0, trailing_all_defaulted and has_default[i])

        param_strs = []
        call_args = []
        for i, p in enumerate(params):
            pn = to_camel(p["name"])
            pt = p["type"]
            opt = "?" if can_be_optional[i] else ""
            if pt == "callback(f32)":
                param_strs.append(f"{pn}: (dt: number) => void")
                call_args.append(pn)
            elif pt in schema["types"]:
                param_strs.append(f"{pn}{opt}: {ts_iface_type(pt)}")
                call_args.append(pn)
            elif pt in schema["enums"]:
                param_strs.append(f"{pn}: number")
                call_args.append(pn)
            else:
                ts_t = ts_iface_type(pt)
                param_strs.append(f"{pn}{opt}: {ts_t}")
                call_args.append(pn)

        sig = ", ".join(param_strs)
        ts_ret = ts_iface_type(ret)

        if method.get("async"):
            lines.append(f"  async {mn}({sig}): Promise<{ts_ret}> {{")
        else:
            lines.append(f"  {mn}({sig}): {ts_ret} {{")

        if mn == "run":
            lines.append("    while (!this.native.shouldClose()) {")
            lines.append("      this.native.beginFrame();")
            lines.append("      update(this.native.deltaTime);")
            lines.append("      this.native.endFrame();")
            lines.append("    }")
        elif mn == "drawSprite":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    this.native.drawSprite(texture, x, y, width, height, rotation, c.r, c.g, c.b, c.a);")
        elif mn == "drawSpriteRect":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    return (this.native as any).drawSpriteRect(texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, c.r, c.g, c.b, c.a);")
        elif mn == "drawQuad":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    this.native.drawQuad(x, y, width, height, c.r, c.g, c.b, c.a);")
        elif mn in ("getMousePosition", "getMouseDelta", "getScrollDelta"):
            native_mn = mn
            lines.append(f"    const arr = this.native.{native_mn}();")
            lines.append("    return { x: arr[0], y: arr[1] };")
        elif mn == "spawnEmpty":
            lines.append("    return this.native.spawnEmpty() as unknown as IEntity;")
        elif mn == "spawnBatch":
            lines.append("    const arr = this.native.spawnBatch(count);")
            lines.append("    return Array.from(arr) as unknown as IEntity[];")
        elif mn == "despawn":
            lines.append("    return this.native.despawn(entity as unknown as NativeEntity);")
        elif mn == "despawnBatch":
            lines.append("    return (this.native as any).despawnBatch(entities as unknown as NativeEntity[]);")
        elif mn == "isAlive":
            lines.append("    return this.native.isAlive(entity as unknown as NativeEntity);")
        elif mn == "addTransform2d":
            lines.append("    this.native.addTransform2D(entity as unknown as NativeEntity, transform as any);")
        elif mn == "getTransform2d":
            lines.append("    return this.native.getTransform2D(entity as unknown as NativeEntity) ?? null;")
        elif mn == "setTransform2d":
            lines.append("    this.native.setTransform2D(entity as unknown as NativeEntity, transform as any);")
        elif mn == "hasTransform2d":
            lines.append("    return this.native.hasTransform2D(entity as unknown as NativeEntity);")
        elif mn == "removeTransform2d":
            lines.append("    return this.native.removeTransform2D(entity as unknown as NativeEntity);")
        elif mn == "addSprite":
            lines.append("    this.native.addSprite(entity as unknown as NativeEntity, sprite as any);")
        elif mn == "getSprite":
            lines.append("    const raw = this.native.getSprite(entity as unknown as NativeEntity);")
            lines.append("    if (!raw) return null;")
            lines.append("    return raw as unknown as ISpriteData;")
        elif mn == "setSprite":
            lines.append("    (this.native as any).setSprite(entity as unknown as NativeEntity, sprite as any);")
        elif mn == "hasSprite":
            lines.append("    return this.native.hasSprite(entity as unknown as NativeEntity);")
        elif mn == "removeSprite":
            lines.append("    return this.native.removeSprite(entity as unknown as NativeEntity);")
        elif mn == "addName":
            lines.append("    this.native.addName(entity as unknown as NativeEntity, name);")
        elif mn == "getName":
            lines.append("    return this.native.getName(entity as unknown as NativeEntity) ?? null;")
        elif mn == "hasName":
            lines.append("    return this.native.hasName(entity as unknown as NativeEntity);")
        elif mn == "removeName":
            lines.append("    return this.native.removeName(entity as unknown as NativeEntity);")
        elif mn == "getRenderStats":
            lines.append("    return (this.native as any).getRenderStats() as unknown as IRenderStats;")
        elif mn == "loadTexture":
            lines.append("    return this.native.loadTexture(path);")
        elif mn == "destroy":
            lines.append("    this.native.destroy();")
        else:
            native_call_args = ", ".join(call_args)
            native_mn = mn
            # Methods not yet on the native napi-rs binding use (as any) to allow future addition.
            native_obj = "this.native" if native_mn in NATIVE_KNOWN_METHODS else "(this.native as any)"
            if ret == "void":
                lines.append(f"    {native_obj}.{native_mn}({native_call_args});")
            else:
                lines.append(f"    return {native_obj}.{native_mn}({native_call_args});")

        lines.append("  }")
        lines.append("")

    lines.append("}")
    lines.append("")
    write_generated(GEN / "node" / "index.g.ts", "\n".join(lines))


# ── index.g.ts (entry point) ────────────────────────────────────────

def gen_entry():
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "export { GoudGame, Color, Vec2, Vec3, Key, MouseButton } from './node/index.g';",
        "export type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact } from './types/engine.g';",
        "export type { Rect } from './types/math.g';",
        "",
    ]
    write_generated(GEN / "index.g.ts", "\n".join(lines))


# ── napi-rs Rust code generation ─────────────────────────────────────

RUST_HEADER = f"// {HEADER_COMMENT}"

# Mapping from schema primitive types to napi-rs Rust types.
# All numeric JS values come in as f64; bool and String stay as-is.
NAPI_RUST_TYPES = {
    "f32": "f64",
    "f64": "f64",
    "u32": "u32",
    "u64": "f64",   # u64 values cross the boundary as f64 (BigInt avoided for plain handles)
    "i32": "i32",
    "i64": "f64",
    "bool": "bool",
    "string": "String",
    "void": "()",
}


def _napi_type(schema_type: str) -> str:
    """Map a schema type to a napi-rs Rust type for method signatures."""
    nullable = schema_type.endswith("?")
    base = schema_type.rstrip("?")
    mapped = NAPI_RUST_TYPES.get(base, base)
    if nullable:
        return f"Option<{mapped}>"
    return mapped


def gen_napi_rust_types():
    """Generate sdks/typescript/native/src/types.g.rs.

    Produces napi(object) structs for Vec2, Vec3, Color, Rect together with
    From impls for converting to/from the engine math types, plus standalone
    #[napi] factory functions for Color.
    """
    lines = [
        RUST_HEADER,
        "use goud_engine::core::math::{",
        "    Color as EngineColor, Rect as EngineRect, Vec2 as EngineVec2, Vec3 as EngineVec3,",
        "};",
        "use napi_derive::napi;",
        "",
    ]

    # ── struct + From impls ──────────────────────────────────────────
    struct_meta = {
        "Vec2":  {
            "engine": "EngineVec2",
            "fields": [("x", "f64"), ("y", "f64")],
            "from_engine": "Self { x: v.x as f64, y: v.y as f64 }",
            "to_engine": "Self::new(v.x as f32, v.y as f32)",
            "to_engine_var": "v",
        },
        "Vec3": {
            "engine": "EngineVec3",
            "fields": [("x", "f64"), ("y", "f64"), ("z", "f64")],
            "from_engine": "Self { x: v.x as f64, y: v.y as f64, z: v.z as f64 }",
            "to_engine": "Self::new(v.x as f32, v.y as f32, v.z as f32)",
            "to_engine_var": "v",
        },
        "Color": {
            "engine": "EngineColor",
            "fields": [("r", "f64"), ("g", "f64"), ("b", "f64"), ("a", "f64")],
            "from_engine": "Self { r: c.r as f64, g: c.g as f64, b: c.b as f64, a: c.a as f64 }",
            "to_engine": "Self::rgba(c.r as f32, c.g as f32, c.b as f32, c.a as f32)",
            "to_engine_var": "c",
        },
        "Rect": {
            "engine": "EngineRect",
            "fields": [("x", "f64"), ("y", "f64"), ("width", "f64"), ("height", "f64")],
            "from_engine": "Self { x: r.x as f64, y: r.y as f64, width: r.width as f64, height: r.height as f64 }",
            "to_engine": "Self::new(r.x as f32, r.y as f32, r.width as f32, r.height as f32)",
            "to_engine_var": "r",
        },
    }

    for name, meta in struct_meta.items():
        eng = meta["engine"]
        var = meta["to_engine_var"]

        lines.append("#[napi(object)]")
        lines.append("#[derive(Clone, Debug)]")
        lines.append(f"pub struct {name} {{")
        for fname, ftype in meta["fields"]:
            lines.append(f"    pub {fname}: {ftype},")
        lines.append("}")
        lines.append("")

        # From<Engine> for Napi
        lines.append(f"impl From<{eng}> for {name} {{")
        lines.append(f"    fn from({var}: {eng}) -> Self {{")
        lines.append(f"        {meta['from_engine']}")
        lines.append("    }")
        lines.append("}")
        lines.append("")

        # From<&Napi> for Engine
        lines.append(f"impl From<&{name}> for {eng} {{")
        lines.append(f"    fn from({var}: &{name}) -> Self {{")
        lines.append(f"        {meta['to_engine']}")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    # ── Color factory functions ─────────────────────────────────────
    # Derive from schema factories for Color
    color_schema = schema["types"]["Color"]
    for factory in color_schema.get("factories", []):
        fname = factory["name"]
        fargs = factory.get("args", [])
        val = factory.get("value")
        fn_name = f"color_{to_snake(fname)}"

        if fname == "rgba":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(r: f64, g: f64, b: f64, a: f64) -> Color {{")
            lines.append("    EngineColor::rgba(r as f32, g as f32, b as f32, a as f32).into()")
            lines.append("}")
        elif fname == "rgb":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(r: f64, g: f64, b: f64) -> Color {{")
            lines.append("    EngineColor::rgb(r as f32, g as f32, b as f32).into()")
            lines.append("}")
        elif fname == "fromHex":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(hex: u32) -> Color {{")
            lines.append("    EngineColor::from_hex(hex).into()")
            lines.append("}")
        elif val is not None and not fargs:
            # Named constant factory (white, black, red, …)
            const_name = to_snake(fname).upper()
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}() -> Color {{")
            lines.append(f"    EngineColor::{const_name}.into()")
            lines.append("}")
        lines.append("")

    write_generated(NATIVE_SRC / "types.g.rs", "\n".join(lines))


def gen_napi_rust_entity():
    """Generate sdks/typescript/native/src/entity.g.rs.

    Produces the Entity napi class that wraps the ECS entity handle.
    BigInt is used for from_bits / to_bits because entity IDs are u64.
    """
    lines = [
        RUST_HEADER,
        "use goud_engine::ecs::Entity as EcsEntity;",
        "use napi::bindgen_prelude::*;",
        "use napi_derive::napi;",
        "",
        "#[napi]",
        "pub struct Entity {",
        "    pub(crate) inner: EcsEntity,",
        "}",
        "",
        "#[napi]",
        "impl Entity {",
        "    #[napi(constructor)]",
        "    pub fn new(index: u32, generation: u32) -> Self {",
        "        Self {",
        "            inner: EcsEntity::new(index, generation),",
        "        }",
        "    }",
        "",
        "    #[napi(factory)]",
        "    pub fn placeholder() -> Self {",
        "        Self {",
        "            inner: EcsEntity::PLACEHOLDER,",
        "        }",
        "    }",
        "",
        "    #[napi(factory)]",
        "    pub fn from_bits(bits: BigInt) -> Result<Self> {",
        "        let (_, value, _) = bits.get_u64();",
        "        Ok(Self {",
        "            inner: EcsEntity::from_bits(value),",
        "        })",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn index(&self) -> u32 {",
        "        self.inner.index()",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn generation(&self) -> u32 {",
        "        self.inner.generation()",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn is_placeholder(&self) -> bool {",
        "        self.inner.is_placeholder()",
        "    }",
        "",
        "    #[napi]",
        "    pub fn to_bits(&self) -> BigInt {",
        "        BigInt::from(self.inner.to_bits())",
        "    }",
        "",
        "    #[napi]",
        "    pub fn display(&self) -> String {",
        '        format!("{}", self.inner)',
        "    }",
        "}",
        "",
    ]

    write_generated(NATIVE_SRC / "entity.g.rs", "\n".join(lines))


def gen_napi_rust_components():
    """Generate sdks/typescript/native/src/components.g.rs.

    Produces Transform2DData and SpriteData napi(object) structs together
    with their From impls, plus standalone #[napi] factory functions.
    """
    lines = [
        RUST_HEADER,
        "use crate::types::{Color, Vec2};",
        "use goud_engine::core::math::{Color as EngineColor, Vec2 as EngineVec2};",
        "use goud_engine::ecs::components::{Sprite, Transform2D};",
        "use napi_derive::napi;",
        "",
        "// =============================================================================",
        "// Transform2D",
        "// =============================================================================",
        "",
        "#[napi(object)]",
        "#[derive(Clone, Debug)]",
        "pub struct Transform2DData {",
        "    pub position_x: f64,",
        "    pub position_y: f64,",
        "    pub rotation: f64,",
        "    pub scale_x: f64,",
        "    pub scale_y: f64,",
        "}",
        "",
        "impl Default for Transform2DData {",
        "    fn default() -> Self {",
        "        Self {",
        "            position_x: 0.0,",
        "            position_y: 0.0,",
        "            rotation: 0.0,",
        "            scale_x: 1.0,",
        "            scale_y: 1.0,",
        "        }",
        "    }",
        "}",
        "",
        "impl From<&Transform2D> for Transform2DData {",
        "    fn from(t: &Transform2D) -> Self {",
        "        Self {",
        "            position_x: t.position.x as f64,",
        "            position_y: t.position.y as f64,",
        "            rotation: t.rotation as f64,",
        "            scale_x: t.scale.x as f64,",
        "            scale_y: t.scale.y as f64,",
        "        }",
        "    }",
        "}",
        "",
        "impl From<&Transform2DData> for Transform2D {",
        "    fn from(data: &Transform2DData) -> Self {",
        "        Transform2D {",
        "            position: EngineVec2::new(data.position_x as f32, data.position_y as f32),",
        "            rotation: data.rotation as f32,",
        "            scale: EngineVec2::new(data.scale_x as f32, data.scale_y as f32),",
        "        }",
        "    }",
        "}",
        "",
        "// =============================================================================",
        "// Sprite",
        "// =============================================================================",
        "",
        "#[napi(object)]",
        "#[derive(Clone, Debug)]",
        "pub struct SpriteData {",
        "    pub color: Color,",
        "    pub flip_x: bool,",
        "    pub flip_y: bool,",
        "    pub anchor_x: f64,",
        "    pub anchor_y: f64,",
        "    pub custom_width: Option<f64>,",
        "    pub custom_height: Option<f64>,",
        "    pub source_rect_x: Option<f64>,",
        "    pub source_rect_y: Option<f64>,",
        "    pub source_rect_width: Option<f64>,",
        "    pub source_rect_height: Option<f64>,",
        "}",
        "",
        "impl From<&Sprite> for SpriteData {",
        "    fn from(s: &Sprite) -> Self {",
        "        let (custom_width, custom_height) = match s.custom_size {",
        "            Some(size) => (Some(size.x as f64), Some(size.y as f64)),",
        "            None => (None, None),",
        "        };",
        "        let (src_x, src_y, src_w, src_h) = match s.source_rect {",
        "            Some(rect) => (",
        "                Some(rect.x as f64),",
        "                Some(rect.y as f64),",
        "                Some(rect.width as f64),",
        "                Some(rect.height as f64),",
        "            ),",
        "            None => (None, None, None, None),",
        "        };",
        "        Self {",
        "            color: s.color.into(),",
        "            flip_x: s.flip_x,",
        "            flip_y: s.flip_y,",
        "            anchor_x: s.anchor.x as f64,",
        "            anchor_y: s.anchor.y as f64,",
        "            custom_width,",
        "            custom_height,",
        "            source_rect_x: src_x,",
        "            source_rect_y: src_y,",
        "            source_rect_width: src_w,",
        "            source_rect_height: src_h,",
        "        }",
        "    }",
        "}",
        "",
        "impl From<&SpriteData> for Sprite {",
        "    fn from(data: &SpriteData) -> Self {",
        "        Sprite {",
        "            color: EngineColor::from(&data.color),",
        "            flip_x: data.flip_x,",
        "            flip_y: data.flip_y,",
        "            anchor: EngineVec2::new(data.anchor_x as f32, data.anchor_y as f32),",
        "            custom_size: match (data.custom_width, data.custom_height) {",
        "                (Some(w), Some(h)) => Some(EngineVec2::new(w as f32, h as f32)),",
        "                _ => None,",
        "            },",
        "            source_rect: match (",
        "                data.source_rect_x,",
        "                data.source_rect_y,",
        "                data.source_rect_width,",
        "                data.source_rect_height,",
        "            ) {",
        "                (Some(x), Some(y), Some(w), Some(h)) => {",
        "                    Some(goud_engine::core::math::Rect::new(",
        "                        x as f32, y as f32, w as f32, h as f32,",
        "                    ))",
        "                }",
        "                _ => None,",
        "            },",
        "            ..Default::default()",
        "        }",
        "    }",
        "}",
        "",
        "// =============================================================================",
        "// Factory functions for Transform2D",
        "// =============================================================================",
        "",
        "#[napi]",
        "pub fn transform2d_default() -> Transform2DData {",
        "    Transform2DData::default()",
        "}",
        "",
        "#[napi]",
        "pub fn transform2d_from_position(x: f64, y: f64) -> Transform2DData {",
        "    Transform2DData {",
        "        position_x: x,",
        "        position_y: y,",
        "        ..Transform2DData::default()",
        "    }",
        "}",
        "",
        "#[napi]",
        "pub fn transform2d_from_scale(x: f64, y: f64) -> Transform2DData {",
        "    Transform2DData {",
        "        scale_x: x,",
        "        scale_y: y,",
        "        ..Transform2DData::default()",
        "    }",
        "}",
        "",
        "#[napi]",
        "pub fn transform2d_from_rotation(radians: f64) -> Transform2DData {",
        "    Transform2DData {",
        "        rotation: radians,",
        "        ..Transform2DData::default()",
        "    }",
        "}",
        "",
        "// Factory functions for Sprite",
        "#[napi]",
        "pub fn sprite_default() -> SpriteData {",
        "    let sprite = Sprite::default();",
        "    SpriteData::from(&sprite)",
        "}",
        "",
        "// Factory functions for Vec2 convenience",
        "#[napi]",
        "pub fn vec2(x: f64, y: f64) -> Vec2 {",
        "    Vec2 { x, y }",
        "}",
        "",
        "#[napi]",
        "pub fn vec2_zero() -> Vec2 {",
        "    Vec2 { x: 0.0, y: 0.0 }",
        "}",
        "",
        "#[napi]",
        "pub fn vec2_one() -> Vec2 {",
        "    Vec2 { x: 1.0, y: 1.0 }",
        "}",
        "",
    ]

    write_generated(NATIVE_SRC / "components.g.rs", "\n".join(lines))


def gen_napi_rust_game():
    """Generate sdks/typescript/native/src/game.g.rs.

    Produces GameConfig and the GoudGame napi class with all lifecycle,
    rendering, input, and ECS methods.  The generated output is functionally
    identical to the hand-written game.rs.
    """
    lines = [
        RUST_HEADER,
        "use crate::components::{SpriteData, Transform2DData};",
        "use crate::entity::Entity;",
        "use goud_engine::ecs::components::{Name, Sprite, Transform2D};",
        "use goud_engine::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};",
        "use goud_engine::ffi::input::{",
        "    goud_input_get_mouse_delta, goud_input_get_mouse_position, goud_input_get_scroll_delta,",
        "    goud_input_key_just_pressed, goud_input_key_just_released, goud_input_key_pressed,",
        "    goud_input_mouse_button_just_pressed, goud_input_mouse_button_just_released,",
        "    goud_input_mouse_button_pressed,",
        "};",
        "use goud_engine::ffi::renderer::{",
        "    goud_renderer_begin, goud_renderer_draw_quad, goud_renderer_draw_sprite,",
        "    goud_renderer_enable_blending, goud_renderer_end, goud_texture_destroy, goud_texture_load,",
        "};",
        "use goud_engine::ffi::window::{",
        "    goud_window_clear, goud_window_create, goud_window_destroy, goud_window_get_delta_time,",
        "    goud_window_poll_events, goud_window_set_should_close, goud_window_should_close,",
        "    goud_window_swap_buffers,",
        "};",
        "use goud_engine::sdk::{GameConfig as EngineGameConfig, GoudGame as EngineGoudGame};",
        "use napi::bindgen_prelude::*;",
        "use napi_derive::napi;",
        "use std::ffi::CString;",
        "",
        "// =============================================================================",
        "// GameConfig",
        "// =============================================================================",
        "",
        "#[napi(object)]",
        "#[derive(Clone, Debug)]",
        "pub struct GameConfig {",
        "    pub title: Option<String>,",
        "    pub width: Option<u32>,",
        "    pub height: Option<u32>,",
        "    pub vsync: Option<bool>,",
        "    pub fullscreen: Option<bool>,",
        "    pub resizable: Option<bool>,",
        "    pub target_fps: Option<u32>,",
        "    pub debug_rendering: Option<bool>,",
        "}",
        "",
        "impl From<&GameConfig> for EngineGameConfig {",
        "    fn from(cfg: &GameConfig) -> Self {",
        "        let defaults = EngineGameConfig::default();",
        "        EngineGameConfig {",
        "            title: cfg.title.clone().unwrap_or(defaults.title),",
        "            width: cfg.width.unwrap_or(defaults.width),",
        "            height: cfg.height.unwrap_or(defaults.height),",
        "            vsync: cfg.vsync.unwrap_or(defaults.vsync),",
        "            fullscreen: cfg.fullscreen.unwrap_or(defaults.fullscreen),",
        "            resizable: cfg.resizable.unwrap_or(defaults.resizable),",
        "            target_fps: cfg.target_fps.unwrap_or(defaults.target_fps),",
        "            debug_rendering: cfg.debug_rendering.unwrap_or(defaults.debug_rendering),",
        "        }",
        "    }",
        "}",
        "",
        "// =============================================================================",
        "// GoudGame",
        "// =============================================================================",
        "",
        "#[napi]",
        "pub struct GoudGame {",
        "    inner: EngineGoudGame,",
        "    context_id: GoudContextId,",
        "    last_delta_time: f32,",
        "}",
        "",
        "#[napi]",
        "impl GoudGame {",
        "    #[napi(constructor)]",
        "    pub fn new(config: Option<GameConfig>) -> Result<Self> {",
        "        let engine_config = match &config {",
        "            Some(cfg) => EngineGameConfig::from(cfg),",
        "            None => EngineGameConfig::default(),",
        "        };",
        "",
        "        let width = config.as_ref().and_then(|c| c.width).unwrap_or(800);",
        "        let height = config.as_ref().and_then(|c| c.height).unwrap_or(600);",
        "        let title_str = config",
        "            .as_ref()",
        '            .and_then(|c| c.title.clone())',
        '            .unwrap_or_else(|| "GoudEngine".to_string());',
        "",
        "        let c_title = CString::new(title_str)",
        '            .map_err(|e| Error::from_reason(format!("Invalid title string: {}", e)))?;',
        "",
        "        // SAFETY: CString guarantees a valid null-terminated pointer.",
        "        let context_id = unsafe { goud_window_create(width, height, c_title.as_ptr()) };",
        "        if context_id == GOUD_INVALID_CONTEXT_ID {",
        '            return Err(Error::from_reason("Failed to create GLFW window"));',
        "        }",
        "",
        "        let game =",
        '            EngineGoudGame::new(engine_config).map_err(|e| Error::from_reason(format!("{}", e)))?;',
        "",
        "        Ok(Self {",
        "            inner: game,",
        "            context_id,",
        "            last_delta_time: 0.0,",
        "        })",
        "    }",
        "",
        "    // =========================================================================",
        "    // Lifecycle",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn should_close(&self) -> bool {",
        "        goud_window_should_close(self.context_id)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn close(&self) {",
        "        goud_window_set_should_close(self.context_id, true);",
        "    }",
        "",
        "    #[napi]",
        "    pub fn destroy(&self) -> bool {",
        "        goud_window_destroy(self.context_id)",
        "    }",
        "",
        "    // =========================================================================",
        "    // Frame Management",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn begin_frame(&mut self, r: Option<f64>, g: Option<f64>, b: Option<f64>, a: Option<f64>) {",
        "        let dt = goud_window_poll_events(self.context_id);",
        "        self.last_delta_time = dt;",
        "        goud_window_clear(",
        "            self.context_id,",
        "            r.unwrap_or(0.0) as f32,",
        "            g.unwrap_or(0.0) as f32,",
        "            b.unwrap_or(0.0) as f32,",
        "            a.unwrap_or(1.0) as f32,",
        "        );",
        "        goud_renderer_begin(self.context_id);",
        "        goud_renderer_enable_blending(self.context_id);",
        "    }",
        "",
        "    #[napi]",
        "    pub fn end_frame(&self) {",
        "        goud_renderer_end(self.context_id);",
        "        goud_window_swap_buffers(self.context_id);",
        "    }",
        "",
        "    // =========================================================================",
        "    // Rendering",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn load_texture(&self, path: String) -> Result<f64> {",
        "        let c_path =",
        '            CString::new(path).map_err(|e| Error::from_reason(format!("Invalid path: {}", e)))?;',
        "        // SAFETY: CString guarantees a valid null-terminated pointer.",
        "        let handle = unsafe { goud_texture_load(self.context_id, c_path.as_ptr()) };",
        "        Ok(handle as f64)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn destroy_texture(&self, handle: f64) -> bool {",
        "        goud_texture_destroy(self.context_id, handle as u64)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn draw_sprite(",
        "        &self,",
        "        texture: f64,",
        "        x: f64,",
        "        y: f64,",
        "        w: f64,",
        "        h: f64,",
        "        rotation: Option<f64>,",
        "        r: Option<f64>,",
        "        g: Option<f64>,",
        "        b: Option<f64>,",
        "        a: Option<f64>,",
        "    ) -> bool {",
        "        goud_renderer_draw_sprite(",
        "            self.context_id,",
        "            texture as u64,",
        "            x as f32,",
        "            y as f32,",
        "            w as f32,",
        "            h as f32,",
        "            rotation.unwrap_or(0.0) as f32,",
        "            r.unwrap_or(1.0) as f32,",
        "            g.unwrap_or(1.0) as f32,",
        "            b.unwrap_or(1.0) as f32,",
        "            a.unwrap_or(1.0) as f32,",
        "        )",
        "    }",
        "",
        "    #[napi]",
        "    pub fn draw_quad(",
        "        &self,",
        "        x: f64,",
        "        y: f64,",
        "        w: f64,",
        "        h: f64,",
        "        r: Option<f64>,",
        "        g: Option<f64>,",
        "        b: Option<f64>,",
        "        a: Option<f64>,",
        "    ) -> bool {",
        "        goud_renderer_draw_quad(",
        "            self.context_id,",
        "            x as f32,",
        "            y as f32,",
        "            w as f32,",
        "            h as f32,",
        "            r.unwrap_or(1.0) as f32,",
        "            g.unwrap_or(1.0) as f32,",
        "            b.unwrap_or(1.0) as f32,",
        "            a.unwrap_or(1.0) as f32,",
        "        )",
        "    }",
        "",
        "    // =========================================================================",
        "    // Input",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn is_key_pressed(&self, key: i32) -> bool {",
        "        goud_input_key_pressed(self.context_id, key)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn is_key_just_pressed(&self, key: i32) -> bool {",
        "        goud_input_key_just_pressed(self.context_id, key)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn is_key_just_released(&self, key: i32) -> bool {",
        "        goud_input_key_just_released(self.context_id, key)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn is_mouse_button_pressed(&self, button: i32) -> bool {",
        "        goud_input_mouse_button_pressed(self.context_id, button)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn is_mouse_button_just_pressed(&self, button: i32) -> bool {",
        "        goud_input_mouse_button_just_pressed(self.context_id, button)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn is_mouse_button_just_released(&self, button: i32) -> bool {",
        "        goud_input_mouse_button_just_released(self.context_id, button)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn get_mouse_position(&self) -> Vec<f64> {",
        "        let mut x: f32 = 0.0;",
        "        let mut y: f32 = 0.0;",
        "        // SAFETY: Passing valid mutable references as out-pointers.",
        "        unsafe { goud_input_get_mouse_position(self.context_id, &mut x, &mut y) };",
        "        vec![x as f64, y as f64]",
        "    }",
        "",
        "    #[napi]",
        "    pub fn get_mouse_delta(&self) -> Vec<f64> {",
        "        let mut dx: f32 = 0.0;",
        "        let mut dy: f32 = 0.0;",
        "        // SAFETY: Passing valid mutable references as out-pointers.",
        "        unsafe { goud_input_get_mouse_delta(self.context_id, &mut dx, &mut dy) };",
        "        vec![dx as f64, dy as f64]",
        "    }",
        "",
        "    #[napi]",
        "    pub fn get_scroll_delta(&self) -> Vec<f64> {",
        "        let mut dx: f32 = 0.0;",
        "        let mut dy: f32 = 0.0;",
        "        // SAFETY: Passing valid mutable references as out-pointers.",
        "        unsafe { goud_input_get_scroll_delta(self.context_id, &mut dx, &mut dy) };",
        "        vec![dx as f64, dy as f64]",
        "    }",
        "",
        "    // =========================================================================",
        "    // Entity Operations (ECS)",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn spawn_empty(&mut self) -> Entity {",
        "        Entity {",
        "            inner: self.inner.spawn_empty(),",
        "        }",
        "    }",
        "",
        "    #[napi]",
        "    pub fn spawn_batch(&mut self, count: u32) -> Vec<Entity> {",
        "        self.inner",
        "            .spawn_batch(count as usize)",
        "            .into_iter()",
        "            .map(|e| Entity { inner: e })",
        "            .collect()",
        "    }",
        "",
        "    #[napi]",
        "    pub fn despawn(&mut self, entity: &Entity) -> bool {",
        "        self.inner.despawn(entity.inner)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn entity_count(&self) -> u32 {",
        "        self.inner.entity_count() as u32",
        "    }",
        "",
        "    #[napi]",
        "    pub fn is_alive(&self, entity: &Entity) -> bool {",
        "        self.inner.is_alive(entity.inner)",
        "    }",
        "",
        "    // =========================================================================",
        "    // Transform2D Component",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn add_transform2d(&mut self, entity: &Entity, data: Transform2DData) {",
        "        let transform = Transform2D::from(&data);",
        "        self.inner.insert(entity.inner, transform);",
        "    }",
        "",
        "    #[napi]",
        "    pub fn get_transform2d(&self, entity: &Entity) -> Option<Transform2DData> {",
        "        self.inner",
        "            .get::<Transform2D>(entity.inner)",
        "            .map(Transform2DData::from)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn set_transform2d(&mut self, entity: &Entity, data: Transform2DData) {",
        "        if let Some(t) = self.inner.get_mut::<Transform2D>(entity.inner) {",
        "            t.position.x = data.position_x as f32;",
        "            t.position.y = data.position_y as f32;",
        "            t.rotation = data.rotation as f32;",
        "            t.scale.x = data.scale_x as f32;",
        "            t.scale.y = data.scale_y as f32;",
        "        }",
        "    }",
        "",
        "    #[napi]",
        "    pub fn has_transform2d(&self, entity: &Entity) -> bool {",
        "        self.inner.has::<Transform2D>(entity.inner)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn remove_transform2d(&mut self, entity: &Entity) -> bool {",
        "        self.inner.remove::<Transform2D>(entity.inner).is_some()",
        "    }",
        "",
        "    // =========================================================================",
        "    // Sprite Component",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn add_sprite(&mut self, entity: &Entity, data: SpriteData) {",
        "        let sprite = Sprite::from(&data);",
        "        self.inner.insert(entity.inner, sprite);",
        "    }",
        "",
        "    #[napi]",
        "    pub fn get_sprite(&self, entity: &Entity) -> Option<SpriteData> {",
        "        self.inner.get::<Sprite>(entity.inner).map(SpriteData::from)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn has_sprite(&self, entity: &Entity) -> bool {",
        "        self.inner.has::<Sprite>(entity.inner)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn remove_sprite(&mut self, entity: &Entity) -> bool {",
        "        self.inner.remove::<Sprite>(entity.inner).is_some()",
        "    }",
        "",
        "    // =========================================================================",
        "    // Name Component",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn add_name(&mut self, entity: &Entity, name: String) {",
        "        self.inner.insert(entity.inner, Name::new(&name));",
        "    }",
        "",
        "    #[napi]",
        "    pub fn get_name(&self, entity: &Entity) -> Option<String> {",
        "        self.inner",
        "            .get::<Name>(entity.inner)",
        "            .map(|n| n.as_str().to_string())",
        "    }",
        "",
        "    #[napi]",
        "    pub fn has_name(&self, entity: &Entity) -> bool {",
        "        self.inner.has::<Name>(entity.inner)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn remove_name(&mut self, entity: &Entity) -> bool {",
        "        self.inner.remove::<Name>(entity.inner).is_some()",
        "    }",
        "",
        "    // =========================================================================",
        "    // Legacy Game Loop (ECS-only)",
        "    // =========================================================================",
        "",
        "    #[napi]",
        "    pub fn update_frame(&mut self, delta_time: f64) {",
        "        let dt = delta_time as f32;",
        "        self.last_delta_time = dt;",
        "        self.inner.update_frame(dt, |_, _| {});",
        "    }",
        "",
        "    // =========================================================================",
        "    // Timing / Stats (getters)",
        "    // =========================================================================",
        "",
        "    #[napi(getter)]",
        "    pub fn delta_time(&self) -> f64 {",
        "        self.last_delta_time as f64",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn total_time(&self) -> f64 {",
        "        self.inner.total_time() as f64",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn fps(&self) -> f64 {",
        "        if self.last_delta_time > 0.0 {",
        "            (1.0 / self.last_delta_time) as f64",
        "        } else {",
        "            0.0",
        "        }",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn frame_count(&self) -> u32 {",
        "        self.inner.frame_count() as u32",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn is_initialized(&self) -> bool {",
        "        self.inner.is_initialized()",
        "    }",
        "",
        "    // =========================================================================",
        "    // Configuration (getters)",
        "    // =========================================================================",
        "",
        "    #[napi(getter)]",
        "    pub fn title(&self) -> String {",
        "        self.inner.title().to_string()",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn window_width(&self) -> u32 {",
        "        self.inner.window_size().0",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn window_height(&self) -> u32 {",
        "        self.inner.window_size().1",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn context_valid(&self) -> bool {",
        "        self.context_id != GOUD_INVALID_CONTEXT_ID",
        "    }",
        "",
        "    /// Returns the raw FFI delta time from the last poll_events call.",
        "    #[napi(getter)]",
        "    pub fn ffi_delta_time(&self) -> f64 {",
        "        goud_window_get_delta_time(self.context_id) as f64",
        "    }",
        "}",
        "",
    ]

    write_generated(NATIVE_SRC / "game.g.rs", "\n".join(lines))


def gen_napi_rust_lib():
    """Generate sdks/typescript/native/src/lib.rs.

    Declares the generated submodules with #[path] attributes so each .g.rs
    file is loaded under its natural module name (crate::types, crate::entity,
    crate::components, crate::game) — identical to how the hand-written files
    were declared.
    """
    lines = [
        RUST_HEADER,
        "#[allow(dead_code)]",
        "#[path = \"components.g.rs\"]",
        "mod components;",
        "#[path = \"entity.g.rs\"]",
        "mod entity;",
        "#[path = \"game.g.rs\"]",
        "mod game;",
        "#[allow(dead_code)]",
        "#[path = \"types.g.rs\"]",
        "mod types;",
        "",
    ]

    write_generated(NATIVE_SRC / "lib.rs", "\n".join(lines))


def gen_napi_rust():
    """Entry point: generate all napi-rs Rust files for the Node.js native addon."""
    gen_napi_rust_types()
    gen_napi_rust_entity()
    gen_napi_rust_components()
    gen_napi_rust_game()
    gen_napi_rust_lib()


if __name__ == "__main__":
    print("Generating TypeScript Node SDK...")
    gen_interface()
    gen_input()
    gen_math()
    gen_node_wrapper()
    gen_entry()
    gen_napi_rust()
    print("TypeScript Node SDK generation complete.")
