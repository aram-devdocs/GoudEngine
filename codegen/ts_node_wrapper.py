#!/usr/bin/env python3
"""Wrapper and entry generation helpers for the TypeScript Node SDK."""

from ts_node_shared import (
    GEN,
    TS,
    HEADER_COMMENT,
    TS_EXCLUDE_METHODS,
    mapping,
    schema,
    to_camel,
    ts_param_name,
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
    "drawSpriteRect", "drawSpriteBatch", "drawTextBatch", "setViewport", "enableDepthTest", "disableDepthTest",
    "clearDepth", "disableBlending", "getRenderStats",
    "getFpsStats", "getRenderMetrics", "setFpsOverlayEnabled", "setFpsUpdateInterval", "setFpsOverlayCorner",
    "getDebuggerSnapshotJson", "getDebuggerManifestJson",
    "setDebuggerPaused", "stepDebugger", "setDebuggerTimeScale", "setDebuggerDebugDrawEnabled",
    "injectDebuggerKeyEvent", "injectDebuggerMouseButton", "injectDebuggerMousePosition",
    "injectDebuggerScroll",
    "setDebuggerProfilingEnabled", "setDebuggerSelectedEntity", "clearDebuggerSelectedEntity",
    "getMemorySummary", "captureDebuggerFrame", "startDebuggerRecording",
    "stopDebuggerRecording", "startDebuggerReplay", "stopDebuggerReplay",
    "getDebuggerReplayStatusJson", "getDebuggerMetricsTraceJson",
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
        "import type { IGoudGame, IUiManager, IUiStyle, IUiEvent, UiNodeId, IEntity, IColor, IVec2, IVec3, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderMetrics, IDebuggerConfig, IContextConfig, IMemoryCategoryStats, IMemorySummary, IDebuggerCapture, IDebuggerReplayArtifact, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IPreloadAssetRequest, IPreloadOptions, IPreloadProgress, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig, IPhysicsWorld2D, IPhysicsWorld3D, IP2pMeshConfig, IRollbackConfig, PreloadAssetInput, PreloadAssetKind, ISpriteCmd, ITextCmd } from '../types/engine.g.js';",
        "import { PhysicsBackend2D, RenderBackendKind, WindowBackendKind } from '../types/input.g.js';",
        "import { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Key, MouseButton, PhysicsBackend2D, RenderBackendKind, WindowBackendKind } from '../types/input.g.js';",
        "export type { IGoudGame, IUiManager, IUiStyle, IUiEvent, UiNodeId, IEntity, IColor, IVec2, IVec3, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderMetrics, IDebuggerConfig, IContextConfig, IMemoryCategoryStats, IMemorySummary, IDebuggerCapture, IDebuggerReplayArtifact, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig, IPhysicsWorld2D, IPhysicsWorld3D, IP2pMeshConfig, IRollbackConfig } from '../types/engine.g.js';",
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
        "  getDebuggerSnapshotJson(): string;",
        "  getDebuggerManifestJson(): string;",
        "  setDebuggerPaused(paused: boolean): void;",
        "  stepDebugger(kind: number, count: number): void;",
        "  setDebuggerTimeScale(scale: number): void;",
        "  setDebuggerDebugDrawEnabled(enabled: boolean): void;",
        "  injectDebuggerKeyEvent(key: number, pressed: boolean): void;",
        "  injectDebuggerMouseButton(button: number, pressed: boolean): void;",
        "  injectDebuggerMousePosition(position: IVec2): void;",
        "  injectDebuggerScroll(delta: IVec2): void;",
        "  setDebuggerProfilingEnabled(enabled: boolean): void;",
        "  setDebuggerSelectedEntity(entityId: number): void;",
        "  clearDebuggerSelectedEntity(): void;",
        "  getMemorySummary(): IMemorySummary;",
        "  captureDebuggerFrame(): IDebuggerCapture;",
        "  startDebuggerRecording(): void;",
        "  stopDebuggerRecording(): IDebuggerReplayArtifact;",
        "  startDebuggerReplay(recording: Uint8Array): void;",
        "  stopDebuggerReplay(): void;",
        "  getDebuggerReplayStatusJson(): string;",
        "  getDebuggerMetricsTraceJson(): string;",
        "}",
        "",
        "interface NativeEngineConfigBinding {",
        "  build(): unknown;",
        "  destroy(): void;",
        "  setRenderBackend(backend: number): unknown;",
        "  setWindowBackend(backend: number): unknown;",
        "  setDebugger(debuggerConfig: IDebuggerConfig): unknown;",
        "}",
        "",
        "interface NativeBindings {",
        "  GoudGame: new (config?: { width?: number; height?: number; title?: string }) => IGoudGame;",
        "  GoudContext: new (config?: Record<string, unknown>) => IGoudContext;",
        "  NativePhysicsWorld2D: new (gravityX: number, gravityY: number, backend?: number) => IPhysicsWorld2D;",
        "  NativePhysicsWorld3D: new (gravityX: number, gravityY: number, gravityZ: number) => IPhysicsWorld3D;",
        "  NativeEngineConfig: new () => NativeEngineConfigBinding;",
        "  UiManager: new () => IUiManager;",
        "}",
        "",
        "function getNativeBindings(): NativeBindings {",
        "  return eval('require')(\"../../../index\") as NativeBindings;",
        "}",
        "",
        "const PRELOAD_TEXTURE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp', 'tga', 'dds']);",
        "const PRELOAD_FONT_EXTENSIONS = new Set(['ttf', 'otf', 'woff', 'woff2', 'fnt']);",
        "",
        "function detectPreloadKind(path: string): PreloadAssetKind {",
        "  const ext = path.split('.').pop()?.toLowerCase() ?? '';",
        "  if (PRELOAD_TEXTURE_EXTENSIONS.has(ext)) return 'texture';",
        "  if (PRELOAD_FONT_EXTENSIONS.has(ext)) return 'font';",
        "  throw new Error(`Unsupported preload asset type for path: ${path}`);",
        "}",
        "",
        "function normalizePreloadAsset(asset: PreloadAssetInput): Required<IPreloadAssetRequest> {",
        "  if (typeof asset === 'string') {",
        "    return { path: asset, kind: detectPreloadKind(asset) };",
        "  }",
        "  return { path: asset.path, kind: asset.kind ?? detectPreloadKind(asset.path) };",
        "}",
        "",
        "function mapMemoryCategoryStats(stats: any): IMemoryCategoryStats {",
        "  return {",
        "    currentBytes: stats.current_bytes ?? stats.currentBytes ?? 0,",
        "    peakBytes: stats.peak_bytes ?? stats.peakBytes ?? 0,",
        "  };",
        "}",
        "",
        "function mapMemorySummary(summary: any): IMemorySummary {",
        "  return {",
        "    rendering: mapMemoryCategoryStats(summary.rendering),",
        "    assets: mapMemoryCategoryStats(summary.assets),",
        "    ecs: mapMemoryCategoryStats(summary.ecs),",
        "    ui: mapMemoryCategoryStats(summary.ui),",
        "    audio: mapMemoryCategoryStats(summary.audio),",
        "    network: mapMemoryCategoryStats(summary.network),",
        "    debugger: mapMemoryCategoryStats(summary.debugger),",
        "    other: mapMemoryCategoryStats(summary.other),",
        "    totalCurrentBytes: summary.total_current_bytes ?? summary.totalCurrentBytes ?? 0,",
        "    totalPeakBytes: summary.total_peak_bytes ?? summary.totalPeakBytes ?? 0,",
        "  };",
        "}",
        "",
    ]
    if tool.get("doc"):
        lines.append(f"/** {tool['doc']} */")
    lines += [
        "export class GoudGame implements IGoudGame {",
        "  private native: any;",
        "  private readonly preloadedTextures = new Map<string, number>();",
        "  private readonly preloadedFonts = new Map<string, number>();",
        "  private readonly texturePathByHandle = new Map<number, string>();",
        "  private readonly fontPathByHandle = new Map<number, string>();",
        "  private preloadInFlight = false;",
        "",
        "  constructor(config?: { width?: number; height?: number; title?: string }) {",
        "    this.native = new (getNativeBindings().GoudGame)(config as any);",
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
            pn = ts_param_name(p["name"])
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
            lines.append("    if (this.preloadInFlight) {")
            lines.append("      throw new Error('game.preload(...) must finish before game.run() starts.');")
            lines.append("    }")
            lines.append("    while (!this.native.shouldClose()) {")
            lines.append("      this.native.beginFrame();")
            lines.append("      update(this.native.deltaTime);")
            lines.append("      this.native.endFrame();")
            lines.append("    }")
        elif mn == "runWithFixedUpdate":
            lines.append("    if (this.preloadInFlight) {")
            lines.append("      throw new Error('game.preload(...) must finish before game.runWithFixedUpdate() starts.');")
            lines.append("    }")
            lines.append("    while (!this.native.shouldClose()) {")
            lines.append("      this.native.beginFrame();")
            lines.append("      if (this.native.fixedTimestepBegin()) {")
            lines.append("        while (this.native.fixedTimestepStep()) {")
            lines.append("          fixedUpdate(this.native.fixedTimestepDt());")
            lines.append("        }")
            lines.append("      }")
            lines.append("      update(this.native.deltaTime);")
            lines.append("      this.native.endFrame();")
            lines.append("    }")
        elif mn == "setFixedTimestep":
            lines.append("    this.native.fixedTimestepSet(stepSize);")
        elif mn == "setMaxFixedSteps":
            lines.append("    this.native.fixedTimestepSetMaxSteps(maxSteps);")
        elif mn == "drawSprite":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    this.native.drawSprite(texture, x, y, width, height, rotation, c.r, c.g, c.b, c.a);")
        elif mn == "drawSpriteRect":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    return this.native.drawSpriteRect(texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, srcMode, c.r, c.g, c.b, c.a);")
        elif mn == "drawSpriteBatch":
            lines.append("    return this.native.drawSpriteBatch(cmds);")
        elif mn == "drawTextBatch":
            lines.append("    return this.native.drawTextBatch(cmds);")
        elif mn == "drawQuad":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    this.native.drawQuad(x, y, width, height, c.r, c.g, c.b, c.a);")
        elif mn == "drawText":
            lines.append("    const c = color ?? Color.white();")
            lines.append("    return this.native.drawText(fontHandle, text, x, y, fontSize, alignment, maxWidth, lineSpacing, direction, c.r, c.g, c.b, c.a);")
        elif mn in ("getMousePosition", "getMouseDelta", "getScrollDelta"):
            lines.append(f"    const value = this.native.{mn}();")
            lines.append("    if (value && typeof value === 'object' && 'x' in value && 'y' in value) {")
            lines.append("      return { x: Number(value.x), y: Number(value.y) };")
            lines.append("    }")
            lines.append("    return { x: value[0], y: value[1] };")
        elif mn == "spawnEmpty":
            lines.append("    return this.native.spawnEmpty() as unknown as IEntity;")
        elif mn == "spawnBatch":
            lines.append("    const arr = this.native.spawnBatch(count);")
            lines.append("    return Array.from(arr) as unknown as IEntity[];")
        elif mn == "despawn":
            lines.append("    return this.native.despawn(entity as any);")
        elif mn == "despawnBatch":
            lines.append("    return (this.native as any).despawnBatch(entities as any);")
        elif mn == "isAlive":
            lines.append("    return this.native.isAlive(entity as any);")
        elif mn == "addTransform2d":
            lines.append("    this.native.addTransform2d(entity as any, transform as any);")
        elif mn == "getTransform2d":
            lines.append("    return this.native.getTransform2d(entity as any) ?? null;")
        elif mn == "setTransform2d":
            lines.append("    this.native.setTransform2d(entity as any, transform as any);")
        elif mn == "hasTransform2d":
            lines.append("    return this.native.hasTransform2d(entity as any);")
        elif mn == "removeTransform2d":
            lines.append("    return this.native.removeTransform2d(entity as any);")
        elif mn == "addSprite":
            lines.append("    this.native.addSprite(entity as any, sprite as any);")
        elif mn == "getSprite":
            lines.append("    const raw = this.native.getSprite(entity as any);")
            lines.append("    if (!raw) return null;")
            lines.append("    return raw as unknown as ISpriteData;")
        elif mn == "setSprite":
            lines.append("    this.native.setSprite(entity as any, sprite as any);")
        elif mn == "hasSprite":
            lines.append("    return this.native.hasSprite(entity as any);")
        elif mn == "removeSprite":
            lines.append("    return this.native.removeSprite(entity as any);")
        elif mn == "addName":
            lines.append("    this.native.addName(entity as any, name);")
        elif mn == "getName":
            lines.append("    return this.native.getName(entity as any) ?? null;")
        elif mn == "hasName":
            lines.append("    return this.native.hasName(entity as any);")
        elif mn == "removeName":
            lines.append("    return this.native.removeName(entity as any);")
        elif mn == "getRenderStats":
            lines.append("    return this.native.getRenderStats() as unknown as IRenderStats;")
        elif mn == "getFpsStats":
            lines.append("    return this.native.getFpsStats() as unknown as IFpsStats;")
        elif mn == "getRenderMetrics":
            lines.append("    return this.native.getRenderMetrics() as unknown as IRenderMetrics;")
        elif mn == "getMemorySummary":
            lines.append("    return mapMemorySummary(this.native.getMemorySummary());")
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
            lines.append("      oneWayLatencyMs: config.oneWayLatencyMs,")
            lines.append("      jitterMs: config.jitterMs,")
            lines.append("      packetLossPercent: config.packetLossPercent,")
            lines.append("    } as unknown as INetworkSimulationConfig);")
        elif mn == "loadTexture":
            lines.append("    const cached = this.preloadedTextures.get(path);")
            lines.append("    if (cached !== undefined) {")
            lines.append("      return cached;")
            lines.append("    }")
            lines.append("    const handle = await this.native.loadTexture(path);")
            lines.append("    this.preloadedTextures.set(path, handle);")
            lines.append("    this.texturePathByHandle.set(handle, path);")
            lines.append("    return handle;")
        elif mn == "destroyTexture":
            lines.append("    const path = this.texturePathByHandle.get(handle);")
            lines.append("    if (path !== undefined) {")
            lines.append("      this.texturePathByHandle.delete(handle);")
            lines.append("      this.preloadedTextures.delete(path);")
            lines.append("    }")
            lines.append("    this.native.destroyTexture(handle);")
        elif mn == "loadFont":
            lines.append("    const cached = this.preloadedFonts.get(path);")
            lines.append("    if (cached !== undefined) {")
            lines.append("      return cached;")
            lines.append("    }")
            lines.append("    const handle = await this.native.loadFont(path);")
            lines.append("    this.preloadedFonts.set(path, handle);")
            lines.append("    this.fontPathByHandle.set(handle, path);")
            lines.append("    return handle;")
        elif mn == "destroyFont":
            lines.append("    const path = this.fontPathByHandle.get(handle);")
            lines.append("    if (path !== undefined) {")
            lines.append("      this.fontPathByHandle.delete(handle);")
            lines.append("      this.preloadedFonts.delete(path);")
            lines.append("    }")
            lines.append("    return this.native.destroyFont(handle);")
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
        "  async preload(assets: PreloadAssetInput[], options: IPreloadOptions = {}): Promise<Record<string, number>> {",
        "    if (this.preloadInFlight) {",
        "      throw new Error('game.preload(...) is already in progress.');",
        "    }",
        "    this.preloadInFlight = true;",
        "    const handles: Record<string, number> = {};",
        "    try {",
        "      const normalized = assets.map(normalizePreloadAsset);",
        "      const total = normalized.length;",
        "      let loaded = 0;",
        "      for (const asset of normalized) {",
        "        const handle = asset.kind === 'font'",
        "          ? await this.loadFont(asset.path)",
        "          : await this.loadTexture(asset.path);",
        "        handles[asset.path] = handle;",
        "        loaded += 1;",
        "        const update: IPreloadProgress = {",
        "          loaded,",
        "          total,",
        "          progress: total === 0 ? 1 : loaded / total,",
        "          path: asset.path,",
        "          kind: asset.kind,",
        "          handle,",
        "        };",
        "        options.onProgress?.(update);",
        "      }",
        "      return handles;",
        "    } finally {",
        "      this.preloadInFlight = false;",
        "    }",
        "  }",
        "",
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


def gen_network_shared_wrapper():
    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "import type {",
        "  INetworkConnectResult,",
        "  INetworkPacket,",
        "  INetworkSimulationConfig,",
        "  INetworkStats,",
        "} from '../generated/types/engine.g.js';",
        "",
        "export interface NetworkContextLike {",
        "  networkHost(protocol: number, port: number): number;",
        "  networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult;",
        "  networkReceivePacket(handle: number): INetworkPacket | null;",
        "  networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number;",
        "  networkPoll(handle: number): number;",
        "  networkDisconnect(handle: number): number;",
        "  getNetworkStats(handle: number): INetworkStats;",
        "  networkPeerCount(handle: number): number;",
        "  setNetworkSimulation(handle: number, config: INetworkSimulationConfig): number;",
        "  clearNetworkSimulation(handle: number): number;",
        "  setNetworkOverlayHandle(handle: number): number;",
        "  clearNetworkOverlayHandle(): number;",
        "}",
        "",
        "export class NetworkManager {",
        "  private readonly context: NetworkContextLike;",
        "",
        "  constructor(gameOrContext: NetworkContextLike) {",
        "    this.context = gameOrContext;",
        "  }",
        "",
        "  host(protocol: number, port: number): NetworkEndpoint {",
        "    const handle = this.context.networkHost(protocol, port);",
        "    if (handle < 0) {",
        "      throw new Error(`networkHost failed with handle ${handle}`);",
        "    }",
        "",
        "    return new NetworkEndpoint(this.context, handle);",
        "  }",
        "",
        "  connect(protocol: number, address: string, port: number): NetworkEndpoint {",
        "    const result = this.context.networkConnectWithPeer(protocol, address, port);",
        "    return new NetworkEndpoint(this.context, result.handle, result.peerId);",
        "  }",
        "}",
        "",
        "export class NetworkEndpoint {",
        "  private readonly context: NetworkContextLike;",
        "",
        "  readonly handle: number;",
        "",
        "  readonly defaultPeerId: number | null;",
        "",
        "  constructor(context: NetworkContextLike, handle: number, defaultPeerId: number | null = null) {",
        "    this.context = context;",
        "    this.handle = handle;",
        "    this.defaultPeerId = defaultPeerId;",
        "  }",
        "",
        "  receive(): INetworkPacket | null {",
        "    return this.context.networkReceivePacket(this.handle);",
        "  }",
        "",
        "  send(data: Uint8Array, channel = 0): number {",
        "    if (this.defaultPeerId === null) {",
        "      throw new Error('This endpoint has no default peer ID. Use sendTo(peerId, data, channel) instead.');",
        "    }",
        "",
        "    return this.sendTo(this.defaultPeerId, data, channel);",
        "  }",
        "",
        "  sendTo(peerId: number, data: Uint8Array, channel = 0): number {",
        "    return this.context.networkSend(this.handle, peerId, data, channel);",
        "  }",
        "",
        "  poll(): number {",
        "    return this.context.networkPoll(this.handle);",
        "  }",
        "",
        "  disconnect(): number {",
        "    return this.context.networkDisconnect(this.handle);",
        "  }",
        "",
        "  getStats(): INetworkStats {",
        "    return this.context.getNetworkStats(this.handle);",
        "  }",
        "",
        "  peerCount(): number {",
        "    return this.context.networkPeerCount(this.handle);",
        "  }",
        "",
        "  setSimulation(config: INetworkSimulationConfig): number {",
        "    return this.context.setNetworkSimulation(this.handle, config);",
        "  }",
        "",
        "  clearSimulation(): number {",
        "    return this.context.clearNetworkSimulation(this.handle);",
        "  }",
        "",
        "  setOverlayTarget(): number {",
        "    return this.context.setNetworkOverlayHandle(this.handle);",
        "  }",
        "",
        "  clearOverlayTarget(): number {",
        "    return this.context.clearNetworkOverlayHandle();",
        "  }",
        "}",
        "",
    ]

    write_generated(TS / "src" / "shared" / "network.ts", "\n".join(lines))


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
        f"export {{ GoudGame, GoudContext{ec_export}{ui_export}{pw2d_export}{pw3d_export}, Color, Vec2, Vec3, Key, MouseButton, PhysicsBackend2D, RenderBackendKind, WindowBackendKind }} from './node/index.g.js';",
        f"export type {{ IGoudGame{ec_type_export}{ui_type_export}{pw2d_type_export}{pw3d_type_export}, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IDebuggerConfig, IContextConfig, IMemoryCategoryStats, IMemorySummary, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig }} from './types/engine.g.js';",
        "export type { IGoudContext, INetworkConnectResult, INetworkPacket } from './node/index.g.js';",
        "export type { Rect } from './types/math.g.js';",
        f"export {{ {', '.join(error_names)} }} from './errors.g.js';",
    ]
    if "diagnostic" in schema:
        lines.append("export { DiagnosticMode } from './diagnostic.g.js';")
    lines.append("")
    write_generated(GEN / "index.g.ts", "\n".join(lines))


def gen_public_entrypoints():
    files = {
        TS / "src" / "index.ts": [
            f"// {HEADER_COMMENT}",
            "export * from './generated/index.g.js';",
            "export { parseDebuggerManifest, parseDebuggerSnapshot } from './shared/debugger.js';",
            "export { NetworkManager, NetworkEndpoint } from './shared/network.js';",
            "export type { NetworkContextLike } from './shared/network.js';",
            "export { NetworkProtocol } from './generated/types/input.g.js';",
            "",
        ],
        TS / "src" / "node" / "index.ts": [
            f"// {HEADER_COMMENT}",
            "export * from '../generated/node/index.g.js';",
            "export { parseDebuggerManifest, parseDebuggerSnapshot } from '../shared/debugger.js';",
            "export { NetworkManager, NetworkEndpoint } from '../shared/network.js';",
            "export type { NetworkContextLike } from '../shared/network.js';",
            "export { NetworkProtocol } from '../generated/types/input.g.js';",
            "",
        ],
        TS / "src" / "web" / "index.ts": [
            f"// {HEADER_COMMENT}",
            "export * from '../generated/web/index.g.js';",
            "export { parseDebuggerManifest, parseDebuggerSnapshot } from '../shared/debugger.js';",
            "export { NetworkManager, NetworkEndpoint } from '../shared/network.js';",
            "export type { NetworkContextLike } from '../shared/network.js';",
            "export { NetworkProtocol } from '../generated/types/input.g.js';",
            "",
        ],
    }

    for path, lines in files.items():
        write_generated(path, "\n".join(lines))
