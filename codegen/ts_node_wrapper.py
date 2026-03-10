#!/usr/bin/env python3
"""Wrapper and entry generation helpers for the TypeScript Node SDK."""

from ts_node_shared import (
    GEN,
    HEADER_COMMENT,
    TS_EXCLUDE_METHODS,
    mapping,
    schema,
    to_camel,
    ts_iface_type,
    ts_type,
    write_generated,
)

NATIVE_KNOWN_METHODS = {
    "shouldClose", "close", "destroy", "beginFrame", "endFrame",
    "loadTexture", "destroyTexture", "loadFont", "destroyFont",
    "drawSprite", "drawQuad", "drawText",
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
    "physicsRaycastEx", "physicsCollisionEventsCount",
    "physicsCollisionEventsRead", "physicsSetCollisionCallback",
    "createCube", "createPlane", "createSphere", "createCylinder",
    "setObjectPosition", "setObjectRotation", "setObjectScale", "destroyObject",
    "addLight", "updateLight", "removeLight",
    "setCameraPosition3D", "setCameraRotation3D",
    "configureGrid", "setGridEnabled", "configureSkybox", "configureFog", "setFogEnabled",
    "render3D", "isAliveBatch", "despawnBatch",
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
        "import type { IGoudGame, IEntity, IColor, IVec2, IVec3, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, IPhysicsWorld2D, IPhysicsWorld3D } from '../types/engine.g.js';",
        "import { PhysicsBackend2D } from '../types/input.g.js';",
        "import { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Key, MouseButton, PhysicsBackend2D } from '../types/input.g.js';",
        "export type { IGoudGame, IEntity, IColor, IVec2, IVec3, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, IPhysicsWorld2D, IPhysicsWorld3D } from '../types/engine.g.js';",
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
        if prop.get("doc"):
            lines.append(f"  /** {prop['doc']} */")
        lines.append(f"  get {pn}(): {ts_type(prop['type'])} {{ return this.native.{pn}; }}")
    lines.append("")

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
                param_strs.append(f"{pn}{opt}: number")
                call_args.append(f"{pn} ?? {p.get('default')}" if can_be_optional[i] else pn)
            else:
                param_strs.append(f"{pn}{opt}: {ts_iface_type(pt)}")
                call_args.append(f"{pn} ?? {p.get('default')}" if can_be_optional[i] else pn)

        sig = ", ".join(param_strs)
        ts_ret = ts_iface_type(ret)
        if method.get("doc"):
            lines.append(f"  /** {method['doc']} */")
        lines.append(f"  {'async ' if method.get('async') else ''}{mn}({sig}): {'Promise<' + ts_ret + '>' if method.get('async') else ts_ret} {{")

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
        elif mn == "drawText":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    return this.native.drawText(fontHandle, text, x, y, fontSize, alignment, maxWidth, lineSpacing, direction, c.r, c.g, c.b, c.a);")
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
        elif mn == "loadFont":
            lines.append("    return this.native.loadFont(path);")
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

    lines += [
        "  loadScene(name: string, json: string): number {",
        "    return (this.native as any).loadScene(name, json);",
        "  }",
        "",
        "  unloadScene(name: string): boolean {",
        "    return (this.native as any).unloadScene(name);",
        "  }",
        "",
        "  setActiveScene(sceneId: number, active: boolean): boolean {",
        "    return (this.native as any).setActiveScene(sceneId, active);",
        "  }",
        "",
        "  // Animation Layer Stack & Events",
    ]

    _anim_wrappers = [
        ("animationLayerStackCreate", "entity: IEntity", "return (this.native as any).animationLayerStackCreate(entity as unknown as NativeEntity);", "number"),
        ("animationLayerAdd", "entity: IEntity, name: string, blendMode: number", "return (this.native as any).animationLayerAdd(entity as unknown as NativeEntity, name, blendMode);", "number"),
        ("animationLayerSetWeight", "entity: IEntity, layerIndex: number, weight: number", "return (this.native as any).animationLayerSetWeight(entity as unknown as NativeEntity, layerIndex, weight);", "number"),
        ("animationLayerPlay", "entity: IEntity, layerIndex: number", "return (this.native as any).animationLayerPlay(entity as unknown as NativeEntity, layerIndex);", "number"),
        ("animationLayerSetClip", "entity: IEntity, layerIndex: number, frameCount: number, frameDuration: number, mode: number", "return (this.native as any).animationLayerSetClip(entity as unknown as NativeEntity, layerIndex, frameCount, frameDuration, mode);", "number"),
        ("animationLayerAddFrame", "entity: IEntity, layerIndex: number, x: number, y: number, w: number, h: number", "return (this.native as any).animationLayerAddFrame(entity as unknown as NativeEntity, layerIndex, x, y, w, h);", "number"),
        ("animationLayerReset", "entity: IEntity, layerIndex: number", "return (this.native as any).animationLayerReset(entity as unknown as NativeEntity, layerIndex);", "number"),
        ("animationClipAddEvent", "entity: IEntity, frameIndex: number, name: string, payloadType: number, payloadInt: number, payloadFloat: number, payloadString?: string | null", "return (this.native as any).animationClipAddEvent(entity as unknown as NativeEntity, frameIndex, name, payloadType, payloadInt, payloadFloat, payloadString ?? null);", "number"),
        ("animationEventsCount", "", "return (this.native as any).animationEventsCount();", "number"),
        ("animationEventsRead", "index: number", "return (this.native as any).animationEventsRead(index) as unknown as IAnimationEventData;", "IAnimationEventData"),
    ]
    for mn, sig, body, ret in _anim_wrappers:
        lines.append(f"  {mn}({sig}): {ret} {{")
        lines.append(f"    {body}")
        lines.append("  }")
        lines.append("")

    lines.append("}")
    lines.append("")

    if "PhysicsWorld2D" in schema.get("tools", {}) and "PhysicsWorld2D" in mapping.get("tools", {}):
        pw2d_tool = schema["tools"]["PhysicsWorld2D"]
        if pw2d_tool.get("doc"):
            lines.append(f"/** {pw2d_tool['doc']} */")
        lines.append("export class PhysicsWorld2D implements IPhysicsWorld2D {")
        lines.append("  private native: any;")
        lines.append("")
        lines.append("  constructor(gravityX: number, gravityY: number, backend: number = 0) {")
        lines.append("    const { NativePhysicsWorld2D } = require('../../../index');")
        lines.append("    this.native = new NativePhysicsWorld2D(gravityX, gravityY, backend);")
        lines.append("  }")
        lines.append("")
        for method in pw2d_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            ret = method.get("returns", "void")
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
            args = ", ".join(to_camel(p["name"]) for p in params)
            lines.append(f"  {mn}({ps}): {ts_iface_type(ret)} {{")
            if ret == "void":
                lines.append(f"    this.native.{mn}({args});")
            else:
                lines.append(f"    return this.native.{mn}({args});")
            lines.append("  }")
            lines.append("")
        lines.append("}")
        lines.append("")

    if "PhysicsWorld3D" in schema.get("tools", {}) and "PhysicsWorld3D" in mapping.get("tools", {}):
        pw3d_tool = schema["tools"]["PhysicsWorld3D"]
        if pw3d_tool.get("doc"):
            lines.append(f"/** {pw3d_tool['doc']} */")
        lines.append("export class PhysicsWorld3D implements IPhysicsWorld3D {")
        lines.append("  private native: any;")
        lines.append("")
        lines.append("  constructor(gravityX: number, gravityY: number, gravityZ: number) {")
        lines.append("    const { NativePhysicsWorld3D } = require('../../../index');")
        lines.append("    this.native = new NativePhysicsWorld3D(gravityX, gravityY, gravityZ);")
        lines.append("  }")
        lines.append("")
        for method in pw3d_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            ret = method.get("returns", "void")
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
            args = ", ".join(to_camel(p["name"]) for p in params)
            lines.append(f"  {mn}({ps}): {ts_iface_type(ret)} {{")
            if ret == "void":
                lines.append(f"    this.native.{mn}({args});")
            else:
                lines.append(f"    return this.native.{mn}({args});")
            lines.append("  }")
            lines.append("")
        lines.append("}")
        lines.append("")

    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        ec_tool = schema["tools"]["EngineConfig"]
        lines += ["import type { IEngineConfig } from '../types/engine.g.js';", ""]
        if ec_tool.get("doc"):
            lines.append(f"/** {ec_tool['doc']} */")
        lines += [
            "export class EngineConfig implements IEngineConfig {",
            "  private native: any;",
            "",
            "  constructor() {",
            "    const { NativeEngineConfig } = require('../../../index');",
            "    this.native = new NativeEngineConfig();",
            "  }",
            "",
        ]
        for method in ec_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            if method.get("doc"):
                lines.append(f"  /** {method['doc']} */")
            if mn == "build":
                lines += [
                    "  build(): GoudGame {",
                    "    const ctx = this.native.build();",
                    "    const game = Object.create(GoudGame.prototype);",
                    "    game.native = ctx;",
                    "    return game;",
                    "  }",
                ]
            elif mn == "destroy":
                lines += ["  destroy(): void {", "    this.native.destroy();", "  }"]
            else:
                ps = ", ".join(f"{to_camel(p['name'])}: {ts_iface_type(p['type'])}" for p in params)
                args = ", ".join(to_camel(p["name"]) for p in params)
                lines += [f"  {mn}({ps}): EngineConfig {{", f"    this.native.{mn}({args});", "    return this;", "  }"]
            lines.append("")
        lines += ["}", ""]

    write_generated(GEN / "node" / "index.g.ts", "\n".join(lines))


def gen_entry():
    has_engine_config = "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {})
    has_physics_world_2d = "PhysicsWorld2D" in schema.get("tools", {}) and "PhysicsWorld2D" in mapping.get("tools", {})
    has_physics_world_3d = "PhysicsWorld3D" in schema.get("tools", {}) and "PhysicsWorld3D" in mapping.get("tools", {})
    ec_export = ", EngineConfig" if has_engine_config else ""
    ec_type_export = ", IEngineConfig" if has_engine_config else ""
    pw2d_export = ", PhysicsWorld2D" if has_physics_world_2d else ""
    pw2d_type_export = ", IPhysicsWorld2D" if has_physics_world_2d else ""
    pw3d_export = ", PhysicsWorld3D" if has_physics_world_3d else ""
    pw3d_type_export = ", IPhysicsWorld3D" if has_physics_world_3d else ""
    errors_section = schema.get("errors", {})
    error_names = ["GoudError"]
    for cat in errors_section.get("categories", []):
        error_names.append(cat["base_class"])
    error_names.append("RecoveryClass")

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        f"export {{ GoudGame{ec_export}{pw2d_export}{pw3d_export}, Color, Vec2, Vec3, Key, MouseButton, PhysicsBackend2D }} from './node/index.g.js';",
        f"export type {{ IGoudGame{ec_type_export}{pw2d_type_export}{pw3d_type_export}, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities }} from './types/engine.g.js';",
        "export type { Rect } from './types/math.g.js';",
        f"export {{ {', '.join(error_names)} }} from './errors.g.js';",
    ]
    if "diagnostic" in schema:
        lines.append("export { DiagnosticMode } from './diagnostic.g.js';")
    lines.append("")
    write_generated(GEN / "index.g.ts", "\n".join(lines))
