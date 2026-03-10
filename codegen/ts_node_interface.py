#!/usr/bin/env python3
"""Interface and math generation helpers for the TypeScript Node SDK."""

from ts_node_shared import (
    GEN,
    HEADER_COMMENT,
    TS_EXCLUDE_METHODS,
    mapping,
    schema,
    to_camel,
    ts_iface_type,
    write_generated,
)


def gen_interface():
    tool = schema["tools"]["GoudGame"]
    lines = [f"// {HEADER_COMMENT}", "", "import type { PhysicsBackend2D } from './input.g.js';", ""]

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
    if schema["types"].get("PhysicsRaycastHit2D", {}).get("doc"):
        lines.append(f"/** {schema['types']['PhysicsRaycastHit2D']['doc']} */")
    lines.append("export interface IPhysicsRaycastHit2D { bodyHandle: number; colliderHandle: number; pointX: number; pointY: number; normalX: number; normalY: number; distance: number; }")
    if schema["types"].get("PhysicsCollisionEvent2D", {}).get("doc"):
        lines.append(f"/** {schema['types']['PhysicsCollisionEvent2D']['doc']} */")
    lines.append("export interface IPhysicsCollisionEvent2D { bodyA: number; bodyB: number; kind: number; }")
    fps_fields = schema["types"]["FpsStats"]["fields"]
    fps_str = "; ".join(f"{to_camel(f['name'])}: number" for f in fps_fields)
    if schema["types"]["FpsStats"].get("doc"):
        lines.append(f"/** {schema['types']['FpsStats']['doc']} */")
    lines.append(f"export interface IFpsStats {{ {fps_str}; }}")

    for cap_name in ["RenderCapabilities", "PhysicsCapabilities", "AudioCapabilities", "InputCapabilities", "NetworkCapabilities"]:
        cap_type = schema["types"][cap_name]
        cap_fields = []
        for f in cap_type["fields"]:
            ts_ft = "boolean" if f["type"] == "bool" else "number"
            cap_fields.append(f"{to_camel(f['name'])}: {ts_ft}")
        cap_str = "; ".join(cap_fields)
        iface_name = {
            "RenderCapabilities": "IRenderCapabilities",
            "PhysicsCapabilities": "IPhysicsCapabilities",
            "AudioCapabilities": "IAudioCapabilities",
            "InputCapabilities": "IInputCapabilities",
            "NetworkCapabilities": "INetworkCapabilities",
        }[cap_name]
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

    if schema["types"].get("UiStyle", {}).get("doc"):
        lines.append(f"/** {schema['types']['UiStyle']['doc']} */")
    lines.append("export interface IUiStyle {")
    lines.append("  backgroundColor?: IColor;")
    lines.append("  foregroundColor?: IColor;")
    lines.append("  borderColor?: IColor;")
    lines.append("  borderWidth?: number;")
    lines.append("  fontFamily?: string;")
    lines.append("  fontSize?: number;")
    lines.append("  texturePath?: string;")
    lines.append("  widgetSpacing?: number;")
    lines.append("}")
    lines.append("")

    if schema["types"].get("UiEvent", {}).get("doc"):
        lines.append(f"/** {schema['types']['UiEvent']['doc']} */")
    lines.append("export interface IUiEvent {")
    lines.append("  eventKind: number;")
    lines.append("  nodeId: number;")
    lines.append("  previousNodeId: number;")
    lines.append("  currentNodeId: number;")
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
            elif pt in schema["types"] or pt in schema["enums"]:
                param_strs.append(f"{pn}{opt}: {ts_iface_type(pt) if pt in schema['types'] else 'number'}")
            else:
                param_strs.append(f"{pn}{opt}: {ts_iface_type(pt)}")

        sig = ", ".join(param_strs)
        ts_ret = ts_iface_type(ret)
        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
        if method.get("async"):
            lines.append(f"  {mn}({sig}): Promise<{ts_ret}>;")
        else:
            lines.append(f"  {mn}({sig}): {ts_ret};")

    _anim_iface = [
        ("animationLayerStackCreate", [("entity", "IEntity")], "number"),
        ("animationLayerAdd", [("entity", "IEntity"), ("name", "string"), ("blendMode", "number")], "number"),
        ("animationLayerSetWeight", [("entity", "IEntity"), ("layerIndex", "number"), ("weight", "number")], "number"),
        ("animationLayerPlay", [("entity", "IEntity"), ("layerIndex", "number")], "number"),
        ("animationLayerSetClip", [("entity", "IEntity"), ("layerIndex", "number"), ("frameCount", "number"), ("frameDuration", "number"), ("mode", "number")], "number"),
        ("animationLayerAddFrame", [("entity", "IEntity"), ("layerIndex", "number"), ("x", "number"), ("y", "number"), ("w", "number"), ("h", "number")], "number"),
        ("animationLayerReset", [("entity", "IEntity"), ("layerIndex", "number")], "number"),
        ("animationClipAddEvent", [("entity", "IEntity"), ("frameIndex", "number"), ("name", "string"), ("payloadType", "number"), ("payloadInt", "number"), ("payloadFloat", "number"), ("payloadString?", "string | null")], "number"),
        ("animationEventsCount", [], "number"),
        ("animationEventsRead", [("index", "number")], "IAnimationEventData"),
    ]
    lines.append("  // Animation Layer Stack & Events")
    for mn, params, ret in _anim_iface:
        sig = ", ".join(f"{pn}: {pt}" for pn, pt in params)
        lines.append(f"  {mn}({sig}): {ret};")

    lines.append("}")
    lines.append("")
    lines.append("/** Data for a fired animation event */")
    lines.append("export interface IAnimationEventData {")
    lines.append("  entity: number;")
    lines.append("  name: string;")
    lines.append("  frameIndex: number;")
    lines.append("  payloadType: number;")
    lines.append("  payloadInt: number;")
    lines.append("  payloadFloat: number;")
    lines.append("  payloadString: string;")
    lines.append("}")
    lines.append("")

    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        ec_tool = schema["tools"]["EngineConfig"]
        if ec_tool.get("doc"):
            lines.append(f"/** {ec_tool['doc']} */")
        lines.append("export interface IEngineConfig {")
        for method in ec_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
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

    if "PhysicsWorld2D" in schema.get("tools", {}) and "PhysicsWorld2D" in mapping.get("tools", {}):
        pw2d_tool = schema["tools"]["PhysicsWorld2D"]
        if pw2d_tool.get("doc"):
            lines.append(f"/** {pw2d_tool['doc']} */")
        lines.append("export interface IPhysicsWorld2D {")
        for method in pw2d_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            ret = method.get("returns", "void")
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
            lines.append(f"  {mn}({ps}): {ts_iface_type(ret)};")
        lines.append("}")
        lines.append("")

    if "PhysicsWorld3D" in schema.get("tools", {}) and "PhysicsWorld3D" in mapping.get("tools", {}):
        pw3d_tool = schema["tools"]["PhysicsWorld3D"]
        if pw3d_tool.get("doc"):
            lines.append(f"/** {pw3d_tool['doc']} */")
        lines.append("export interface IPhysicsWorld3D {")
        for method in pw3d_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            ret = method.get("returns", "void")
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
            lines.append(f"  {mn}({ps}): {ts_iface_type(ret)};")
        lines.append("}")
        lines.append("")

    if "UiManager" in schema.get("tools", {}) and "UiManager" in mapping.get("tools", {}):
        ui_tool = schema["tools"]["UiManager"]
        if ui_tool.get("doc"):
            lines.append(f"/** {ui_tool['doc']} */")
        lines.append("export interface IUiManager {")
        for method in ui_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            ret = method.get("returns", "void")
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            ps = []
            for p in params:
                pn = to_camel(p["name"])
                pt = p["type"]
                if pt == "UiStyle":
                    ps.append(f"{pn}: IUiStyle")
                elif pt in schema.get("enums", {}):
                    ps.append(f"{pn}: number")
                else:
                    ps.append(f"{pn}: {ts_iface_type(pt)}")
            lines.append(f"  {mn}({', '.join(ps)}): {ts_iface_type(ret)};")
        lines.append("  createPanel(): number;")
        lines.append("  createLabel(text: string): number;")
        lines.append("  createButton(enabled?: boolean): number;")
        lines.append("  createImage(path: string): number;")
        lines.append("  createSlider(min: number, max: number, value: number, enabled?: boolean): number;")
        lines.append("}")
        lines.append("")

    write_generated(GEN / "types" / "engine.g.ts", "\n".join(lines))


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
                interps = ", ".join(f"this.{fn} + (other.{fn} - this.{fn}) * t" for fn in field_names)
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
