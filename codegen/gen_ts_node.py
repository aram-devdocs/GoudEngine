#!/usr/bin/env python3
"""Generates the complete TypeScript Desktop (Node.js) SDK from the universal schema.

Produces:
  sdks/typescript/native/src/game.g.rs    -- napi-rs Rust addon calling FFI
  sdks/typescript/src/types/engine.g.ts   -- IGoudGame interface
  sdks/typescript/src/types/input.g.ts    -- Key, MouseButton enums
  sdks/typescript/src/types/math.g.ts     -- Color, Vec2, Vec3, Rect classes
  sdks/typescript/src/node/index.g.ts     -- GoudGame wrapper with run() loop
  sdks/typescript/src/index.g.ts          -- Entry point
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


if __name__ == "__main__":
    print("Generating TypeScript Node SDK...")
    gen_interface()
    gen_input()
    gen_math()
    gen_node_wrapper()
    gen_entry()
    print("TypeScript Node SDK generation complete.")
