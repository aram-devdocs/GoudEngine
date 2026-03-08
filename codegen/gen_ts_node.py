#!/usr/bin/env python3
"""Generates the complete TypeScript Desktop (Node.js) SDK from the universal schema.

Produces:
  sdks/typescript/native/src/types.g.rs      -- Vec2, Vec3, Color, Rect + factory fns
  sdks/typescript/native/src/components.g.rs -- Transform2DData, SpriteData (FFI-only)
  sdks/typescript/native/src/entity.g.rs     -- Entity napi class (u64 bits, no EcsEntity)
  sdks/typescript/native/src/game.g.rs       -- GameConfig, GoudGame class (FFI-only)
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
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping, load_errors,
    to_pascal, to_snake, to_camel, write_generated, TYPESCRIPT_TYPES,
)

TS = SDKS_DIR / "typescript"
GEN = TS / "src" / "generated"
NATIVE_SRC = TS / "native" / "src"
schema = load_schema()
mapping = load_ffi_mapping()


IFACE_TYPES = {
    "Entity": "IEntity",
    "Transform2D": "ITransform2DData",
    "Sprite": "ISpriteData",
    "Vec2": "IVec2",
    "Vec3": "IVec3",
    "Color": "IColor",
    "Rect": "IRect",
    "RenderStats": "IRenderStats",
    "FpsStats": "IFpsStats",
    "Contact": "IContact",
    "Entity[]": "IEntity[]",
    "RenderCapabilities": "IRenderCapabilities",
    "PhysicsCapabilities": "IPhysicsCapabilities",
    "AudioCapabilities": "IAudioCapabilities",
    "InputCapabilities": "IInputCapabilities",
    "NetworkCapabilities": "INetworkCapabilities",
}


TS_EXCLUDE_METHODS = {
    "componentRegisterType", "componentAdd", "componentRemove",
    "componentHas", "componentGet", "componentGetMut",
    "componentAddBatch", "componentRemoveBatch", "componentHasBatch",
    "isAliveBatch",
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


# ---- engine.g.ts (IGoudGame interface) ------------------------------------

def gen_interface():
    tool = schema["tools"]["GoudGame"]
    lines = [f"// {HEADER_COMMENT}", ""]

    if schema["types"]["Vec2"].get("doc"):
        lines.append(f"/** {schema['types']['Vec2']['doc']} */")
    lines.append("export interface IVec2 { x: number; y: number; }")
    if schema["types"]["Vec3"].get("doc"):
        lines.append(f"/** {schema['types']['Vec3']['doc']} */")
    lines.append("export interface IVec3 { x: number; y: number; z: number; }")
    if schema["types"]["Color"].get("doc"):
        lines.append(f"/** {schema['types']['Color']['doc']} */")
    lines.append("export interface IColor { r: number; g: number; b: number; a: number; }")
    if schema["types"]["Rect"].get("doc"):
        lines.append(f"/** {schema['types']['Rect']['doc']} */")
    lines.append("export interface IRect { x: number; y: number; width: number; height: number; }")
    rs_fields = schema["types"]["RenderStats"]["fields"]
    rs_str = "; ".join(f"{to_camel(f['name'])}: number" for f in rs_fields)
    if schema["types"]["RenderStats"].get("doc"):
        lines.append(f"/** {schema['types']['RenderStats']['doc']} */")
    lines.append(f"export interface IRenderStats {{ {rs_str}; }}")
    if schema["types"].get("Contact", {}).get("doc"):
        lines.append(f"/** {schema['types']['Contact']['doc']} */")
    lines.append("export interface IContact { pointX: number; pointY: number; normalX: number; normalY: number; penetration: number; }")
    fps_fields = schema["types"]["FpsStats"]["fields"]
    fps_str = "; ".join(f"{to_camel(f['name'])}: number" for f in fps_fields)
    if schema["types"]["FpsStats"].get("doc"):
        lines.append(f"/** {schema['types']['FpsStats']['doc']} */")
    lines.append(f"export interface IFpsStats {{ {fps_str}; }}")

    # Capability interfaces
    for cap_name in ["RenderCapabilities", "PhysicsCapabilities", "AudioCapabilities", "InputCapabilities", "NetworkCapabilities"]:
        cap_type = schema["types"][cap_name]
        cap_fields = []
        for f in cap_type["fields"]:
            ft = f["type"]
            if ft == "bool":
                ts_ft = "boolean"
            else:
                ts_ft = "number"
            cap_fields.append(f"{to_camel(f['name'])}: {ts_ft}")
        cap_str = "; ".join(cap_fields)
        iface_name = IFACE_TYPES[cap_name]
        if cap_type.get("doc"):
            lines.append(f"/** {cap_type['doc']} */")
        lines.append(f"export interface {iface_name} {{ {cap_str}; }}")
    lines.append("")

    if schema["types"].get("Entity", {}).get("doc"):
        lines.append(f"/** {schema['types']['Entity']['doc']} */")
    lines.append("export interface IEntity {")
    lines.append("  readonly index: number;")
    lines.append("  readonly generation: number;")
    lines.append("  readonly isPlaceholder: boolean;")
    lines.append("  toBits(): bigint;")
    lines.append("}")
    lines.append("")

    if schema["types"]["Transform2D"].get("doc"):
        lines.append(f"/** {schema['types']['Transform2D']['doc']} */")
    lines.append("export interface ITransform2DData {")
    for f in schema["types"]["Transform2D"]["fields"]:
        if f.get("doc"):
            lines.append(f"  /** {f['doc']} */")
        lines.append(f"  {to_camel(f['name'])}: number;")
    lines.append("}")
    lines.append("")

    if schema["types"]["Sprite"].get("doc"):
        lines.append(f"/** {schema['types']['Sprite']['doc']} */")
    lines.append("export interface ISpriteData {")
    for f in schema["types"]["Sprite"]["fields"]:
        fn = to_camel(f["name"])
        ft = f["type"]
        if f.get("doc"):
            lines.append(f"  /** {f['doc']} */")
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

    if tool.get("doc"):
        lines.append(f"/** {tool['doc']} */")
    lines.append("export interface IGoudGame {")
    for prop in tool["properties"]:
        if prop.get("doc"):
            lines.append(f"  /** {prop['doc']} */")
        lines.append(f"  readonly {to_camel(prop['name'])}: {ts_iface_type(prop['type'])};")

    for method in tool["methods"]:
        mn = to_camel(method["name"])
        if mn in TS_EXCLUDE_METHODS:
            continue
        params = method.get("params", [])
        ret = method.get("returns", "void")

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
        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
        if method.get("async"):
            lines.append(f"  {mn}({sig}): Promise<{ts_ret}>;")
        else:
            lines.append(f"  {mn}({sig}): {ts_ret};")

    lines.append("}")
    lines.append("")

    # EngineConfig interface
    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        ec_tool = schema["tools"]["EngineConfig"]
        if ec_tool.get("doc"):
            lines.append(f"/** {ec_tool['doc']} */")
        lines.append("export interface IEngineConfig {")
        for method in ec_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            ret = method.get("returns", "void")
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            if mn == "build":
                lines.append("  build(): IGoudGame;")
            elif mn == "destroy":
                lines.append("  destroy(): void;")
            else:
                ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
                lines.append(f"  {mn}({ps}): IEngineConfig;")
        lines.append("}")
        lines.append("")

    write_generated(GEN / "types" / "engine.g.ts", "\n".join(lines))


# ---- input.g.ts -----------------------------------------------------------

def gen_input():
    lines = [f"// {HEADER_COMMENT}", ""]

    for enum_name, enum_def in schema["enums"].items():
        if enum_def.get("doc"):
            lines.append(f"/** {enum_def['doc']} */")
        lines.append(f"export enum {enum_name} {{")
        value_docs = enum_def.get("value_docs", {})
        for vname, vval in enum_def["values"].items():
            if value_docs.get(vname):
                lines.append(f"  /** {value_docs[vname]} */")
            lines.append(f"  {vname} = {vval},")
        lines.append("}")
        lines.append("")

    write_generated(GEN / "types" / "input.g.ts", "\n".join(lines))


# ---- math.g.ts ------------------------------------------------------------

def gen_math():
    lines = [f"// {HEADER_COMMENT}", "", "import type { IColor, IVec2, IVec3, IRect } from './engine.g.js';", ""]

    for type_name in ("Color", "Vec2", "Vec3", "Rect"):
        type_def = schema["types"][type_name]
        fields = type_def.get("fields", [])
        iface = f"I{type_name}"

        if type_def.get("doc"):
            lines.append(f"/** {type_def['doc']} */")
        lines.append(f"export class {type_name} implements {iface} {{")
        ctor_params = ", ".join(f"public {to_camel(f['name'])}: number" for f in fields)
        lines.append(f"  constructor({ctor_params}) {{}}")
        lines.append("")

        for factory in type_def.get("factories", []):
            fn = to_camel(factory["name"])
            fargs = factory.get("args", [])
            val = factory.get("value")
            if factory.get("doc"):
                lines.append(f"  /** {factory['doc']} */")
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
            if meth.get("doc"):
                lines.append(f"  /** {meth['doc']} */")
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
                field_names = [to_camel(f['name']) for f in fields]
                interps = ", ".join(
                    f"this.{fn} + (other.{fn} - this.{fn}) * t" for fn in field_names
                )
                lines.append(f"  {mn}(other: {type_name}, t: number): {type_name} {{ return new {type_name}({interps}); }}")
            elif mn == "withAlpha":
                lines.append(f"  {mn}(a: number): {type_name} {{ return new {type_name}(this.r, this.g, this.b, a); }}")
            elif mn == "contains":
                lines.append(f"  {mn}(p: IVec2): boolean {{ return p.x >= this.x && p.x <= this.x + this.width && p.y >= this.y && p.y <= this.y + this.height; }}")
            elif mn == "intersects":
                lines.append(f"  {mn}(o: IRect): boolean {{ return this.x < o.x + o.width && this.x + this.width > o.x && this.y < o.y + o.height && this.y + this.height > o.y; }}")

        lines.append("}")
        lines.append("")

    write_generated(GEN / "types" / "math.g.ts", "\n".join(lines))


# ---- node/index.g.ts (TS wrapper) -----------------------------------------

NATIVE_KNOWN_METHODS = {
    "shouldClose", "close", "destroy", "beginFrame", "endFrame",
    "loadTexture", "destroyTexture", "drawSprite", "drawQuad",
    "isKeyPressed", "isKeyJustPressed", "isKeyJustReleased",
    "isMouseButtonPressed", "isMouseButtonJustPressed", "isMouseButtonJustReleased",
    "getMousePosition", "getMouseDelta", "getScrollDelta",
    "spawnEmpty", "spawnBatch", "despawn", "entityCount", "isAlive",
    "addTransform2D", "getTransform2D", "setTransform2D", "hasTransform2D", "removeTransform2D",
    "addSprite", "getSprite", "setSprite", "hasSprite", "removeSprite",
    "addName", "getName", "hasName", "removeName",
    "drawSpriteRect", "setViewport", "enableDepthTest", "disableDepthTest",
    "clearDepth", "disableBlending", "getRenderStats",
    "getFpsStats", "setFpsOverlayEnabled", "setFpsUpdateInterval", "setFpsOverlayCorner",
    "mapActionKey", "isActionPressed", "isActionJustPressed", "isActionJustReleased",
    "collisionAabbAabb", "collisionCircleCircle", "collisionCircleAabb",
    "pointInRect", "pointInCircle", "aabbOverlap", "circleOverlap",
    "distance", "distanceSquared",
    "createCube", "createPlane", "createSphere", "createCylinder",
    "setObjectPosition", "setObjectRotation", "setObjectScale", "destroyObject",
    "addLight", "updateLight", "removeLight",
    "setCameraPosition3D", "setCameraRotation3D",
    "configureGrid", "setGridEnabled", "configureSkybox", "configureFog", "setFogEnabled",
    "render3D",
    "isAliveBatch", "despawnBatch",
    "windowWidth", "windowHeight",
    "getRenderCapabilities", "getPhysicsCapabilities", "getAudioCapabilities",
    "getInputCapabilities", "getNetworkCapabilities",
    "checkHotSwapShortcut",
}


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
        "import type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities } from '../types/engine.g.js';",
        "import { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Key, MouseButton } from '../types/input.g.js';",
        "export type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities } from '../types/engine.g.js';",
        "",
    ]
    if tool.get("doc"):
        lines.append(f"/** {tool['doc']} */")
    lines += [
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
        pt = ts_type(prop["type"])
        if prop.get("doc"):
            lines.append(f"  /** {prop['doc']} */")
        lines.append(f"  get {pn}(): {pt} {{ return this.native.{pn}; }}")
    lines.append("")

    for method in tool["methods"]:
        mn = to_camel(method["name"])
        if mn in TS_EXCLUDE_METHODS:
            continue
        mm = tm["methods"].get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")

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
                if can_be_optional[i]:
                    default_val = p.get("default")
                    call_args.append(f"{pn} ?? {default_val}")
                else:
                    call_args.append(pn)

        sig = ", ".join(param_strs)
        ts_ret = ts_iface_type(ret)

        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
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
            lines.append("    return this.native.drawSpriteRect(texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, c.r, c.g, c.b, c.a);")
        elif mn == "drawQuad":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    this.native.drawQuad(x, y, width, height, c.r, c.g, c.b, c.a);")
        elif mn in ("getMousePosition", "getMouseDelta", "getScrollDelta"):
            lines.append(f"    const arr = this.native.{mn}();")
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
            lines.append("    this.native.setSprite(entity as unknown as NativeEntity, sprite as any);")
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
            lines.append("    return this.native.getRenderStats() as unknown as IRenderStats;")
        elif mn == "getFpsStats":
            lines.append("    return this.native.getFpsStats() as unknown as IFpsStats;")
        elif mn == "loadTexture":
            lines.append("    return this.native.loadTexture(path);")
        elif mn == "destroy":
            lines.append("    this.native.destroy();")
        else:
            native_call_args = ", ".join(call_args)
            native_obj = "this.native" if mn in NATIVE_KNOWN_METHODS else "(this.native as any)"
            if ret == "void":
                lines.append(f"    {native_obj}.{mn}({native_call_args});")
            else:
                lines.append(f"    return {native_obj}.{mn}({native_call_args});")

        lines.append("  }")
        lines.append("")

    lines.append("}")
    lines.append("")

    # EngineConfig class
    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        ec_tool = schema["tools"]["EngineConfig"]
        lines.append("import type { IEngineConfig } from '../types/engine.g.js';")
        lines.append("")
        if ec_tool.get("doc"):
            lines.append(f"/** {ec_tool['doc']} */")
        lines.append("export class EngineConfig implements IEngineConfig {")
        lines.append("  private native: any;")
        lines.append("")
        lines.append("  constructor() {")
        lines.append("    const { NativeEngineConfig } = require('../../../index');")
        lines.append("    this.native = new NativeEngineConfig();")
        lines.append("  }")
        lines.append("")

        for method in ec_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            if mn == "build":
                lines.append("  build(): GoudGame {")
                lines.append("    const ctx = this.native.build();")
                lines.append("    const game = Object.create(GoudGame.prototype);")
                lines.append("    game.native = ctx;")
                lines.append("    return game;")
                lines.append("  }")
            elif mn == "destroy":
                lines.append("  destroy(): void {")
                lines.append("    this.native.destroy();")
                lines.append("  }")
            else:
                ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
                lines.append(f"  {mn}({ps}): EngineConfig {{")
                args = ", ".join(to_camel(p["name"]) for p in params)
                lines.append(f"    this.native.{mn}({args});")
                lines.append("    return this;")
                lines.append("  }")
            lines.append("")

        lines.append("}")
        lines.append("")

    write_generated(GEN / "node" / "index.g.ts", "\n".join(lines))


# ---- index.g.ts (entry point) ---------------------------------------------

def gen_entry():
    has_engine_config = "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {})
    ec_export = ", EngineConfig" if has_engine_config else ""
    ec_type_export = ", IEngineConfig" if has_engine_config else ""
    # Build error re-export from schema categories
    errors_section = schema.get("errors", {})
    error_names = ["GoudError"]
    for cat in errors_section.get("categories", []):
        error_names.append(cat["base_class"])
    error_names.append("RecoveryClass")

    has_diagnostic = "diagnostic" in schema

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        f"export {{ GoudGame{ec_export}, Color, Vec2, Vec3, Key, MouseButton }} from './node/index.g.js';",
        f"export type {{ IGoudGame{ec_type_export}, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities }} from './types/engine.g.js';",
        "export type { Rect } from './types/math.g.js';",
        f"export {{ {', '.join(error_names)} }} from './errors.g.js';",
    ]
    if has_diagnostic:
        lines.append("export { DiagnosticMode } from './diagnostic.g.js';")
    lines.append("")
    write_generated(GEN / "index.g.ts", "\n".join(lines))


# ============================================================================
# napi-rs Rust code generation (FFI-only -- no Rust SDK access)
# ============================================================================

RUST_HEADER = f"// {HEADER_COMMENT}"

NAPI_RUST_TYPES = {
    "f32": "f64", "f64": "f64", "u32": "u32", "u64": "f64",
    "i32": "i32", "i64": "f64", "bool": "bool", "string": "String", "void": "()",
}


def _napi_type(schema_type: str) -> str:
    nullable = schema_type.endswith("?")
    base = schema_type.rstrip("?")
    mapped = NAPI_RUST_TYPES.get(base, base)
    if nullable:
        return f"Option<{mapped}>"
    return mapped


def gen_napi_rust_types():
    """Generate sdks/typescript/native/src/types.g.rs.

    Produces napi(object) structs for Vec2, Vec3, Color, Rect plus standalone
    #[napi] factory functions for Color.  FFI-only: no engine type imports.
    """
    lines = [
        RUST_HEADER,
        "use napi_derive::napi;",
        "",
    ]

    # -- struct definitions (no engine From impls) -------------------------
    struct_meta = {
        "Vec2":  {"fields": [("x", "f64"), ("y", "f64")]},
        "Vec3":  {"fields": [("x", "f64"), ("y", "f64"), ("z", "f64")]},
        "Color": {"fields": [("r", "f64"), ("g", "f64"), ("b", "f64"), ("a", "f64")]},
        "Rect":  {"fields": [("x", "f64"), ("y", "f64"), ("width", "f64"), ("height", "f64")]},
    }

    for name, meta in struct_meta.items():
        lines.append("#[napi(object)]")
        lines.append("#[derive(Clone, Debug)]")
        lines.append(f"pub struct {name} {{")
        for fname, ftype in meta["fields"]:
            lines.append(f"    pub {fname}: {ftype},")
        lines.append("}")
        lines.append("")

    # -- Color factory functions (pure data, no engine imports) ------------
    color_schema = schema["types"]["Color"]
    for factory in color_schema.get("factories", []):
        fname = factory["name"]
        fargs = factory.get("args", [])
        val = factory.get("value")
        fn_name = f"color_{to_snake(fname)}"

        if fname == "rgba":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(r: f64, g: f64, b: f64, a: f64) -> Color {{")
            lines.append("    Color { r, g, b, a }")
            lines.append("}")
        elif fname == "rgb":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(r: f64, g: f64, b: f64) -> Color {{")
            lines.append("    Color { r, g, b, a: 1.0 }")
            lines.append("}")
        elif fname == "fromHex":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(hex: u32) -> Color {{")
            lines.append("    Color {")
            lines.append("        r: ((hex >> 16) & 0xFF) as f64 / 255.0,")
            lines.append("        g: ((hex >> 8) & 0xFF) as f64 / 255.0,")
            lines.append("        b: (hex & 0xFF) as f64 / 255.0,")
            lines.append("        a: 1.0,")
            lines.append("    }")
            lines.append("}")
        elif fname == "fromU8":
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}(r: u32, g: u32, b: u32, a: u32) -> Color {{")
            lines.append("    Color {")
            lines.append("        r: r as f64 / 255.0,")
            lines.append("        g: g as f64 / 255.0,")
            lines.append("        b: b as f64 / 255.0,")
            lines.append("        a: a as f64 / 255.0,")
            lines.append("    }")
            lines.append("}")
        elif val is not None and not fargs:
            # Named constant factory (white, black, red, ...)
            vals = val
            lines.append("#[napi]")
            lines.append(f"pub fn {fn_name}() -> Color {{")
            lines.append(f"    Color {{ r: {float(vals[0])}, g: {float(vals[1])}, b: {float(vals[2])}, a: {float(vals[3])} }}")
            lines.append("}")
        lines.append("")

    write_generated(NATIVE_SRC / "types.g.rs", "\n".join(lines))


def gen_napi_rust_entity():
    """Generate sdks/typescript/native/src/entity.g.rs.

    Produces the Entity napi class that wraps a u64 entity ID.
    FFI-only: no EcsEntity import; bit packing done inline.
    Entity bits layout: (generation << 32) | index
    PLACEHOLDER: index=u32::MAX, generation=0 => bits = 0x00000000FFFFFFFF
    """
    lines = [
        RUST_HEADER,
        "use napi::bindgen_prelude::*;",
        "use napi_derive::napi;",
        "",
        "/// PLACEHOLDER entity bits: index=u32::MAX, generation=0.",
        "const PLACEHOLDER_BITS: u64 = u32::MAX as u64;",
        "",
        "#[napi]",
        "pub struct Entity {",
        "    /// Packed entity bits: (generation << 32) | index.",
        "    pub(crate) bits: u64,",
        "}",
        "",
        "#[napi]",
        "impl Entity {",
        "    #[napi(constructor)]",
        "    pub fn new(index: u32, generation: u32) -> Self {",
        "        Self {",
        "            bits: ((generation as u64) << 32) | (index as u64),",
        "        }",
        "    }",
        "",
        "    #[napi(factory)]",
        "    pub fn placeholder() -> Self {",
        "        Self {",
        "            bits: PLACEHOLDER_BITS,",
        "        }",
        "    }",
        "",
        "    #[napi(factory)]",
        "    pub fn from_bits(bits: BigInt) -> Result<Self> {",
        "        let (_, value, _) = bits.get_u64();",
        "        Ok(Self { bits: value })",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn index(&self) -> u32 {",
        "        self.bits as u32",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn generation(&self) -> u32 {",
        "        (self.bits >> 32) as u32",
        "    }",
        "",
        "    #[napi(getter)]",
        "    pub fn is_placeholder(&self) -> bool {",
        "        self.bits == PLACEHOLDER_BITS",
        "    }",
        "",
        "    #[napi]",
        "    pub fn to_bits(&self) -> BigInt {",
        "        BigInt::from(self.bits)",
        "    }",
        "",
        "    #[napi]",
        "    pub fn display(&self) -> String {",
        "        let index = self.bits as u32;",
        "        let gen = (self.bits >> 32) as u32;",
        '        format!("Entity({}:{})", index, gen)',
        "    }",
        "}",
        "",
    ]

    write_generated(NATIVE_SRC / "entity.g.rs", "\n".join(lines))


def gen_napi_rust_components():
    """Generate sdks/typescript/native/src/components.g.rs.

    Produces Transform2DData and SpriteData napi(object) structs together
    with standalone #[napi] factory functions.
    FFI-only: no engine component imports or From impls.
    """
    lines = [
        RUST_HEADER,
        "use crate::types::Color;",
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
        "impl Default for SpriteData {",
        "    fn default() -> Self {",
        "        Self {",
        "            color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },",
        "            flip_x: false,",
        "            flip_y: false,",
        "            anchor_x: 0.5,",
        "            anchor_y: 0.5,",
        "            custom_width: None,",
        "            custom_height: None,",
        "            source_rect_x: None,",
        "            source_rect_y: None,",
        "            source_rect_width: None,",
        "            source_rect_height: None,",
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
        "    SpriteData::default()",
        "}",
        "",
        "// Factory functions for Vec2 convenience",
        "#[napi]",
        "pub fn vec2(x: f64, y: f64) -> crate::types::Vec2 {",
        "    crate::types::Vec2 { x, y }",
        "}",
        "",
        "#[napi]",
        "pub fn vec2_zero() -> crate::types::Vec2 {",
        "    crate::types::Vec2 { x: 0.0, y: 0.0 }",
        "}",
        "",
        "#[napi]",
        "pub fn vec2_one() -> crate::types::Vec2 {",
        "    crate::types::Vec2 { x: 1.0, y: 1.0 }",
        "}",
        "",
    ]

    write_generated(NATIVE_SRC / "components.g.rs", "\n".join(lines))


def gen_napi_rust_game():
    """Generate sdks/typescript/native/src/game.g.rs.

    FFI-only: no EngineGoudGame.  All operations go through goud_* FFI
    functions.  Component ops use context registry with typed ECS access.
    Lifecycle, rendering, input, collision, and 3D use pure FFI functions.
    """
    write_generated(NATIVE_SRC / "game.g.rs", _game_rs_content())



def _game_rs_content():
    return RUST_HEADER + r"""
use crate::components::{SpriteData, Transform2DData};
use crate::entity::Entity;
use goud_engine::ffi::collision::{
    goud_collision_aabb_aabb, goud_collision_aabb_overlap, goud_collision_circle_aabb,
    goud_collision_circle_circle, goud_collision_circle_overlap,
    goud_collision_distance, goud_collision_distance_squared,
    goud_collision_point_in_circle, goud_collision_point_in_rect, GoudContact,
};
use goud_engine::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};
use goud_engine::ffi::debug::{
    goud_debug_get_fps_stats, goud_debug_set_fps_overlay_corner,
    goud_debug_set_fps_overlay_enabled, goud_debug_set_fps_update_interval,
};
use goud_engine::sdk::debug_overlay::FpsStats;
use goud_engine::ffi::entity::{
    goud_entity_count, goud_entity_despawn, goud_entity_is_alive,
    goud_entity_spawn_batch, goud_entity_spawn_empty,
};
use goud_engine::ffi::input::{
    goud_input_get_mouse_delta, goud_input_get_mouse_position,
    goud_input_get_scroll_delta, goud_input_key_just_pressed,
    goud_input_key_just_released, goud_input_key_pressed,
    goud_input_mouse_button_just_pressed, goud_input_mouse_button_just_released,
    goud_input_mouse_button_pressed,
    goud_input_map_action_key, goud_input_action_pressed,
    goud_input_action_just_pressed, goud_input_action_just_released,
};
use goud_engine::ffi::renderer::{
    goud_renderer_begin, goud_renderer_clear_depth, goud_renderer_disable_blending,
    goud_renderer_disable_depth_test, goud_renderer_draw_quad,
    goud_renderer_draw_sprite, goud_renderer_draw_sprite_rect,
    goud_renderer_enable_blending, goud_renderer_enable_depth_test,
    goud_renderer_end, goud_renderer_get_stats, goud_renderer_set_viewport,
    goud_texture_destroy, goud_texture_load, GoudRenderStats,
};
use goud_engine::ffi::renderer3d::{
    goud_renderer3d_add_light, goud_renderer3d_configure_fog,
    goud_renderer3d_configure_grid, goud_renderer3d_configure_skybox,
    goud_renderer3d_create_cube, goud_renderer3d_create_cylinder,
    goud_renderer3d_create_plane, goud_renderer3d_create_sphere,
    goud_renderer3d_destroy_object, goud_renderer3d_remove_light,
    goud_renderer3d_render, goud_renderer3d_set_camera_position,
    goud_renderer3d_set_camera_rotation, goud_renderer3d_set_fog_enabled,
    goud_renderer3d_set_grid_enabled, goud_renderer3d_set_object_position,
    goud_renderer3d_set_object_rotation, goud_renderer3d_set_object_scale,
    goud_renderer3d_update_light,
};
use goud_engine::ffi::providers::{
    goud_provider_render_capabilities, goud_provider_physics_capabilities,
    goud_provider_audio_capabilities, goud_provider_input_capabilities,
    goud_provider_network_capabilities, goud_provider_check_hot_swap_shortcut,
};
use goud_engine::core::providers::types::{
    RenderCapabilities, PhysicsCapabilities, AudioCapabilities,
};
use goud_engine::core::providers::input_types::InputCapabilities;
use goud_engine::core::providers::network_types::NetworkCapabilities;
use goud_engine::ffi::window::{
    goud_window_clear, goud_window_create, goud_window_destroy,
    goud_window_get_delta_time, goud_window_get_size, goud_window_poll_events,
    goud_window_set_should_close, goud_window_should_close,
    goud_window_swap_buffers,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::ffi::CString;

// =============================================================================
// GameConfig
// =============================================================================

#[napi(object)]
#[derive(Clone, Debug)]
pub struct GameConfig {
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

// =============================================================================
// RenderStats / Contact napi objects
// =============================================================================

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiRenderStats {
    pub draw_calls: u32,
    pub triangles: u32,
    pub texture_binds: u32,
    pub shader_binds: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiContact {
    pub point_x: f64,
    pub point_y: f64,
    pub normal_x: f64,
    pub normal_y: f64,
    pub penetration: f64,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiFpsStats {
    pub current_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub avg_fps: f64,
    pub frame_time_ms: f64,
}

// =============================================================================
// Provider Capabilities napi objects
// =============================================================================

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiRenderCapabilities {
    pub max_texture_units: u32,
    pub max_texture_size: u32,
    pub supports_instancing: bool,
    pub supports_compute: bool,
    pub supports_msaa: bool,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiPhysicsCapabilities {
    pub supports_continuous_collision: bool,
    pub supports_joints: bool,
    pub max_bodies: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiAudioCapabilities {
    pub supports_spatial: bool,
    pub max_channels: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiInputCapabilities {
    pub supports_gamepad: bool,
    pub supports_touch: bool,
    pub max_gamepads: u32,
}

#[napi(object)]
#[derive(Clone, Debug)]
pub struct NapiNetworkCapabilities {
    pub supports_hosting: bool,
    pub max_connections: u32,
    pub max_channels: u32,
    pub max_message_size: u32,
}

// =============================================================================
// GoudGame -- FFI-only, no Rust SDK access
// =============================================================================

#[napi]
pub struct GoudGame {
    context_id: GoudContextId,
    last_delta_time: f32,
    title: String,
    frame_count: u64,
    total_time: f64,
}

#[napi]
impl GoudGame {
    #[napi(constructor)]
    pub fn new(config: Option<GameConfig>) -> Result<Self> {
        let width = config.as_ref().and_then(|c| c.width).unwrap_or(800);
        let height = config.as_ref().and_then(|c| c.height).unwrap_or(600);
        let title_str = config
            .as_ref()
            .and_then(|c| c.title.clone())
            .unwrap_or_else(|| "GoudEngine".to_string());

        let c_title = CString::new(title_str.as_str())
            .map_err(|e| Error::from_reason(format!("Invalid title string: {}", e)))?;

        // SAFETY: CString guarantees a valid null-terminated pointer.
        let context_id = unsafe { goud_window_create(width, height, c_title.as_ptr()) };
        if context_id == GOUD_INVALID_CONTEXT_ID {
            return Err(Error::from_reason("Failed to create GLFW window"));
        }

        Ok(Self { context_id, last_delta_time: 0.0, title: title_str, frame_count: 0, total_time: 0.0 })
    }

    // =========================================================================
    // Lifecycle
    // =========================================================================

    #[napi]
    pub fn should_close(&self) -> bool { goud_window_should_close(self.context_id) }

    #[napi]
    pub fn close(&self) { goud_window_set_should_close(self.context_id, true); }

    #[napi]
    pub fn destroy(&self) -> bool { goud_window_destroy(self.context_id) }

    // =========================================================================
    // Frame Management
    // =========================================================================

    #[napi]
    pub fn begin_frame(&mut self, r: Option<f64>, g: Option<f64>, b: Option<f64>, a: Option<f64>) {
        let dt = goud_window_poll_events(self.context_id);
        self.last_delta_time = dt;
        goud_window_clear(self.context_id,
            r.unwrap_or(0.0) as f32, g.unwrap_or(0.0) as f32,
            b.unwrap_or(0.0) as f32, a.unwrap_or(1.0) as f32);
        goud_renderer_begin(self.context_id);
        goud_renderer_enable_blending(self.context_id);
    }

    #[napi]
    pub fn end_frame(&self) {
        goud_renderer_end(self.context_id);
        goud_window_swap_buffers(self.context_id);
    }

    // =========================================================================
    // Rendering
    // =========================================================================

    #[napi]
    pub fn load_texture(&self, path: String) -> Result<f64> {
        let c_path = CString::new(path)
            .map_err(|e| Error::from_reason(format!("Invalid path: {}", e)))?;
        // SAFETY: CString guarantees a valid null-terminated pointer.
        let handle = unsafe { goud_texture_load(self.context_id, c_path.as_ptr()) };
        Ok(handle as f64)
    }

    #[napi]
    pub fn destroy_texture(&self, handle: f64) -> bool {
        goud_texture_destroy(self.context_id, handle as u64)
    }

    #[napi]
    pub fn draw_sprite(&self, texture: f64, x: f64, y: f64, w: f64, h: f64,
                       rotation: Option<f64>, r: Option<f64>, g: Option<f64>,
                       b: Option<f64>, a: Option<f64>) -> bool {
        goud_renderer_draw_sprite(self.context_id, texture as u64,
            x as f32, y as f32, w as f32, h as f32, rotation.unwrap_or(0.0) as f32,
            r.unwrap_or(1.0) as f32, g.unwrap_or(1.0) as f32,
            b.unwrap_or(1.0) as f32, a.unwrap_or(1.0) as f32)
    }

    #[napi]
    pub fn draw_sprite_rect(&self, texture: f64, x: f64, y: f64, w: f64, h: f64,
                            rotation: Option<f64>, src_x: f64, src_y: f64,
                            src_w: f64, src_h: f64, r: Option<f64>, g: Option<f64>,
                            b: Option<f64>, a: Option<f64>) -> bool {
        goud_renderer_draw_sprite_rect(self.context_id, texture as u64,
            x as f32, y as f32, w as f32, h as f32, rotation.unwrap_or(0.0) as f32,
            src_x as f32, src_y as f32, src_w as f32, src_h as f32,
            r.unwrap_or(1.0) as f32, g.unwrap_or(1.0) as f32,
            b.unwrap_or(1.0) as f32, a.unwrap_or(1.0) as f32)
    }

    #[napi]
    pub fn draw_quad(&self, x: f64, y: f64, w: f64, h: f64,
                     r: Option<f64>, g: Option<f64>, b: Option<f64>, a: Option<f64>) -> bool {
        goud_renderer_draw_quad(self.context_id,
            x as f32, y as f32, w as f32, h as f32,
            r.unwrap_or(1.0) as f32, g.unwrap_or(1.0) as f32,
            b.unwrap_or(1.0) as f32, a.unwrap_or(1.0) as f32)
    }

    #[napi]
    pub fn set_viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        goud_renderer_set_viewport(self.context_id, x, y, width, height);
    }

    #[napi]
    pub fn enable_depth_test(&self) { goud_renderer_enable_depth_test(self.context_id); }
    #[napi]
    pub fn disable_depth_test(&self) { goud_renderer_disable_depth_test(self.context_id); }
    #[napi]
    pub fn clear_depth(&self) { goud_renderer_clear_depth(self.context_id); }
    #[napi]
    pub fn disable_blending(&self) { goud_renderer_disable_blending(self.context_id); }

    #[napi]
    pub fn get_render_stats(&self) -> NapiRenderStats {
        let mut stats = GoudRenderStats { draw_calls: 0, triangles: 0, texture_binds: 0, shader_binds: 0 };
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_renderer_get_stats(self.context_id, &mut stats) };
        NapiRenderStats { draw_calls: stats.draw_calls, triangles: stats.triangles, texture_binds: stats.texture_binds, shader_binds: stats.shader_binds }
    }

    // =========================================================================
    // Debug overlay
    // =========================================================================

    #[napi]
    pub fn get_fps_stats(&self) -> NapiFpsStats {
        let mut stats = FpsStats::default();
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_debug_get_fps_stats(self.context_id, &mut stats) };
        NapiFpsStats {
            current_fps: stats.current_fps as f64,
            min_fps: stats.min_fps as f64,
            max_fps: stats.max_fps as f64,
            avg_fps: stats.avg_fps as f64,
            frame_time_ms: stats.frame_time_ms as f64,
        }
    }

    #[napi]
    pub fn set_fps_overlay_enabled(&self, enabled: bool) {
        goud_debug_set_fps_overlay_enabled(self.context_id, enabled);
    }

    #[napi]
    pub fn set_fps_update_interval(&self, interval: f64) {
        goud_debug_set_fps_update_interval(self.context_id, interval as f32);
    }

    #[napi]
    pub fn set_fps_overlay_corner(&self, corner: i32) {
        goud_debug_set_fps_overlay_corner(self.context_id, corner);
    }

    // =========================================================================
    // Provider Capabilities
    // =========================================================================

    #[napi]
    pub fn get_render_capabilities(&self) -> NapiRenderCapabilities {
        let mut caps = RenderCapabilities::default();
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_provider_render_capabilities(self.context_id, &mut caps) };
        NapiRenderCapabilities {
            max_texture_units: caps.max_texture_units,
            max_texture_size: caps.max_texture_size,
            supports_instancing: caps.supports_instancing,
            supports_compute: caps.supports_compute,
            supports_msaa: caps.supports_msaa,
        }
    }

    #[napi]
    pub fn get_physics_capabilities(&self) -> NapiPhysicsCapabilities {
        let mut caps = PhysicsCapabilities::default();
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_provider_physics_capabilities(self.context_id, &mut caps) };
        NapiPhysicsCapabilities {
            supports_continuous_collision: caps.supports_continuous_collision,
            supports_joints: caps.supports_joints,
            max_bodies: caps.max_bodies,
        }
    }

    #[napi]
    pub fn get_audio_capabilities(&self) -> NapiAudioCapabilities {
        let mut caps = AudioCapabilities::default();
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_provider_audio_capabilities(self.context_id, &mut caps) };
        NapiAudioCapabilities {
            supports_spatial: caps.supports_spatial,
            max_channels: caps.max_channels,
        }
    }

    #[napi]
    pub fn get_input_capabilities(&self) -> NapiInputCapabilities {
        let mut caps = InputCapabilities::default();
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_provider_input_capabilities(self.context_id, &mut caps) };
        NapiInputCapabilities {
            supports_gamepad: caps.supports_gamepad,
            supports_touch: caps.supports_touch,
            max_gamepads: caps.max_gamepads,
        }
    }

    #[napi]
    pub fn get_network_capabilities(&self) -> NapiNetworkCapabilities {
        let mut caps = NetworkCapabilities::default();
        // SAFETY: Passing a valid mutable reference as out-pointer.
        unsafe { goud_provider_network_capabilities(self.context_id, &mut caps) };
        NapiNetworkCapabilities {
            supports_hosting: caps.supports_hosting,
            max_connections: caps.max_connections,
            max_channels: caps.max_channels as u32,
            max_message_size: caps.max_message_size,
        }
    }

    /// Checks if the hot-swap shortcut (F5) was pressed and cycles render provider. Debug builds only.
    #[napi]
    pub fn check_hot_swap_shortcut(&self) -> bool {
        // SAFETY: context_id is a valid opaque handle obtained at construction.
        goud_provider_check_hot_swap_shortcut(self.context_id) != 0
    }

    // =========================================================================
    // Input
    // =========================================================================

    #[napi]
    pub fn is_key_pressed(&self, key: i32) -> bool { goud_input_key_pressed(self.context_id, key) }
    #[napi]
    pub fn is_key_just_pressed(&self, key: i32) -> bool { goud_input_key_just_pressed(self.context_id, key) }
    #[napi]
    pub fn is_key_just_released(&self, key: i32) -> bool { goud_input_key_just_released(self.context_id, key) }
    #[napi]
    pub fn is_mouse_button_pressed(&self, button: i32) -> bool { goud_input_mouse_button_pressed(self.context_id, button) }
    #[napi]
    pub fn is_mouse_button_just_pressed(&self, button: i32) -> bool { goud_input_mouse_button_just_pressed(self.context_id, button) }
    #[napi]
    pub fn is_mouse_button_just_released(&self, button: i32) -> bool { goud_input_mouse_button_just_released(self.context_id, button) }

    #[napi]
    pub fn get_mouse_position(&self) -> Vec<f64> {
        let (mut x, mut y) = (0.0f32, 0.0f32);
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_input_get_mouse_position(self.context_id, &mut x, &mut y) };
        vec![x as f64, y as f64]
    }

    #[napi]
    pub fn get_mouse_delta(&self) -> Vec<f64> {
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_input_get_mouse_delta(self.context_id, &mut dx, &mut dy) };
        vec![dx as f64, dy as f64]
    }

    #[napi]
    pub fn get_scroll_delta(&self) -> Vec<f64> {
        let (mut dx, mut dy) = (0.0f32, 0.0f32);
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_input_get_scroll_delta(self.context_id, &mut dx, &mut dy) };
        vec![dx as f64, dy as f64]
    }

    #[napi]
    pub fn map_action_key(&self, action: String, key: i32) -> Result<bool> {
        let c = CString::new(action).map_err(|e| Error::from_reason(format!("{}", e)))?;
        // SAFETY: CString guarantees a valid null-terminated pointer.
        Ok(unsafe { goud_input_map_action_key(self.context_id, c.as_ptr(), key) })
    }

    #[napi]
    pub fn is_action_pressed(&self, action: String) -> Result<bool> {
        let c = CString::new(action).map_err(|e| Error::from_reason(format!("{}", e)))?;
        // SAFETY: CString guarantees a valid null-terminated pointer.
        Ok(unsafe { goud_input_action_pressed(self.context_id, c.as_ptr()) })
    }

    #[napi]
    pub fn is_action_just_pressed(&self, action: String) -> Result<bool> {
        let c = CString::new(action).map_err(|e| Error::from_reason(format!("{}", e)))?;
        // SAFETY: CString guarantees a valid null-terminated pointer.
        Ok(unsafe { goud_input_action_just_pressed(self.context_id, c.as_ptr()) })
    }

    #[napi]
    pub fn is_action_just_released(&self, action: String) -> Result<bool> {
        let c = CString::new(action).map_err(|e| Error::from_reason(format!("{}", e)))?;
        // SAFETY: CString guarantees a valid null-terminated pointer.
        Ok(unsafe { goud_input_action_just_released(self.context_id, c.as_ptr()) })
    }

    // =========================================================================
    // Entity Operations (ECS) -- via FFI
    // =========================================================================

    #[napi]
    pub fn spawn_empty(&self) -> Entity {
        Entity { bits: goud_entity_spawn_empty(self.context_id) }
    }

    #[napi]
    pub fn spawn_batch(&self, count: u32) -> Vec<Entity> {
        let mut out = vec![0u64; count as usize];
        // SAFETY: out buffer is correctly sized for count u64 values.
        let n = unsafe { goud_entity_spawn_batch(self.context_id, count, out.as_mut_ptr()) };
        out.truncate(n as usize);
        out.into_iter().map(|bits| Entity { bits }).collect()
    }

    #[napi]
    pub fn despawn(&self, entity: &Entity) -> bool {
        goud_entity_despawn(self.context_id, entity.bits).success
    }

    #[napi]
    pub fn entity_count(&self) -> u32 { goud_entity_count(self.context_id) }

    #[napi]
    pub fn is_alive(&self, entity: &Entity) -> bool {
        goud_entity_is_alive(self.context_id, entity.bits)
    }

    // =========================================================================
    // Transform2D Component -- via FFI context registry
    // =========================================================================

    #[napi]
    pub fn add_transform2d(&self, entity: &Entity, data: Transform2DData) {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Transform2D;
        use goud_engine::core::math::Vec2;
        let mut reg = get_context_registry().lock().unwrap();
        if let Some(ctx) = reg.get_mut(self.context_id) {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            ctx.world_mut().insert(e, Transform2D {
                position: Vec2::new(data.position_x as f32, data.position_y as f32),
                rotation: data.rotation as f32,
                scale: Vec2::new(data.scale_x as f32, data.scale_y as f32),
            });
        }
    }

    #[napi]
    pub fn get_transform2d(&self, entity: &Entity) -> Option<Transform2DData> {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Transform2D;
        let reg = get_context_registry().lock().unwrap();
        reg.get(self.context_id).and_then(|ctx| {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            ctx.world().get::<Transform2D>(e).map(|t| Transform2DData {
                position_x: t.position.x as f64, position_y: t.position.y as f64,
                rotation: t.rotation as f64, scale_x: t.scale.x as f64, scale_y: t.scale.y as f64,
            })
        })
    }

    #[napi]
    pub fn set_transform2d(&self, entity: &Entity, data: Transform2DData) {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Transform2D;
        let mut reg = get_context_registry().lock().unwrap();
        if let Some(ctx) = reg.get_mut(self.context_id) {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            if let Some(t) = ctx.world_mut().get_mut::<Transform2D>(e) {
                t.position.x = data.position_x as f32; t.position.y = data.position_y as f32;
                t.rotation = data.rotation as f32;
                t.scale.x = data.scale_x as f32; t.scale.y = data.scale_y as f32;
            }
        }
    }

    #[napi]
    pub fn has_transform2d(&self, entity: &Entity) -> bool {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Transform2D;
        let reg = get_context_registry().lock().unwrap();
        reg.get(self.context_id).is_some_and(|ctx| {
            ctx.world().has::<Transform2D>(goud_engine::ecs::Entity::from_bits(entity.bits))
        })
    }

    #[napi]
    pub fn remove_transform2d(&self, entity: &Entity) -> bool {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Transform2D;
        let mut reg = get_context_registry().lock().unwrap();
        reg.get_mut(self.context_id).is_some_and(|ctx| {
            ctx.world_mut().remove::<Transform2D>(goud_engine::ecs::Entity::from_bits(entity.bits)).is_some()
        })
    }

    // =========================================================================
    // Sprite Component -- via FFI context registry
    // =========================================================================

    #[napi]
    pub fn add_sprite(&self, entity: &Entity, data: SpriteData) {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Sprite;
        use goud_engine::core::math::{Color, Rect, Vec2};
        let mut reg = get_context_registry().lock().unwrap();
        if let Some(ctx) = reg.get_mut(self.context_id) {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            ctx.world_mut().insert(e, Sprite {
                color: Color::rgba(data.color.r as f32, data.color.g as f32, data.color.b as f32, data.color.a as f32),
                flip_x: data.flip_x, flip_y: data.flip_y,
                anchor: Vec2::new(data.anchor_x as f32, data.anchor_y as f32),
                custom_size: match (data.custom_width, data.custom_height) {
                    (Some(w), Some(h)) => Some(Vec2::new(w as f32, h as f32)), _ => None,
                },
                source_rect: match (data.source_rect_x, data.source_rect_y, data.source_rect_width, data.source_rect_height) {
                    (Some(x), Some(y), Some(w), Some(h)) => Some(Rect::new(x as f32, y as f32, w as f32, h as f32)), _ => None,
                },
                ..Default::default()
            });
        }
    }

    #[napi]
    pub fn get_sprite(&self, entity: &Entity) -> Option<SpriteData> {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Sprite;
        let reg = get_context_registry().lock().unwrap();
        reg.get(self.context_id).and_then(|ctx| {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            ctx.world().get::<Sprite>(e).map(|s| {
                let (cw, ch) = s.custom_size.map_or((None, None), |sz| (Some(sz.x as f64), Some(sz.y as f64)));
                let (sx, sy, sw, sh) = s.source_rect.map_or((None,None,None,None), |r| (Some(r.x as f64), Some(r.y as f64), Some(r.width as f64), Some(r.height as f64)));
                SpriteData {
                    color: crate::types::Color { r: s.color.r as f64, g: s.color.g as f64, b: s.color.b as f64, a: s.color.a as f64 },
                    flip_x: s.flip_x, flip_y: s.flip_y,
                    anchor_x: s.anchor.x as f64, anchor_y: s.anchor.y as f64,
                    custom_width: cw, custom_height: ch,
                    source_rect_x: sx, source_rect_y: sy, source_rect_width: sw, source_rect_height: sh,
                }
            })
        })
    }

    #[napi]
    pub fn set_sprite(&self, entity: &Entity, data: SpriteData) {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Sprite;
        use goud_engine::core::math::{Color, Rect, Vec2};
        let mut reg = get_context_registry().lock().unwrap();
        if let Some(ctx) = reg.get_mut(self.context_id) {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            if let Some(s) = ctx.world_mut().get_mut::<Sprite>(e) {
                s.color = Color::rgba(data.color.r as f32, data.color.g as f32, data.color.b as f32, data.color.a as f32);
                s.flip_x = data.flip_x; s.flip_y = data.flip_y;
                s.anchor = Vec2::new(data.anchor_x as f32, data.anchor_y as f32);
                s.custom_size = match (data.custom_width, data.custom_height) { (Some(w), Some(h)) => Some(Vec2::new(w as f32, h as f32)), _ => None };
                s.source_rect = match (data.source_rect_x, data.source_rect_y, data.source_rect_width, data.source_rect_height) {
                    (Some(x), Some(y), Some(w), Some(h)) => Some(Rect::new(x as f32, y as f32, w as f32, h as f32)), _ => None };
            }
        }
    }

    #[napi]
    pub fn has_sprite(&self, entity: &Entity) -> bool {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Sprite;
        let reg = get_context_registry().lock().unwrap();
        reg.get(self.context_id).is_some_and(|ctx| {
            ctx.world().has::<Sprite>(goud_engine::ecs::Entity::from_bits(entity.bits))
        })
    }

    #[napi]
    pub fn remove_sprite(&self, entity: &Entity) -> bool {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Sprite;
        let mut reg = get_context_registry().lock().unwrap();
        reg.get_mut(self.context_id).is_some_and(|ctx| {
            ctx.world_mut().remove::<Sprite>(goud_engine::ecs::Entity::from_bits(entity.bits)).is_some()
        })
    }

    // =========================================================================
    // Name Component -- via FFI context registry
    // =========================================================================

    #[napi]
    pub fn add_name(&self, entity: &Entity, name: String) {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Name;
        let mut reg = get_context_registry().lock().unwrap();
        if let Some(ctx) = reg.get_mut(self.context_id) {
            let e = goud_engine::ecs::Entity::from_bits(entity.bits);
            ctx.world_mut().insert(e, Name::new(&name));
        }
    }

    #[napi]
    pub fn get_name(&self, entity: &Entity) -> Option<String> {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Name;
        let reg = get_context_registry().lock().unwrap();
        reg.get(self.context_id).and_then(|ctx| {
            ctx.world().get::<Name>(goud_engine::ecs::Entity::from_bits(entity.bits)).map(|n| n.as_str().to_string())
        })
    }

    #[napi]
    pub fn has_name(&self, entity: &Entity) -> bool {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Name;
        let reg = get_context_registry().lock().unwrap();
        reg.get(self.context_id).is_some_and(|ctx| {
            ctx.world().has::<Name>(goud_engine::ecs::Entity::from_bits(entity.bits))
        })
    }

    #[napi]
    pub fn remove_name(&self, entity: &Entity) -> bool {
        use goud_engine::ffi::context::get_context_registry;
        use goud_engine::ecs::components::Name;
        let mut reg = get_context_registry().lock().unwrap();
        reg.get_mut(self.context_id).is_some_and(|ctx| {
            ctx.world_mut().remove::<Name>(goud_engine::ecs::Entity::from_bits(entity.bits)).is_some()
        })
    }

    // =========================================================================
    // Collision -- via FFI (no context needed)
    // =========================================================================

    #[napi]
    pub fn collision_aabb_aabb(&self,
        ca_x: f64, ca_y: f64, hw_a: f64, hh_a: f64,
        cb_x: f64, cb_y: f64, hw_b: f64, hh_b: f64,
    ) -> Option<NapiContact> {
        let mut contact = std::mem::MaybeUninit::<GoudContact>::uninit();
        // SAFETY: Passing valid pointer to uninitialized memory for out-param.
        let hit = unsafe { goud_collision_aabb_aabb(
            ca_x as f32, ca_y as f32, hw_a as f32, hh_a as f32,
            cb_x as f32, cb_y as f32, hw_b as f32, hh_b as f32,
            contact.as_mut_ptr()) };
        if hit {
            // SAFETY: goud_collision_aabb_aabb wrote into contact on success.
            let c = unsafe { contact.assume_init() };
            Some(NapiContact { point_x: c.point_x as f64, point_y: c.point_y as f64,
                normal_x: c.normal_x as f64, normal_y: c.normal_y as f64, penetration: c.penetration as f64 })
        } else { None }
    }

    #[napi]
    pub fn collision_circle_circle(&self,
        ca_x: f64, ca_y: f64, ra: f64, cb_x: f64, cb_y: f64, rb: f64,
    ) -> Option<NapiContact> {
        let mut contact = std::mem::MaybeUninit::<GoudContact>::uninit();
        // SAFETY: Passing valid pointer to uninitialized memory for out-param.
        let hit = unsafe { goud_collision_circle_circle(
            ca_x as f32, ca_y as f32, ra as f32, cb_x as f32, cb_y as f32, rb as f32,
            contact.as_mut_ptr()) };
        if hit {
            // SAFETY: goud_collision_circle_circle wrote into contact on success.
            let c = unsafe { contact.assume_init() };
            Some(NapiContact { point_x: c.point_x as f64, point_y: c.point_y as f64,
                normal_x: c.normal_x as f64, normal_y: c.normal_y as f64, penetration: c.penetration as f64 })
        } else { None }
    }

    #[napi]
    pub fn collision_circle_aabb(&self,
        cx: f64, cy: f64, cr: f64, bx: f64, by: f64, bhw: f64, bhh: f64,
    ) -> Option<NapiContact> {
        let mut contact = std::mem::MaybeUninit::<GoudContact>::uninit();
        // SAFETY: Passing valid pointer to uninitialized memory for out-param.
        let hit = unsafe { goud_collision_circle_aabb(
            cx as f32, cy as f32, cr as f32, bx as f32, by as f32, bhw as f32, bhh as f32,
            contact.as_mut_ptr()) };
        if hit {
            // SAFETY: goud_collision_circle_aabb wrote into contact on success.
            let c = unsafe { contact.assume_init() };
            Some(NapiContact { point_x: c.point_x as f64, point_y: c.point_y as f64,
                normal_x: c.normal_x as f64, normal_y: c.normal_y as f64, penetration: c.penetration as f64 })
        } else { None }
    }

    #[napi]
    pub fn point_in_rect(&self, px: f64, py: f64, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
        goud_collision_point_in_rect(px as f32, py as f32, rx as f32, ry as f32, rw as f32, rh as f32)
    }
    #[napi]
    pub fn point_in_circle(&self, px: f64, py: f64, cx: f64, cy: f64, cr: f64) -> bool {
        goud_collision_point_in_circle(px as f32, py as f32, cx as f32, cy as f32, cr as f32)
    }
    #[napi]
    pub fn aabb_overlap(&self, ax1: f64, ay1: f64, ax2: f64, ay2: f64, bx1: f64, by1: f64, bx2: f64, by2: f64) -> bool {
        goud_collision_aabb_overlap(ax1 as f32, ay1 as f32, ax2 as f32, ay2 as f32, bx1 as f32, by1 as f32, bx2 as f32, by2 as f32)
    }
    #[napi]
    pub fn circle_overlap(&self, x1: f64, y1: f64, r1: f64, x2: f64, y2: f64, r2: f64) -> bool {
        goud_collision_circle_overlap(x1 as f32, y1 as f32, r1 as f32, x2 as f32, y2 as f32, r2 as f32)
    }
    #[napi]
    pub fn distance(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        goud_collision_distance(x1 as f32, y1 as f32, x2 as f32, y2 as f32) as f64
    }
    #[napi]
    pub fn distance_squared(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        goud_collision_distance_squared(x1 as f32, y1 as f32, x2 as f32, y2 as f32) as f64
    }

    // =========================================================================
    // 3D Renderer -- via FFI
    // =========================================================================

    #[napi]
    pub fn create_cube(&self, texture_id: u32, width: f64, height: f64, depth: f64) -> u32 {
        goud_renderer3d_create_cube(self.context_id, texture_id, width as f32, height as f32, depth as f32)
    }
    #[napi]
    pub fn create_plane(&self, texture_id: u32, width: f64, depth: f64) -> u32 {
        goud_renderer3d_create_plane(self.context_id, texture_id, width as f32, depth as f32)
    }
    #[napi]
    pub fn create_sphere(&self, texture_id: u32, diameter: f64, segments: u32) -> u32 {
        goud_renderer3d_create_sphere(self.context_id, texture_id, diameter as f32, segments)
    }
    #[napi]
    pub fn create_cylinder(&self, texture_id: u32, radius: f64, height: f64, segments: u32) -> u32 {
        goud_renderer3d_create_cylinder(self.context_id, texture_id, radius as f32, height as f32, segments)
    }
    #[napi]
    pub fn set_object_position(&self, id: u32, x: f64, y: f64, z: f64) -> bool {
        goud_renderer3d_set_object_position(self.context_id, id, x as f32, y as f32, z as f32)
    }
    #[napi]
    pub fn set_object_rotation(&self, id: u32, x: f64, y: f64, z: f64) -> bool {
        goud_renderer3d_set_object_rotation(self.context_id, id, x as f32, y as f32, z as f32)
    }
    #[napi]
    pub fn set_object_scale(&self, id: u32, x: f64, y: f64, z: f64) -> bool {
        goud_renderer3d_set_object_scale(self.context_id, id, x as f32, y as f32, z as f32)
    }
    #[napi]
    pub fn destroy_object(&self, id: u32) -> bool {
        goud_renderer3d_destroy_object(self.context_id, id)
    }
    #[napi]
    pub fn add_light(&self, lt: i32, px: f64, py: f64, pz: f64, dx: f64, dy: f64, dz: f64,
                     r: f64, g: f64, b: f64, intensity: f64, range: f64, spot_angle: f64) -> u32 {
        goud_renderer3d_add_light(self.context_id, lt,
            px as f32, py as f32, pz as f32, dx as f32, dy as f32, dz as f32,
            r as f32, g as f32, b as f32, intensity as f32, range as f32, spot_angle as f32)
    }
    #[napi]
    pub fn update_light(&self, lid: u32, lt: i32, px: f64, py: f64, pz: f64, dx: f64, dy: f64, dz: f64,
                        r: f64, g: f64, b: f64, intensity: f64, range: f64, spot_angle: f64) -> bool {
        goud_renderer3d_update_light(self.context_id, lid, lt,
            px as f32, py as f32, pz as f32, dx as f32, dy as f32, dz as f32,
            r as f32, g as f32, b as f32, intensity as f32, range as f32, spot_angle as f32)
    }
    #[napi]
    pub fn remove_light(&self, lid: u32) -> bool { goud_renderer3d_remove_light(self.context_id, lid) }
    #[napi]
    pub fn set_camera_position_3d(&self, x: f64, y: f64, z: f64) -> bool {
        goud_renderer3d_set_camera_position(self.context_id, x as f32, y as f32, z as f32)
    }
    #[napi]
    pub fn set_camera_rotation_3d(&self, pitch: f64, yaw: f64, roll: f64) -> bool {
        goud_renderer3d_set_camera_rotation(self.context_id, pitch as f32, yaw as f32, roll as f32)
    }
    #[napi]
    pub fn configure_grid(&self, enabled: bool, size: f64, divisions: u32) -> bool {
        goud_renderer3d_configure_grid(self.context_id, enabled, size as f32, divisions)
    }
    #[napi]
    pub fn set_grid_enabled(&self, enabled: bool) -> bool { goud_renderer3d_set_grid_enabled(self.context_id, enabled) }
    #[napi]
    pub fn configure_skybox(&self, enabled: bool, r: f64, g: f64, b: f64, a: f64) -> bool {
        goud_renderer3d_configure_skybox(self.context_id, enabled, r as f32, g as f32, b as f32, a as f32)
    }
    #[napi]
    pub fn configure_fog(&self, enabled: bool, r: f64, g: f64, b: f64, density: f64) -> bool {
        goud_renderer3d_configure_fog(self.context_id, enabled, r as f32, g as f32, b as f32, density as f32)
    }
    #[napi]
    pub fn set_fog_enabled(&self, enabled: bool) -> bool { goud_renderer3d_set_fog_enabled(self.context_id, enabled) }
    #[napi]
    pub fn render_3d(&self) -> bool { goud_renderer3d_render(self.context_id) }

    // =========================================================================
    // Timing / Stats (getters)
    // =========================================================================

    #[napi(getter)]
    pub fn delta_time(&self) -> f64 { self.last_delta_time as f64 }

    #[napi(getter)]
    pub fn fps(&self) -> f64 {
        if self.last_delta_time > 0.0 { (1.0 / self.last_delta_time) as f64 } else { 0.0 }
    }

    #[napi]
    pub fn update_frame(&mut self, dt: f64) {
        self.last_delta_time = dt as f32;
        self.frame_count += 1;
        self.total_time += dt;
    }

    #[napi(getter)]
    pub fn title(&self) -> String { self.title.clone() }

    #[napi(getter)]
    pub fn frame_count(&self) -> u32 { self.frame_count as u32 }

    #[napi(getter)]
    pub fn total_time(&self) -> f64 { self.total_time }

    #[napi(getter)]
    pub fn window_width(&self) -> u32 {
        let (mut w, mut h) = (0u32, 0u32);
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_window_get_size(self.context_id, &mut w, &mut h) };
        w
    }

    #[napi(getter)]
    pub fn window_height(&self) -> u32 {
        let (mut w, mut h) = (0u32, 0u32);
        // SAFETY: Passing valid mutable references as out-pointers.
        unsafe { goud_window_get_size(self.context_id, &mut w, &mut h) };
        h
    }

    #[napi(getter)]
    pub fn context_valid(&self) -> bool { self.context_id != GOUD_INVALID_CONTEXT_ID }

    /// Returns the raw FFI delta time from the last poll_events call.
    #[napi(getter)]
    pub fn ffi_delta_time(&self) -> f64 { goud_window_get_delta_time(self.context_id) as f64 }
}

// =============================================================================
// NativeEngineConfig -- Builder for GoudGame via FFI
// =============================================================================

#[napi(js_name = "NativeEngineConfig")]
pub struct NativeEngineConfig {
    handle: *mut std::ffi::c_void,
    title: String,
}

#[napi]
impl NativeEngineConfig {
    #[napi(constructor)]
    pub fn new() -> Self {
        let handle = goud_engine::ffi::engine_config::goud_engine_config_create();
        Self { handle, title: String::new() }
    }

    #[napi]
    pub fn set_title(&mut self, title: String) -> bool {
        if self.handle.is_null() { return false; }
        let c_title = match CString::new(title.clone()) {
            Ok(s) => s,
            Err(_) => return false,
        };
        self.title = title;
        // SAFETY: handle is valid, CString guarantees null-terminated.
        unsafe { goud_engine::ffi::engine_config::goud_engine_config_set_title(self.handle, c_title.as_ptr()) }
    }

    #[napi]
    pub fn set_size(&self, width: u32, height: u32) -> bool {
        if self.handle.is_null() { return false; }
        // SAFETY: handle is valid.
        unsafe { goud_engine::ffi::engine_config::goud_engine_config_set_size(self.handle, width, height) }
    }

    #[napi]
    pub fn set_vsync(&self, enabled: bool) -> bool {
        if self.handle.is_null() { return false; }
        // SAFETY: handle is valid.
        unsafe { goud_engine::ffi::engine_config::goud_engine_config_set_vsync(self.handle, enabled) }
    }

    #[napi]
    pub fn set_fullscreen(&self, enabled: bool) -> bool {
        if self.handle.is_null() { return false; }
        // SAFETY: handle is valid.
        unsafe { goud_engine::ffi::engine_config::goud_engine_config_set_fullscreen(self.handle, enabled) }
    }

    #[napi]
    pub fn set_target_fps(&self, fps: u32) -> bool {
        if self.handle.is_null() { return false; }
        // SAFETY: handle is valid.
        unsafe { goud_engine::ffi::engine_config::goud_engine_config_set_target_fps(self.handle, fps) }
    }

    #[napi]
    pub fn set_fps_overlay(&self, enabled: bool) -> bool {
        if self.handle.is_null() { return false; }
        // SAFETY: handle is valid.
        unsafe { goud_engine::ffi::engine_config::goud_engine_config_set_fps_overlay(self.handle, enabled) }
    }

    #[napi]
    pub fn build(&mut self) -> Result<GoudGame> {
        if self.handle.is_null() {
            return Err(Error::from_reason("EngineConfig already consumed"));
        }
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        // SAFETY: handle is valid and we take ownership.
        let context_id = unsafe { goud_engine::ffi::engine_config::goud_engine_create(handle) };
        if context_id == GOUD_INVALID_CONTEXT_ID {
            return Err(Error::from_reason("Failed to create engine from config"));
        }
        Ok(GoudGame {
            context_id,
            last_delta_time: 0.0,
            title: std::mem::take(&mut self.title),
            frame_count: 0,
            total_time: 0.0,
        })
    }

    #[napi]
    pub fn destroy(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: handle is valid and we own it.
            unsafe { goud_engine::ffi::engine_config::goud_engine_config_destroy(self.handle) };
            self.handle = std::ptr::null_mut();
        }
    }
}
"""


def gen_napi_rust_lib():
    """Generate lib.rs."""
    lines = [
        RUST_HEADER,
        "#[allow(dead_code)]",
        '#[path = "components.g.rs"]',
        "mod components;",
        '#[path = "entity.g.rs"]',
        "mod entity;",
        '#[path = "game.g.rs"]',
        "mod game;",
        "#[allow(dead_code)]",
        '#[path = "types.g.rs"]',
        "mod types;",
        "",
    ]
    write_generated(NATIVE_SRC / "lib.rs", "\n".join(lines))


def gen_napi_rust():
    gen_napi_rust_types()
    gen_napi_rust_entity()
    gen_napi_rust_components()
    gen_napi_rust_game()
    gen_napi_rust_lib()


def gen_errors():
    categories, codes = load_errors(schema)
    if not categories:
        return

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "export enum RecoveryClass {",
        "  Recoverable = 0,",
        "  Fatal = 1,",
        "  Degraded = 2,",
        "}",
        "",
        "/** Base error for all GoudEngine errors. */",
        "export class GoudError extends Error {",
        "  public readonly code: number;",
        "  public readonly category: string;",
        "  public readonly subsystem: string;",
        "  public readonly operation: string;",
        "  public readonly recovery: RecoveryClass;",
        "  public readonly recoveryHint: string;",
        "",
        "  constructor(",
        "    code: number,",
        "    message: string,",
        "    category: string,",
        "    subsystem: string,",
        "    operation: string,",
        "    recovery: RecoveryClass,",
        "    recoveryHint: string,",
        "  ) {",
        "    super(message);",
        "    this.name = new.target.name;",
        "    this.code = code;",
        "    this.category = category;",
        "    this.subsystem = subsystem;",
        "    this.operation = operation;",
        "    this.recovery = recovery;",
        "    this.recoveryHint = recoveryHint;",
        "",
        "    // Maintain proper prototype chain for instanceof checks",
        "    Object.setPrototypeOf(this, new.target.prototype);",
        "  }",
        "",
        "  /**",
        "   * Build the correct typed error subclass from a code and message.",
        "   * Subsystem and operation are optional context strings.",
        "   */",
        "  static fromCode(",
        "    code: number,",
        "    message: string,",
        '    subsystem: string = "",',
        '    operation: string = "",',
        "  ): GoudError {",
        "    const category = categoryFromCode(code);",
        "    const recovery = recoveryFromCategory(category);",
        "    const hint = hintFromCode(code);",
        "    const Subclass = CATEGORY_CLASS_MAP[category] ?? GoudError;",
        "",
        "    return new Subclass(",
        "      code, message, category, subsystem, operation, recovery, hint,",
        "    );",
        "  }",
        "}",
        "",
    ]

    # Category subclasses
    for cat in categories:
        cls = cat["base_class"]
        lines.append(f"export class {cls} extends GoudError {{}}")
    lines.append("")

    # CATEGORY_CLASS_MAP
    lines.append("const CATEGORY_CLASS_MAP: Record<string, typeof GoudError> = {")
    for cat in categories:
        lines.append(f"  {cat['name']}: {cat['base_class']},")
    lines += ["};", ""]

    # categoryFromCode
    lines.append("function categoryFromCode(code: number): string {")
    sorted_cats = sorted(categories, key=lambda c: c["range_start"], reverse=True)
    for cat in sorted_cats:
        lines.append(f'  if (code >= {cat["range_start"]}) return "{cat["name"]}";')
    lines += ['  return "Unknown";', "}", ""]

    # recoveryFromCategory - derive default recovery from category
    # Look at codes to find which categories are fatal by default
    fatal_cats = set()
    for c in codes:
        if c["recovery"] == "fatal":
            fatal_cats.add(c["category"])
    lines.append("/**")
    lines.append(" * Default recovery class derived from code range. This is a fallback")
    lines.append(" * for environments where the native FFI is not available (e.g., web).")
    lines.append(" * Desktop environments should prefer the value from")
    lines.append(" * goud_error_recovery_class.")
    lines.append(" */")
    lines.append("function recoveryFromCategory(category: string): RecoveryClass {")
    lines.append("  switch (category) {")
    for cat_name in sorted(fatal_cats):
        lines.append(f'    case "{cat_name}":')
    lines.append("      return RecoveryClass.Fatal;")
    lines.append("    default:")
    lines.append("      return RecoveryClass.Recoverable;")
    lines += ["  }", "}", ""]

    # hintFromCode
    lines.append("/** Static hint lookup matching the codegen schema. */")
    lines.append("function hintFromCode(code: number): string {")
    lines.append('  return HINTS[code] ?? "";')
    lines += ["}", ""]

    # HINTS map
    lines.append("const HINTS: Record<number, string> = {")
    for c in codes:
        lines.append(f'  {c["code"]}: "{c["hint"]}",')
    lines += ["};", ""]

    write_generated(GEN / "errors.g.ts", "\n".join(lines))


def gen_diagnostic():
    if "diagnostic" not in schema:
        return
    diag = schema["diagnostic"]
    cls = diag["class_name"]
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "/**",
        f" * {diag['doc']}",
        " *",
        " * In web/WASM builds these are no-ops.",
        " */",
        f"export class {cls} {{",
        "  private static _enabled = false;",
        "",
    ]
    for method in diag["methods"]:
        name = method["name"]
        ffi = method["ffi"]
        doc = method["doc"]

        if method.get("buffer_protocol"):
            lines += [
                f"  /** {doc} */",
                f"  static get {name}(): string {{",
                "    try {",
                "      const native = require('../node/index.g.js');",
                f"      if (typeof native.{ffi} === 'function') {{",
                f"        return native.{ffi}() ?? \"\";",
                "      }",
                "    } catch {",
                "      // Web/WASM fallback",
                "    }",
                '    return "";',
                "  }",
            ]
        elif method["returns"] == "void":
            lines += [
                f"  /** {doc} */",
                f"  static {name}({method['params'][0]['name']}: boolean): void {{",
                "    try {",
                "      const native = require('../node/index.g.js');",
                f"      if (typeof native.{ffi} === 'function') {{",
                f"        native.{ffi}({method['params'][0]['name']});",
                "      }",
                "    } catch {",
                "      // Web/WASM fallback",
                "    }",
                f"    {cls}._enabled = {method['params'][0]['name']};",
                "  }",
            ]
        elif method["returns"] == "bool":
            lines += [
                f"  /** {doc} */",
                f"  static get {name}(): boolean {{",
                "    try {",
                "      const native = require('../node/index.g.js');",
                f"      if (typeof native.{ffi} === 'function') {{",
                f"        return native.{ffi}();",
                "      }",
                "    } catch {",
                "      // Web/WASM fallback",
                "    }",
                f"    return {cls}._enabled;",
                "  }",
            ]
        lines.append("")

    lines.append("}")
    lines.append("")

    write_generated(GEN / "diagnostic.g.ts", "\n".join(lines))


if __name__ == "__main__":
    print("Generating TypeScript Node SDK...")
    gen_interface()
    gen_input()
    gen_math()
    gen_node_wrapper()
    gen_entry()
    gen_napi_rust()
    gen_errors()
    gen_diagnostic()
    print("TypeScript Node SDK generation complete.")
