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
from ts_node_wrapper_sections import (
    append_animation_wrappers,
    append_context_wrapper,
    append_engine_config_wrapper,
    append_physics_world_2d_wrapper,
    append_physics_world_3d_wrapper,
    append_ui_manager_wrapper,
    has_engine_config,
    has_physics_world_2d,
    has_physics_world_3d,
    has_ui_manager,
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
    "networkHost", "networkConnect", "networkConnectWithPeer", "networkDisconnect", "networkSend",
    "networkReceive", "networkReceivePacket", "networkPoll", "getNetworkStats", "networkPeerCount",
    "setNetworkSimulation", "clearNetworkSimulation",
    "setNetworkOverlayHandle", "clearNetworkOverlayHandle",
    "checkHotSwapShortcut",
    "update", "render", "nodeCount",
    "createNode", "removeNode", "setParent", "getParent", "getChildCount", "getChildAt",
    "setWidget", "setStyle", "setLabelText", "setButtonEnabled", "setImageTexturePath",
    "setSlider", "eventCount", "eventRead",
}


def gen_node_wrapper():
    tool = schema["tools"]["GoudGame"]
    tm = mapping["tools"]["GoudGame"]
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "import {",
        "  GoudGame as NativeGoudGame,",
        "  GoudContext as NativeGoudContext,",
        "  UiManager as NativeUiManager,",
        "  Entity as NativeEntity,",
        "  type GameConfig,",
        "} from '../../../index';",
        "",
        "import type { IGoudGame, IUiManager, IUiStyle, IUiEvent, UiNodeId, IEntity, IColor, IVec2, IVec3, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig, IPhysicsWorld2D, IPhysicsWorld3D } from '../types/engine.g.js';",
        "import { PhysicsBackend2D } from '../types/input.g.js';",
        "import { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Key, MouseButton, PhysicsBackend2D } from '../types/input.g.js';",
        "export type { IGoudGame, IUiManager, IUiStyle, IUiEvent, UiNodeId, IEntity, IColor, IVec2, IVec3, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig, IPhysicsWorld2D, IPhysicsWorld3D } from '../types/engine.g.js';",
        "",
        "export interface INetworkConnectResult { handle: number; peerId: number; }",
        "export interface INetworkPacket { peerId: number; data: Uint8Array; }",
        "export interface IGoudContext {",
        "  destroy(): boolean;",
        "  isValid(): boolean;",
        "  getNetworkCapabilities(): INetworkCapabilities;",
        "  networkHost(protocol: number, port: number): number;",
        "  networkConnect(protocol: number, address: string, port: number): number;",
        "  networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult;",
        "  networkDisconnect(handle: number): number;",
        "  networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number;",
        "  networkReceive(handle: number): Uint8Array;",
        "  networkReceivePacket(handle: number): INetworkPacket | null;",
        "  networkPoll(handle: number): number;",
        "  getNetworkStats(handle: number): INetworkStats;",
        "  networkPeerCount(handle: number): number;",
        "  setNetworkSimulation(handle: number, config: INetworkSimulationConfig): number;",
        "  clearNetworkSimulation(handle: number): number;",
        "  setNetworkOverlayHandle(handle: number): number;",
        "  clearNetworkOverlayHandle(): number;",
        "}",
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
        elif mn == "getNetworkStats":
            lines.append("    return this.native.getNetworkStats(handle) as unknown as INetworkStats;")
        elif mn == "networkConnectWithPeer":
            lines.append("    return this.native.networkConnectWithPeer(protocol, address, port) as unknown as INetworkConnectResult;")
        elif mn == "networkSend":
            lines.append("    return this.native.networkSend(handle, peerId, Buffer.from(data), channel);")
        elif mn == "networkReceivePacket":
            lines.append("    return this.native.networkReceivePacket(handle) as unknown as INetworkPacket | null;")
        elif mn == "setNetworkSimulation":
            lines.append("    return this.native.setNetworkSimulation(handle, {")
            lines.append("      one_way_latency_ms: config.oneWayLatencyMs,")
            lines.append("      jitter_ms: config.jitterMs,")
            lines.append("      packet_loss_percent: config.packetLossPercent,")
            lines.append("    } as unknown as INetworkSimulationConfig);")
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

    append_animation_wrappers(lines)

    lines.append("}")
    lines.append("")

    append_context_wrapper(lines)

    if has_physics_world_2d():
        append_physics_world_2d_wrapper(lines)

    if has_physics_world_3d():
        append_physics_world_3d_wrapper(lines)

    if has_engine_config():
        append_engine_config_wrapper(lines)

    if has_ui_manager():
        append_ui_manager_wrapper(lines)

    write_generated(GEN / "node" / "index.g.ts", "\n".join(lines))


def gen_entry():
    has_engine_config_export = has_engine_config()
    has_ui_manager_export = has_ui_manager()
    has_physics_world_2d_export = has_physics_world_2d()
    has_physics_world_3d_export = has_physics_world_3d()
    ec_export = ", EngineConfig" if has_engine_config_export else ""
    ec_type_export = ", IEngineConfig" if has_engine_config_export else ""
    ui_export = ", UiManager" if has_ui_manager_export else ""
    ui_type_export = ", IUiManager, IUiStyle, IUiEvent, UiNodeId" if has_ui_manager_export else ""
    pw2d_export = ", PhysicsWorld2D" if has_physics_world_2d_export else ""
    pw2d_type_export = ", IPhysicsWorld2D" if has_physics_world_2d_export else ""
    pw3d_export = ", PhysicsWorld3D" if has_physics_world_3d_export else ""
    pw3d_type_export = ", IPhysicsWorld3D" if has_physics_world_3d_export else ""
    errors_section = schema.get("errors", {})
    error_names = ["GoudError"]
    for cat in errors_section.get("categories", []):
        error_names.append(cat["base_class"])
    error_names.append("RecoveryClass")

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        f"export {{ GoudGame, GoudContext{ec_export}{ui_export}{pw2d_export}{pw3d_export}, Color, Vec2, Vec3, Key, MouseButton, PhysicsBackend2D }} from './node/index.g.js';",
        f"export type {{ IGoudGame{ec_type_export}{ui_type_export}{pw2d_type_export}{pw3d_type_export}, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig }} from './types/engine.g.js';",
        "export type { IGoudContext, INetworkConnectResult, INetworkPacket } from './node/index.g.js';",
        "export type { Rect } from './types/math.g.js';",
        f"export {{ {', '.join(error_names)} }} from './errors.g.js';",
    ]
    if "diagnostic" in schema:
        lines.append("export { DiagnosticMode } from './diagnostic.g.js';")
    lines.append("")
    write_generated(GEN / "index.g.ts", "\n".join(lines))
