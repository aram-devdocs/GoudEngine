#!/usr/bin/env python3
"""Generates the complete TypeScript Web SDK from the universal schema.

Produces:
  sdks/typescript/src/web/index.g.ts   -- GoudGame wrapping WasmGame, wasm loader, canvas, rAF
  sdks/typescript/src/web/input.g.ts   -- browser KeyboardEvent.code -> GLFW key code map
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from sdk_common import (
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping, load_errors,
    to_camel, to_pascal, to_snake, write_generated, TYPESCRIPT_TYPES,
)
from ts_node_wrapper import gen_network_shared_wrapper

TS = SDKS_DIR / "typescript"
GEN = TS / "src" / "generated"
schema = load_schema()
mapping = load_ffi_mapping(schema)


def emit_jsdoc(lines: list, doc: str | None, indent: str = "  ") -> None:
    """Emit a JSDoc comment if doc string exists."""
    if doc:
        lines.append(f"{indent}/** {doc} */")


def ts_type(t: str) -> str:
    base = t.rstrip("?")
    mapped = TYPESCRIPT_TYPES.get(base, base)
    if t.endswith("?"):
        return f"{mapped} | null"
    return mapped


UI_STYLE_WASM_SIGNATURE = (
    "set_style("
    "node_id: bigint, "
    "background_r?: number, background_g?: number, background_b?: number, background_a?: number, "
    "foreground_r?: number, foreground_g?: number, foreground_b?: number, foreground_a?: number, "
    "border_r?: number, border_g?: number, border_b?: number, border_a?: number, "
    "border_width?: number, "
    "font_family?: string, "
    "font_size?: number, "
    "texture_path?: string, "
    "widget_spacing?: number"
    "): number;"
)


def emit_ui_manager_set_style(lines: list[str]) -> None:
    """Emit the thin public UiManager.setStyle wrapper for the scalar wasm ABI."""
    lines.append("  setStyle(nodeId: UiNodeId, style: IUiStyle): number {")
    lines.append("    return this.handle.set_style(")
    lines.append("      toWasmUiNodeId(nodeId),")
    lines.append("      style.backgroundColor?.r,")
    lines.append("      style.backgroundColor?.g,")
    lines.append("      style.backgroundColor?.b,")
    lines.append("      style.backgroundColor?.a,")
    lines.append("      style.foregroundColor?.r,")
    lines.append("      style.foregroundColor?.g,")
    lines.append("      style.foregroundColor?.b,")
    lines.append("      style.foregroundColor?.a,")
    lines.append("      style.borderColor?.r,")
    lines.append("      style.borderColor?.g,")
    lines.append("      style.borderColor?.b,")
    lines.append("      style.borderColor?.a,")
    lines.append("      style.borderWidth,")
    lines.append("      style.fontFamily,")
    lines.append("      style.fontSize,")
    lines.append("      style.texturePath,")
    lines.append("      style.widgetSpacing,")
    lines.append("    );")
    lines.append("  }")


# ── web/input.g.ts ──────────────────────────────────────────────────

def gen_web_input():
    key_enum = schema["enums"]["Key"]
    code_map = key_enum.get("web_code_map", {})

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "interface WasmInputSink {",
        "  press_key(keyCode: number): void;",
        "  release_key(keyCode: number): void;",
        "  press_mouse_button(button: number): void;",
        "  release_mouse_button(button: number): void;",
        "  set_mouse_position(x: number, y: number): void;",
        "  add_scroll_delta(dx: number, dy: number): void;",
        "}",
        "",
        "const CODE_MAP: Record<string, number> = {",
    ]

    for key_name, glfw_val in key_enum["values"].items():
        browser_code = code_map.get(key_name)
        if browser_code:
            lines.append(f"  {browser_code}: {glfw_val},")

    lines.append("};")
    lines.append("")
    lines.append("export function codeToKeyCode(code: string): number | undefined {")
    lines.append("  return CODE_MAP[code];")
    lines.append("}")
    lines.append("")

    lines.append("export function attachInputHandlers(target: HTMLElement, sink: WasmInputSink): () => void {")
    lines.append("  const onKeyDown = (e: KeyboardEvent) => { const kc = codeToKeyCode(e.code); if (kc !== undefined) { e.preventDefault(); sink.press_key(kc); } };")
    lines.append("  const onKeyUp = (e: KeyboardEvent) => { const kc = codeToKeyCode(e.code); if (kc !== undefined) { e.preventDefault(); sink.release_key(kc); } };")
    lines.append("  const onMouseDown = (e: MouseEvent) => sink.press_mouse_button(e.button);")
    lines.append("  const onMouseUp = (e: MouseEvent) => sink.release_mouse_button(e.button);")
    lines.append("  const onMouseMove = (e: MouseEvent) => { const r = target.getBoundingClientRect(); sink.set_mouse_position(e.clientX - r.left, e.clientY - r.top); };")
    lines.append("  const onWheel = (e: WheelEvent) => { e.preventDefault(); sink.add_scroll_delta(e.deltaX, e.deltaY); };")
    lines.append("  const onCtx = (e: Event) => e.preventDefault();")
    lines.append("  const onTouchStart = (e: TouchEvent) => { e.preventDefault(); const t = e.touches[0]; if (t) { const r = target.getBoundingClientRect(); sink.set_mouse_position(t.clientX - r.left, t.clientY - r.top); sink.press_mouse_button(0); } };")
    lines.append("  const onTouchEnd = (e: TouchEvent) => { e.preventDefault(); sink.release_mouse_button(0); };")
    lines.append("")
    lines.append("  window.addEventListener('keydown', onKeyDown);")
    lines.append("  window.addEventListener('keyup', onKeyUp);")
    lines.append("  target.addEventListener('mousedown', onMouseDown);")
    lines.append("  target.addEventListener('mouseup', onMouseUp);")
    lines.append("  target.addEventListener('mousemove', onMouseMove);")
    lines.append("  target.addEventListener('wheel', onWheel, { passive: false });")
    lines.append("  target.addEventListener('contextmenu', onCtx);")
    lines.append("  target.addEventListener('touchstart', onTouchStart, { passive: false });")
    lines.append("  target.addEventListener('touchend', onTouchEnd, { passive: false });")
    lines.append("")
    lines.append("  return () => {")
    lines.append("    window.removeEventListener('keydown', onKeyDown);")
    lines.append("    window.removeEventListener('keyup', onKeyUp);")
    lines.append("    target.removeEventListener('mousedown', onMouseDown);")
    lines.append("    target.removeEventListener('mouseup', onMouseUp);")
    lines.append("    target.removeEventListener('mousemove', onMouseMove);")
    lines.append("    target.removeEventListener('wheel', onWheel);")
    lines.append("    target.removeEventListener('contextmenu', onCtx);")
    lines.append("    target.removeEventListener('touchstart', onTouchStart);")
    lines.append("    target.removeEventListener('touchend', onTouchEnd);")
    lines.append("  };")
    lines.append("}")
    lines.append("")

    write_generated(GEN / "web" / "input.g.ts", "\n".join(lines))


# ── web/index.g.ts ──────────────────────────────────────────────────

def gen_web_wrapper():
    tool = schema["tools"]["GoudGame"]

    lines = [
        f"// {HEADER_COMMENT}",
        "",
        "import type { IGoudGame, IUiManager, IUiStyle, IUiEvent, UiNodeId, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderMetrics, IDebuggerConfig, IContextConfig, IMemoryCategoryStats, IMemorySummary, IDebuggerCapture, IDebuggerReplayArtifact, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IPreloadAssetRequest, IPreloadOptions, IPreloadProgress, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig, INetworkConnectResult, INetworkPacket, IP2pMeshConfig, IRollbackConfig, PreloadAssetInput, PreloadAssetKind, ISpriteCmd, ITextCmd } from '../types/engine.g.js';",
        "import { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "import { PhysicsBackend2D, RenderBackendKind, WindowBackendKind } from '../types/input.g.js';",
        "import { attachInputHandlers } from './input.g.js';",
        "",
        "export { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Key, MouseButton, PhysicsBackend2D, RenderBackendKind, WindowBackendKind } from '../types/input.g.js';",
        "export { Rect } from '../types/math.g.js';",
        "export type { IGoudGame, IUiManager, IUiStyle, IUiEvent, UiNodeId, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact, IFpsStats, IRenderMetrics, IDebuggerConfig, IContextConfig, IMemoryCategoryStats, IMemorySummary, IDebuggerCapture, IDebuggerReplayArtifact, IPhysicsRaycastHit2D, IPhysicsCollisionEvent2D, IAnimationEventData, IPreloadAssetRequest, IPreloadOptions, IPreloadProgress, IRenderCapabilities, IPhysicsCapabilities, IAudioCapabilities, IInputCapabilities, INetworkCapabilities, INetworkStats, INetworkSimulationConfig, INetworkConnectResult, INetworkPacket, IP2pMeshConfig, IRollbackConfig, PreloadAssetInput, PreloadAssetKind } from '../types/engine.g.js';",
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
        "const WEB_RENDER_CAPABILITIES: IRenderCapabilities = { maxTextureUnits: 0, maxTextureSize: 0, supportsInstancing: false, supportsCompute: false, supportsMsaa: false };",
        "const WEB_PHYSICS_CAPABILITIES: IPhysicsCapabilities = { supportsContinuousCollision: false, supportsJoints: false, maxBodies: 0 };",
        "const WEB_AUDIO_CAPABILITIES: IAudioCapabilities = { supportsSpatial: true, maxChannels: 32 };",
        "const WEB_INPUT_CAPABILITIES: IInputCapabilities = { supportsGamepad: false, supportsTouch: true, maxGamepads: 0 };",
        "",
    ]

    # Build WasmGameHandle interface from schema wasm_interface entries
    wasm_iface = tool.get("wasm_interface", [])
    method_lookup = {m["name"]: m for m in tool["methods"]}
    lines.append("interface WasmGameHandle {")
    for entry in wasm_iface:
        if "sig" in entry:
            # Direct signature (web-only or property)
            lines.append(f"  {entry['sig']};")
        elif "method" in entry:
            # Schema method reference -- use its wasm_signature
            method = method_lookup.get(entry["method"], {})
            sig = method.get("wasm_signature", "")
            if sig:
                lines.append(f"  {sig};")
    lines.extend([
        "  network_host(protocol: number, port: number): number;",
        "  network_connect(protocol: number, address: string, port: number): number;",
        "  network_connect_with_peer(protocol: number, address: string, port: number): WasmNetworkConnectResult;",
        "  network_disconnect(handle: number): number;",
        "  network_send(handle: number, peer_id: bigint, data: Uint8Array, channel: number): number;",
        "  network_receive(handle: number): Uint8Array;",
        "  network_receive_packet(handle: number): WasmNetworkPacket | undefined;",
        "  network_poll(handle: number): number;",
        "  get_network_stats(handle: number): WasmNetworkStats;",
        "  get_network_capabilities(): WasmNetworkCapabilities;",
        "  network_peer_count(handle: number): number;",
        "  set_network_simulation(handle: number, one_way_latency_ms: number, jitter_ms: number, packet_loss_percent: number): number;",
        "  clear_network_simulation(handle: number): number;",
        "  set_network_overlay_handle(handle: number): number;",
        "  clear_network_overlay_handle(): number;",
    ])
    lines.extend([
        "  initDebugger(label: string): void;",
        "  dispatchDebuggerRequest(json: string): string;",
        "  getDebuggerSnapshotJson(): string;",
    ])
    lines.append("}")
    lines.append("")
    lines.extend([
        "interface WasmNetworkConnectResult {",
        "  handle: number;",
        "  peer_id: bigint | number;",
        "}",
        "",
        "interface WasmNetworkPacket {",
        "  peer_id: bigint | number;",
        "  data: Uint8Array;",
        "}",
        "",
        "interface WasmNetworkStats {",
        "  bytes_sent: bigint | number;",
        "  bytes_received: bigint | number;",
        "  packets_sent: bigint | number;",
        "  packets_received: bigint | number;",
        "  packets_lost: bigint | number;",
        "  rtt_ms: number;",
        "  send_bandwidth_bytes_per_sec: number;",
        "  receive_bandwidth_bytes_per_sec: number;",
        "  packet_loss_percent: number;",
        "  jitter_ms: number;",
        "}",
        "",
        "interface WasmNetworkCapabilities {",
        "  supports_hosting: boolean;",
        "  max_connections: number;",
        "  max_channels: number;",
        "  max_message_size: number;",
        "}",
        "",
    ])

    lines.append("interface WasmExports {")
    lines.append("  WasmGame: {")
    lines.append("    new(width: number, height: number, title: string): WasmGameHandle;")
    lines.append("    createWithCanvas(canvas: HTMLCanvasElement, w: number, h: number, title: string): Promise<WasmGameHandle>;")
    lines.append("  };")
    lines.append("  WasmUiManager: {")
    lines.append("    new(): WasmUiManagerHandle;")
    lines.append("  };")
    lines.append("}")
    lines.append("")

    lines.append("interface WasmUiManagerHandle {")
    lines.append("  free(): void;")
    lines.append("  update(): void;")
    lines.append("  render(): void;")
    lines.append("  node_count(): number;")
    lines.append("  create_node(component_type: number): bigint;")
    lines.append("  remove_node(node_id: bigint): number;")
    lines.append("  set_parent(child_id: bigint, parent_id: bigint): number;")
    lines.append("  get_parent(node_id: bigint): bigint;")
    lines.append("  get_child_count(node_id: bigint): number;")
    lines.append("  get_child_at(node_id: bigint, index: number): bigint;")
    lines.append("  set_widget(node_id: bigint, widget_kind: number): number;")
    lines.append(f"  {UI_STYLE_WASM_SIGNATURE}")
    lines.append("  set_label_text(node_id: bigint, text: string): number;")
    lines.append("  set_button_enabled(node_id: bigint, enabled: boolean): number;")
    lines.append("  set_image_texture_path(node_id: bigint, path: string): number;")
    lines.append("  set_slider(node_id: bigint, min: number, max: number, value: number, enabled: boolean): number;")
    lines.append("  event_count(): number;")
    lines.append("  event_read(index: number): IUiEvent | undefined;")
    lines.append("}")
    lines.append("")

    lines.append("let _wasmModule: WasmExports | null = null;")
    lines.append("")
    lines.append("export async function initWasm(wasmUrl?: string): Promise<void> {")
    lines.append("  if (_wasmModule) return;")
    lines.append("  const url = wasmUrl ?? new URL('goud_engine_bg.wasm', import.meta.url).href;")
    lines.append("  const mod = await import(/* webpackIgnore: true */ url.replace(/_bg\\.wasm$/, '.js'));")
    lines.append("  await mod.default(url);")
    lines.append("  _wasmModule = mod as unknown as WasmExports;")
    lines.append("}")
    lines.append("")
    lines.append("function getWasm(): WasmExports {")
    lines.append("  if (!_wasmModule) throw new Error('Wasm not loaded. Call initWasm() first.');")
    lines.append("  return _wasmModule;")
    lines.append("}")
    lines.append("")
    lines.append("function toWasmUiNodeId(nodeId: UiNodeId): bigint {")
    lines.append("  return typeof nodeId === 'bigint' ? nodeId : BigInt(nodeId);")
    lines.append("}")
    lines.append("")

    lines.append("export class UiManager implements IUiManager {")
    lines.append("  private handle: WasmUiManagerHandle;")
    lines.append("")
    lines.append("  private constructor(handle: WasmUiManagerHandle) {")
    lines.append("    this.handle = handle;")
    lines.append("  }")
    lines.append("")
    lines.append("  static async create(wasmUrl?: string): Promise<UiManager> {")
    lines.append("    await initWasm(wasmUrl);")
    lines.append("    const wasm = getWasm();")
    lines.append("    return new UiManager(new wasm.WasmUiManager());")
    lines.append("  }")
    lines.append("")
    lines.append("  destroy(): void { this.handle.free(); }")
    lines.append("  update(): void { this.handle.update(); }")
    lines.append("  render(): void { this.handle.render(); }")
    lines.append("  nodeCount(): number { return this.handle.node_count(); }")
    lines.append("  createNode(componentType: number): UiNodeId { return this.handle.create_node(componentType); }")
    lines.append("  removeNode(nodeId: UiNodeId): number { return this.handle.remove_node(toWasmUiNodeId(nodeId)); }")
    lines.append("  setParent(childId: UiNodeId, parentId: UiNodeId): number { return this.handle.set_parent(toWasmUiNodeId(childId), toWasmUiNodeId(parentId)); }")
    lines.append("  getParent(nodeId: UiNodeId): UiNodeId { return this.handle.get_parent(toWasmUiNodeId(nodeId)); }")
    lines.append("  getChildCount(nodeId: UiNodeId): number { return this.handle.get_child_count(toWasmUiNodeId(nodeId)); }")
    lines.append("  getChildAt(nodeId: UiNodeId, index: number): UiNodeId { return this.handle.get_child_at(toWasmUiNodeId(nodeId), index); }")
    lines.append("  setWidget(nodeId: UiNodeId, widgetKind: number): number { return this.handle.set_widget(toWasmUiNodeId(nodeId), widgetKind); }")
    emit_ui_manager_set_style(lines)
    lines.append("  setLabelText(nodeId: UiNodeId, text: string): number { return this.handle.set_label_text(toWasmUiNodeId(nodeId), text); }")
    lines.append("  setButtonEnabled(nodeId: UiNodeId, enabled: boolean): number { return this.handle.set_button_enabled(toWasmUiNodeId(nodeId), enabled); }")
    lines.append("  setImageTexturePath(nodeId: UiNodeId, path: string): number { return this.handle.set_image_texture_path(toWasmUiNodeId(nodeId), path); }")
    lines.append("  setSlider(nodeId: UiNodeId, min: number, max: number, value: number, enabled: boolean): number { return this.handle.set_slider(toWasmUiNodeId(nodeId), min, max, value, enabled); }")
    lines.append("  eventCount(): number { return this.handle.event_count(); }")
    lines.append("  eventRead(index: number): IUiEvent | null { return this.handle.event_read(index) ?? null; }")
    lines.append("")
    lines.append("  createPanel(): UiNodeId { return this.createNode(0); }")
    lines.append("  createLabel(text: string): UiNodeId { const nodeId = this.createNode(2); this.setLabelText(nodeId, text); return nodeId; }")
    lines.append("  createButton(enabled = true): UiNodeId { const nodeId = this.createNode(1); this.setButtonEnabled(nodeId, enabled); return nodeId; }")
    lines.append("  createImage(path: string): UiNodeId { const nodeId = this.createNode(3); this.setImageTexturePath(nodeId, path); return nodeId; }")
    lines.append("  createSlider(min: number, max: number, value: number, enabled = true): UiNodeId { const nodeId = this.createNode(4); this.setSlider(nodeId, min, max, value, enabled); return nodeId; }")
    lines.append("")
    lines.append("  // TODO: wasm UI positioning -- these stub methods satisfy the IUiManager interface")
    lines.append("  setNodePosition(nodeId: UiNodeId, x: number, y: number): number { return 0; }")
    lines.append("  setNodeVisible(nodeId: UiNodeId, visible: boolean): number { return 0; }")
    lines.append("}")
    lines.append("")

    lines.append("class WebEntity implements IEntity {")
    lines.append("  constructor(private _bits: bigint) {}")
    lines.append("  get index(): number { return Number(this._bits & 0xFFFFFFFFn); }")
    lines.append("  get generation(): number { return Number(this._bits >> 32n); }")
    lines.append("  get isPlaceholder(): boolean { return this._bits === 0xFFFFFFFFFFFFFFFFn; }")
    lines.append("  toBits(): bigint { return this._bits; }")
    lines.append("}")
    lines.append("")

    lines.append("export interface WebGameConfig {")
    lines.append("  width?: number; height?: number; title?: string;")
    lines.append("  canvas?: HTMLCanvasElement; wasmUrl?: string;")
    lines.append("}")
    lines.append("")

    emit_jsdoc(lines, tool.get("doc"), indent="")
    lines.append("export class GoudGame implements IGoudGame {")
    lines.append("  private handle: WasmGameHandle;")
    lines.append("  private canvas: HTMLCanvasElement;")
    lines.append("  private detachInput: (() => void) | null = null;")
    lines.append("  private rafId = 0;")
    lines.append("  private running = false;")
    lines.append("  private lastTs = 0;")
    lines.append("  private _shouldClose = false;")
    lines.append("  private _updateFn: ((dt: number) => void) | null = null;")
    lines.append("  private _audioGlobalVolume = 1;")
    lines.append("  private _audioChannelVolumes = new Map<number, number>();")
    lines.append("  private _activeAudioPlayers = new Set<number>();")
    lines.append("  private readonly preloadedTextures = new Map<string, number>();")
    lines.append("  private readonly preloadedFonts = new Map<string, number>();")
    lines.append("  private readonly texturePathByHandle = new Map<number, string>();")
    lines.append("  private readonly fontPathByHandle = new Map<number, string>();")
    lines.append("  private preloadInFlight = false;")
    lines.append("")
    lines.append("  private constructor(handle: WasmGameHandle, canvas: HTMLCanvasElement) {")
    lines.append("    this.handle = handle; this.canvas = canvas;")
    lines.append("  }")
    lines.append("")
    lines.append("  static async create(config: WebGameConfig = {}): Promise<GoudGame> {")
    lines.append("    await initWasm(config.wasmUrl);")
    lines.append("    const wasm = getWasm();")
    lines.append("    const w = config.width ?? 800, h = config.height ?? 600, t = config.title ?? 'GoudEngine';")
    lines.append("    let canvas = config.canvas ?? null;")
    lines.append("    if (!canvas) { canvas = document.createElement('canvas'); canvas.width = w; canvas.height = h; canvas.style.display = 'block'; document.body.appendChild(canvas); }")
    lines.append("    else { canvas.width = w; canvas.height = h; }")
    lines.append("    document.title = t;")
    lines.append("    let handle: WasmGameHandle;")
    lines.append("    try { handle = await wasm.WasmGame.createWithCanvas(canvas, w, h, t); }")
    lines.append("    catch { handle = new wasm.WasmGame(w, h, t); }")
    lines.append("    const game = new GoudGame(handle, canvas);")
    lines.append("    let resizeReady = false;")
    lines.append("    new ResizeObserver(entries => { if (!resizeReady) return; requestAnimationFrame(() => { for (const e of entries) { const r = e.contentRect; handle.set_canvas_size(Math.round(r.width), Math.round(r.height)); } }); }).observe(canvas);")
    lines.append("    resizeReady = true;")
    lines.append("    return game;")
    lines.append("  }")
    lines.append("")

    # Property name mapping: camelCase schema name -> snake_case wasm handle property
    _WASM_PROP = {
        "deltaTime": "delta_time",
        "fps": "fps",
        "windowWidth": "window_width",
        "windowHeight": "window_height",
        "title": "title",
        "totalTime": "total_time",
        "frameCount": "frame_count",
    }
    # Properties not available in WASM — return a sensible default.
    _WASM_STUB_PROPS = {
        "interpolationAlpha": "0",
    }
    for prop in tool["properties"]:
        pn = to_camel(prop["name"])
        pt = ts_type(prop["type"])
        emit_jsdoc(lines, prop.get("doc"))
        if pn in _WASM_STUB_PROPS:
            lines.append(f"  get {pn}(): {pt} {{ return {_WASM_STUB_PROPS[pn]}; }}")
        elif pn == "frameCount":
            wasm_name = _WASM_PROP.get(pn, pn)
            lines.append(f"  get {pn}(): {pt} {{ return Number(this.handle.{wasm_name}); }}")
        else:
            wasm_name = _WASM_PROP.get(pn, pn)
            lines.append(f"  get {pn}(): {pt} {{ return this.handle.{wasm_name}; }}")
    lines.append("")

    # Build method doc lookup from schema (keyed by snake_case for wasm handle matching)
    _method_docs = {}
    for m in tool["methods"]:
        if m.get("doc"):
            _method_docs[m["name"]] = m["doc"]
            _method_docs[to_snake(m["name"])] = m["doc"]

    emit_jsdoc(lines, _method_docs.get("set_clear_color"))
    lines.append("  setClearColor(r: number, g: number, b: number, a: number): void { this.handle.set_clear_color(r, g, b, a); }")
    lines.append("")
    emit_jsdoc(lines, _method_docs.get("should_close"))
    lines.append("  shouldClose(): boolean { return this._shouldClose; }")
    emit_jsdoc(lines, _method_docs.get("close"))
    lines.append("  close(): void { this._shouldClose = true; this.stop(); }")
    emit_jsdoc(lines, _method_docs.get("set_window_size"))
    lines.append("  setWindowSize(width: number, height: number): boolean {")
    lines.append("    this.canvas.width = width;")
    lines.append("    this.canvas.height = height;")
    lines.append("    this.handle.set_canvas_size(width, height);")
    lines.append("    return true;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("destroy"))
    lines.append("  destroy(): void {")
    lines.append("    this.stop();")
    lines.append("    this._activeAudioPlayers.clear();")
    lines.append("    this.handle.free();")
    lines.append("  }")
    lines.append("")
    emit_jsdoc(lines, _method_docs.get("begin_frame"))
    lines.append("  beginFrame(r = 0, g = 0, b = 0, a = 1): void {")
    lines.append("    this.handle.set_clear_color(r, g, b, a);")
    lines.append("    const now = performance.now();")
    lines.append("    const dtMs = now - (this.lastTs || now);")
    lines.append("    this.lastTs = now;")
    lines.append("    this.handle.begin_frame(dtMs / 1000);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("end_frame"))
    lines.append("  endFrame(): void { this.handle.end_frame(); }")
    emit_jsdoc(lines, _method_docs.get("update_frame"))
    lines.append("  updateFrame(dt: number): void { this.handle.begin_frame(dt); }")
    lines.append("")
    emit_jsdoc(lines, _method_docs.get("run"))
    lines.append("  run(update: (dt: number) => void): void {")
    lines.append("    if (this.running) return;")
    lines.append("    if (this.preloadInFlight) {")
    lines.append("      throw new Error('game.preload(...) must finish before game.run() starts.');")
    lines.append("    }")
    lines.append("    if (update.constructor.name === 'AsyncFunction') {")
    lines.append("      console.warn('GoudEngine: game.run() callback should be synchronous. Async callbacks may cause borrow conflicts in WASM.');")
    lines.append("    }")
    lines.append("    this._updateFn = update;")
    lines.append("    this.running = true; this.lastTs = performance.now();")
    lines.append("    this.detachInput = attachInputHandlers(this.canvas, this.handle);")
    lines.append("    this._startLoop(update);")
    lines.append("  }")
    lines.append("")
    lines.append("  runWithFixedUpdate(fixedUpdate: (dt: number) => void, update: (dt: number) => void): void {")
    lines.append("    if (this.running) return;")
    lines.append("    if (this.preloadInFlight) {")
    lines.append("      throw new Error('game.preload(...) must finish before game.runWithFixedUpdate() starts.');")
    lines.append("    }")
    lines.append("    this._updateFn = (dt: number) => {")
    lines.append("      // Fixed timestep not directly available in WASM yet — fall through to update")
    lines.append("      update(dt);")
    lines.append("    };")
    lines.append("    this.running = true; this.lastTs = performance.now();")
    lines.append("    this.detachInput = attachInputHandlers(this.canvas, this.handle);")
    lines.append("    this._startLoop(this._updateFn);")
    lines.append("  }")
    lines.append("")
    lines.append("  setFixedTimestep(_stepSize: number): void { /* WASM stub */ }")
    lines.append("  setMaxFixedSteps(_maxSteps: number): void { /* WASM stub */ }")
    lines.append("")

    lines.append("  stop(): void;")
    lines.append("  stop(_entity: IEntity): number;")
    lines.append("  stop(entity?: IEntity): void | number {")
    lines.append("    if (entity !== undefined) return 0;")
    lines.append("    this.running = false;")
    lines.append("    if (this.rafId) { cancelAnimationFrame(this.rafId); this.rafId = 0; }")
    lines.append("    if (this.detachInput) { this.detachInput(); this.detachInput = null; }")
    lines.append("    this._updateFn = null;")
    lines.append("  }")
    lines.append("")
    lines.append("  pause(): void {")
    lines.append("    this.running = false;")
    lines.append("    if (this.rafId) { cancelAnimationFrame(this.rafId); this.rafId = 0; }")
    lines.append("  }")
    lines.append("")
    lines.append("  resume(): void {")
    lines.append("    if (!this._updateFn || this.running) return;")
    lines.append("    this.running = true; this.lastTs = performance.now();")
    lines.append("    if (!this.detachInput) { this.detachInput = attachInputHandlers(this.canvas, this.handle); }")
    lines.append("    this._startLoop(this._updateFn);")
    lines.append("  }")
    lines.append("")
    lines.append("  private _startLoop(update: (dt: number) => void): void {")
    lines.append("    const loop = (ts: number) => {")
    lines.append("      if (!this.running) return;")
    lines.append("      const dt = Math.min((ts - this.lastTs) / 1000, 0.1); this.lastTs = ts;")
    lines.append("      this.handle.begin_frame(dt);")
    lines.append("      update(this.handle.delta_time);")
    lines.append("      this.handle.end_frame();")
    lines.append("      this.rafId = requestAnimationFrame(loop);")
    lines.append("    };")
    lines.append("    this.rafId = requestAnimationFrame(loop);")
    lines.append("  }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("load_texture"))
    lines.append("  async loadTexture(path: string): Promise<number> {")
    lines.append("    const cached = this.preloadedTextures.get(path);")
    lines.append("    if (cached !== undefined) {")
    lines.append("      return cached;")
    lines.append("    }")
    lines.append("    const resp = await fetch(path);")
    lines.append("    if (!resp.ok) throw new Error(`Failed to load texture: ${path} (HTTP ${resp.status})`);")
    lines.append("    const bytes = new Uint8Array(await resp.arrayBuffer());")
    lines.append("    const handle = this.handle.register_texture_from_bytes(bytes);")
    lines.append("    this.preloadedTextures.set(path, handle);")
    lines.append("    this.texturePathByHandle.set(handle, path);")
    lines.append("    return handle;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("destroy_texture"))
    lines.append("  destroyTexture(handle: number): void {")
    lines.append("    const path = this.texturePathByHandle.get(handle);")
    lines.append("    if (path !== undefined) {")
    lines.append("      this.texturePathByHandle.delete(handle);")
    lines.append("      this.preloadedTextures.delete(path);")
    lines.append("    }")
    lines.append("    this.handle.destroy_texture(handle);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("load_font"))
    lines.append("  async loadFont(path: string): Promise<number> {")
    lines.append("    const cached = this.preloadedFonts.get(path);")
    lines.append("    if (cached !== undefined) {")
    lines.append("      return cached;")
    lines.append("    }")
    lines.append("    const resp = await fetch(path);")
    lines.append("    if (!resp.ok) throw new Error(`Failed to load font: ${path} (HTTP ${resp.status})`);")
    lines.append("    const bytes = new Uint8Array(await resp.arrayBuffer());")
    lines.append("    const handle = this.handle.register_font_from_bytes(bytes);")
    lines.append("    this.preloadedFonts.set(path, handle);")
    lines.append("    this.fontPathByHandle.set(handle, path);")
    lines.append("    return handle;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("destroy_font"))
    lines.append("  destroyFont(handle: number): boolean {")
    lines.append("    const path = this.fontPathByHandle.get(handle);")
    lines.append("    if (path !== undefined) {")
    lines.append("      this.fontPathByHandle.delete(handle);")
    lines.append("      this.preloadedFonts.delete(path);")
    lines.append("    }")
    lines.append("    return this.handle.destroy_font(handle);")
    lines.append("  }")
    lines.append("  async preload(assets: PreloadAssetInput[], options: IPreloadOptions = {}): Promise<Record<string, number>> {")
    lines.append("    if (this.preloadInFlight) {")
    lines.append("      throw new Error('game.preload(...) is already in progress.');")
    lines.append("    }")
    lines.append("    this.preloadInFlight = true;")
    lines.append("    const handles: Record<string, number> = {};")
    lines.append("    try {")
    lines.append("      const normalized = assets.map(normalizePreloadAsset);")
    lines.append("      const total = normalized.length;")
    lines.append("      let loaded = 0;")
    lines.append("      for (const asset of normalized) {")
    lines.append("        const handle = asset.kind === 'font'")
    lines.append("          ? await this.loadFont(asset.path)")
    lines.append("          : await this.loadTexture(asset.path);")
    lines.append("        handles[asset.path] = handle;")
    lines.append("        loaded += 1;")
    lines.append("        const update: IPreloadProgress = {")
    lines.append("          loaded,")
    lines.append("          total,")
    lines.append("          progress: total === 0 ? 1 : loaded / total,")
    lines.append("          path: asset.path,")
    lines.append("          kind: asset.kind,")
    lines.append("          handle,")
    lines.append("        };")
    lines.append("        options.onProgress?.(update);")
    lines.append("      }")
    lines.append("      return handles;")
    lines.append("    } finally {")
    lines.append("      this.preloadInFlight = false;")
    lines.append("    }")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("draw_text"))
    lines.append("  drawText(fontHandle: number, text: string, x: number, y: number, fontSize = 16, alignment = 0, maxWidth = 0, lineSpacing = 1, direction = 0, color?: IColor): boolean {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    return this.handle.draw_text(fontHandle, text, x, y, fontSize, alignment, maxWidth, lineSpacing, direction, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("draw_sprite"))
    lines.append("  drawSprite(texture: number, x: number, y: number, width: number, height: number, rotation = 0, color?: IColor): void {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    this.handle.draw_sprite(texture, x, y, width, height, rotation, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("draw_sprite_rect"))
    lines.append("  drawSpriteRect(texture: number, x: number, y: number, width: number, height: number, rotation = 0, srcX = 0, srcY = 0, srcW = 1, srcH = 1, color?: IColor, srcMode = 1): boolean {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    // WASM path only supports normalized UVs; srcMode is ignored")
    lines.append("    return this.handle.draw_sprite_rect(texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    lines.append("  /** Draws a batch of sprites (not yet supported on WASM target, returns 0). */")
    lines.append("  drawSpriteBatch(_cmds: ISpriteCmd[]): number {")
    lines.append("    return 0;")
    lines.append("  }")
    lines.append("  /** Draws a batch of text labels (not yet supported on WASM target, returns 0). */")
    lines.append("  drawTextBatch(_cmds: ITextCmd[]): number {")
    lines.append("    return 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("draw_quad"))
    lines.append("  drawQuad(x: number, y: number, width: number, height: number, color?: IColor): void {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    this.handle.draw_quad(x, y, width, height, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("audio_play"))
    lines.append("  audioPlay(data: Uint8Array): number {")
    lines.append("    const playerId = this.handle.audio_play(data);")
    lines.append("    if (playerId >= 0) this._activeAudioPlayers.add(playerId);")
    lines.append("    return playerId;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_play_on_channel"))
    lines.append("  audioPlayOnChannel(data: Uint8Array, channel: number): number {")
    lines.append("    const fn = (this.handle as any).audio_play_on_channel as ((data: Uint8Array, channel: number) => number) | undefined;")
    lines.append("    const playerId = fn ? fn.call(this.handle, data, channel) : this.handle.audio_play(data);")
    lines.append("    if (playerId >= 0) this._activeAudioPlayers.add(playerId);")
    lines.append("    return playerId;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_play_with_settings"))
    lines.append("  audioPlayWithSettings(data: Uint8Array, volume: number, speed: number, looping: boolean, channel: number): number {")
    lines.append("    const fn = (this.handle as any).audio_play_with_settings as ((data: Uint8Array, volume: number, speed: number, looping: boolean, channel: number) => number) | undefined;")
    lines.append("    const playerId = fn ? fn.call(this.handle, data, volume, speed, looping, channel) : this.handle.audio_play(data);")
    lines.append("    if (playerId < 0) return playerId;")
    lines.append("    this._activeAudioPlayers.add(playerId);")
    lines.append("    if (!fn) {")
    lines.append("      this.handle.audio_set_player_volume(playerId, volume);")
    lines.append("      this.handle.audio_set_player_speed(playerId, speed);")
    lines.append("    }")
    lines.append("    return playerId;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_stop"))
    lines.append("  audioStop(playerId: number): number {")
    lines.append("    const rc = this.handle.audio_stop(playerId);")
    lines.append("    if (rc === 0) this._activeAudioPlayers.delete(playerId);")
    lines.append("    return rc;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_pause"))
    lines.append("  audioPause(playerId: number): number { return this.handle.audio_pause(playerId); }")
    emit_jsdoc(lines, _method_docs.get("audio_resume"))
    lines.append("  audioResume(playerId: number): number { return this.handle.audio_resume(playerId); }")
    emit_jsdoc(lines, _method_docs.get("audio_stop_all"))
    lines.append("  audioStopAll(): number {")
    lines.append("    const rc = this.handle.audio_stop_all();")
    lines.append("    if (rc === 0) this._activeAudioPlayers.clear();")
    lines.append("    return rc;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_set_global_volume"))
    lines.append("  audioSetGlobalVolume(volume: number): number {")
    lines.append("    const fn = (this.handle as any).audio_set_global_volume as ((volume: number) => number) | undefined;")
    lines.append("    if (fn) {")
    lines.append("      const rc = fn.call(this.handle, volume);")
    lines.append("      if (rc === 0) this._audioGlobalVolume = volume;")
    lines.append("      return rc;")
    lines.append("    }")
    lines.append("    this._audioGlobalVolume = volume;")
    lines.append("    return 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_get_global_volume"))
    lines.append("  audioGetGlobalVolume(): number {")
    lines.append("    const fn = (this.handle as any).audio_get_global_volume as (() => number) | undefined;")
    lines.append("    return fn ? fn.call(this.handle) : this._audioGlobalVolume;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_set_channel_volume"))
    lines.append("  audioSetChannelVolume(channel: number, volume: number): number {")
    lines.append("    const fn = (this.handle as any).audio_set_channel_volume as ((channel: number, volume: number) => number) | undefined;")
    lines.append("    if (fn) return fn.call(this.handle, channel, volume);")
    lines.append("    this._audioChannelVolumes.set(channel, volume);")
    lines.append("    return 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_get_channel_volume"))
    lines.append("  audioGetChannelVolume(channel: number): number {")
    lines.append("    const fn = (this.handle as any).audio_get_channel_volume as ((channel: number) => number) | undefined;")
    lines.append("    if (fn) return fn.call(this.handle, channel);")
    lines.append("    return this._audioChannelVolumes.get(channel) ?? this._audioGlobalVolume;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_is_playing"))
    lines.append("  audioIsPlaying(playerId: number): number {")
    lines.append("    const fn = (this.handle as any).audio_is_playing as ((playerId: number) => number) | undefined;")
    lines.append("    if (fn) return fn.call(this.handle, playerId);")
    lines.append("    return this._activeAudioPlayers.has(playerId) ? 1 : 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_active_count"))
    lines.append("  audioActiveCount(): number {")
    lines.append("    const fn = (this.handle as any).audio_active_count as (() => number) | undefined;")
    lines.append("    return fn ? fn.call(this.handle) : this._activeAudioPlayers.size;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_cleanup_finished"))
    lines.append("  audioCleanupFinished(): number {")
    lines.append("    const fn = (this.handle as any).audio_cleanup_finished as (() => number) | undefined;")
    lines.append("    return fn ? fn.call(this.handle) : 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audioPlaySpatial3d"))
    lines.append("  audioPlaySpatial3d(data: Uint8Array, sourceX: number, sourceY: number, sourceZ: number, listenerX: number, listenerY: number, listenerZ: number, maxDistance: number, rolloff: number): number {")
    lines.append("    const playerId = this.handle.audio_play_spatial_3d(data, sourceX, sourceY, sourceZ, listenerX, listenerY, listenerZ, maxDistance, rolloff);")
    lines.append("    if (playerId >= 0) this._activeAudioPlayers.add(playerId);")
    lines.append("    return playerId;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audioUpdateSpatial3d"))
    lines.append("  audioUpdateSpatial3d(playerId: number, sourceX: number, sourceY: number, sourceZ: number, listenerX: number, listenerY: number, listenerZ: number, maxDistance: number, rolloff: number): number {")
    lines.append("    return this.handle.audio_update_spatial_volume_3d(playerId, sourceX, sourceY, sourceZ, listenerX, listenerY, listenerZ, maxDistance, rolloff);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audioSetListenerPosition3d"))
    lines.append("  audioSetListenerPosition3d(x: number, y: number, z: number): number { return this.handle.audio_set_listener_position_3d(x, y, z); }")
    emit_jsdoc(lines, _method_docs.get("audioSetSourcePosition3d"))
    lines.append("  audioSetSourcePosition3d(playerId: number, x: number, y: number, z: number, maxDistance: number, rolloff: number): number {")
    lines.append("    return this.handle.audio_set_source_position_3d(playerId, x, y, z, maxDistance, rolloff);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_set_player_volume"))
    lines.append("  audioSetPlayerVolume(playerId: number, volume: number): number { return this.handle.audio_set_player_volume(playerId, volume); }")
    emit_jsdoc(lines, _method_docs.get("audio_set_player_speed"))
    lines.append("  audioSetPlayerSpeed(playerId: number, speed: number): number { return this.handle.audio_set_player_speed(playerId, speed); }")
    emit_jsdoc(lines, _method_docs.get("audio_crossfade"))
    lines.append("  audioCrossfade(fromPlayerId: number, toPlayerId: number, mix: number): number {")
    lines.append("    return this.handle.audio_crossfade(fromPlayerId, toPlayerId, mix);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_crossfade_to"))
    lines.append("  audioCrossfadeTo(fromPlayerId: number, data: Uint8Array, durationSec: number, channel: number): number {")
    lines.append("    const fn = (this.handle as any).audio_crossfade_to as ((fromPlayerId: number, data: Uint8Array, durationSec: number, channel: number) => number) | undefined;")
    lines.append("    const playerId = fn ? fn.call(this.handle, fromPlayerId, data, durationSec, channel) : this.handle.audio_play(data);")
    lines.append("    if (playerId >= 0) this._activeAudioPlayers.add(playerId);")
    lines.append("    if (!fn && playerId >= 0) this.handle.audio_crossfade(fromPlayerId, playerId, 1.0);")
    lines.append("    return playerId;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_mix_with"))
    lines.append("  audioMixWith(primaryPlayerId: number, data: Uint8Array, secondaryVolume: number, secondaryChannel: number): number {")
    lines.append("    const fn = (this.handle as any).audio_mix_with as ((primaryPlayerId: number, data: Uint8Array, secondaryVolume: number, secondaryChannel: number) => number) | undefined;")
    lines.append("    const playerId = fn ? fn.call(this.handle, primaryPlayerId, data, secondaryVolume, secondaryChannel) : this.handle.audio_play(data);")
    lines.append("    if (playerId >= 0) this._activeAudioPlayers.add(playerId);")
    lines.append("    if (!fn && playerId >= 0) {")
    lines.append("      this.handle.audio_set_player_volume(playerId, secondaryVolume);")
    lines.append("      const mix = Math.max(0, Math.min(1, secondaryVolume));")
    lines.append("      this.handle.audio_crossfade(primaryPlayerId, playerId, mix);")
    lines.append("    }")
    lines.append("    return playerId;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_update_crossfades"))
    lines.append("  audioUpdateCrossfades(deltaSec: number): number {")
    lines.append("    const fn = (this.handle as any).audio_update_crossfades as ((deltaSec: number) => number) | undefined;")
    lines.append("    return fn ? fn.call(this.handle, deltaSec) : 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_active_crossfade_count"))
    lines.append("  audioActiveCrossfadeCount(): number {")
    lines.append("    const fn = (this.handle as any).audio_active_crossfade_count as (() => number) | undefined;")
    lines.append("    return fn ? fn.call(this.handle) : 0;")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("audio_activate"))
    lines.append("  audioActivate(): number {")
    lines.append("    const fn = (this.handle as any).audio_activate as (() => number) | undefined;")
    lines.append("    return fn ? fn.call(this.handle) : 0;")
    lines.append("  }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("is_key_pressed"))
    lines.append("  isKeyPressed(key: number): boolean { return this.handle.is_key_pressed(key); }")
    emit_jsdoc(lines, _method_docs.get("is_key_just_pressed"))
    lines.append("  isKeyJustPressed(key: number): boolean { return this.handle.is_key_just_pressed(key); }")
    emit_jsdoc(lines, _method_docs.get("is_key_just_released"))
    lines.append("  isKeyJustReleased(key: number): boolean { return this.handle.is_key_just_released(key); }")
    emit_jsdoc(lines, _method_docs.get("is_mouse_button_pressed"))
    lines.append("  isMouseButtonPressed(button: number): boolean { return this.handle.is_mouse_button_pressed(button); }")
    emit_jsdoc(lines, _method_docs.get("is_mouse_button_just_pressed"))
    lines.append("  isMouseButtonJustPressed(button: number): boolean { return this.handle.is_mouse_button_just_pressed(button); }")
    emit_jsdoc(lines, _method_docs.get("is_mouse_button_just_released"))
    lines.append("  isMouseButtonJustReleased(button: number): boolean { return this.handle.is_mouse_button_just_released(button); }")
    emit_jsdoc(lines, _method_docs.get("get_mouse_position"))
    lines.append("  getMousePosition(): IVec2 { return { x: this.handle.mouse_x(), y: this.handle.mouse_y() }; }")
    emit_jsdoc(lines, _method_docs.get("get_mouse_delta"))
    lines.append("  getMouseDelta(): IVec2 { return { x: 0, y: 0 }; }")
    emit_jsdoc(lines, _method_docs.get("get_scroll_delta"))
    lines.append("  getScrollDelta(): IVec2 { return { x: this.handle.scroll_dx(), y: this.handle.scroll_dy() }; }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("map_action_key"))
    lines.append("  mapActionKey(action: string, key: number): boolean { return this.handle.map_action_key(action, key); }")
    emit_jsdoc(lines, _method_docs.get("is_action_pressed"))
    lines.append("  isActionPressed(action: string): boolean { return this.handle.is_action_pressed(action); }")
    emit_jsdoc(lines, _method_docs.get("is_action_just_pressed"))
    lines.append("  isActionJustPressed(action: string): boolean { return this.handle.is_action_just_pressed(action); }")
    emit_jsdoc(lines, _method_docs.get("is_action_just_released"))
    lines.append("  isActionJustReleased(action: string): boolean { return this.handle.is_action_just_released(action); }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("get_render_stats"))
    lines.append("  getRenderStats(): IRenderStats {")
    lines.append("    const s = this.handle.get_render_stats();")
    lines.append("    if (!s) return { drawCalls: 0, triangles: 0, textureBinds: 0, shaderBinds: 0 };")
    lines.append("    return { drawCalls: s.draw_calls, triangles: s.triangles, textureBinds: s.texture_binds, shaderBinds: 0 };")
    lines.append("  }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("spawn_empty"))
    lines.append("  spawnEmpty(): IEntity { return new WebEntity(this.handle.spawn_empty()); }")
    emit_jsdoc(lines, _method_docs.get("spawn_batch"))
    lines.append("  spawnBatch(count: number): IEntity[] {")
    lines.append("    const arr = this.handle.spawn_batch(count);")
    lines.append("    return Array.from(arr, bits => new WebEntity(bits));")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("despawn"))
    lines.append("  despawn(entity: IEntity): boolean { return this.handle.despawn(entity.toBits()); }")
    emit_jsdoc(lines, _method_docs.get("despawn_batch"))
    lines.append("  despawnBatch(entities: IEntity[]): number {")
    lines.append("    const bits = BigUint64Array.from(entities.map(e => e.toBits()));")
    lines.append("    return this.handle.despawn_batch(bits);")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("entity_count"))
    lines.append("  entityCount(): number { return this.handle.entity_count(); }")
    emit_jsdoc(lines, _method_docs.get("is_alive"))
    lines.append("  isAlive(entity: IEntity): boolean { return this.handle.is_alive(entity.toBits()); }")
    emit_jsdoc(lines, _method_docs.get("clone_entity"))
    lines.append("  cloneEntity(entity: IEntity): IEntity { return new WebEntity(this.handle.clone_entity(entity.toBits())); }")
    emit_jsdoc(lines, _method_docs.get("clone_entity_recursive"))
    lines.append("  cloneEntityRecursive(entity: IEntity): IEntity { return new WebEntity(this.handle.clone_entity_recursive(entity.toBits())); }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("add_transform2d"))
    lines.append("  addTransform2d(entity: IEntity, t: ITransform2DData): void { this.handle.add_transform2d(entity.toBits(), t.positionX, t.positionY, t.rotation, t.scaleX, t.scaleY); }")
    emit_jsdoc(lines, _method_docs.get("get_transform2d"))
    lines.append("  getTransform2d(entity: IEntity): ITransform2DData | null {")
    lines.append("    const t = this.handle.get_transform2d(entity.toBits());")
    lines.append("    if (!t) return null;")
    lines.append("    return { positionX: t.position_x, positionY: t.position_y, rotation: t.rotation, scaleX: t.scale_x, scaleY: t.scale_y };")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("set_transform2d"))
    lines.append("  setTransform2d(entity: IEntity, t: ITransform2DData): void { this.handle.set_transform2d(entity.toBits(), t.positionX, t.positionY, t.rotation, t.scaleX, t.scaleY); }")
    emit_jsdoc(lines, _method_docs.get("has_transform2d"))
    lines.append("  hasTransform2d(entity: IEntity): boolean { return this.handle.has_transform2d(entity.toBits()); }")
    emit_jsdoc(lines, _method_docs.get("remove_transform2d"))
    lines.append("  removeTransform2d(entity: IEntity): boolean { return this.handle.remove_transform2d(entity.toBits()); }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("add_sprite"))
    lines.append("  addSprite(entity: IEntity, s: ISpriteData): void { this.handle.add_sprite(entity.toBits(), s.textureHandle, s.colorR, s.colorG, s.colorB, s.colorA, s.flipX, s.flipY, s.zLayer, s.anchorX, s.anchorY); }")
    emit_jsdoc(lines, _method_docs.get("get_sprite"))
    lines.append("  getSprite(entity: IEntity): ISpriteData | null {")
    lines.append("    const s = this.handle.get_sprite(entity.toBits());")
    lines.append("    if (!s) return null;")
    lines.append("    return { textureHandle: s.texture_handle, colorR: s.r, colorG: s.g, colorB: s.b, colorA: s.a, sourceRectX: 0, sourceRectY: 0, sourceRectWidth: 0, sourceRectHeight: 0, hasSourceRect: false, flipX: s.flip_x, flipY: s.flip_y, zLayer: s.z_layer, anchorX: s.anchor_x, anchorY: s.anchor_y, customSizeX: 0, customSizeY: 0, hasCustomSize: false };")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("set_sprite"))
    lines.append("  setSprite(entity: IEntity, s: ISpriteData): void { this.handle.set_sprite(entity.toBits(), s.textureHandle, s.colorR, s.colorG, s.colorB, s.colorA, s.flipX, s.flipY, s.zLayer, s.anchorX, s.anchorY); }")
    emit_jsdoc(lines, _method_docs.get("has_sprite"))
    lines.append("  hasSprite(entity: IEntity): boolean { return this.handle.has_sprite(entity.toBits()); }")
    emit_jsdoc(lines, _method_docs.get("remove_sprite"))
    lines.append("  removeSprite(entity: IEntity): boolean { return this.handle.remove_sprite(entity.toBits()); }")
    lines.append("")

    emit_jsdoc(lines, _method_docs.get("add_name"))
    lines.append("  addName(entity: IEntity, name: string): void { this.handle.add_name(entity.toBits(), name); }")
    emit_jsdoc(lines, _method_docs.get("get_name"))
    lines.append("  getName(entity: IEntity): string | null { return this.handle.get_name(entity.toBits()) ?? null; }")
    emit_jsdoc(lines, _method_docs.get("has_name"))
    lines.append("  hasName(entity: IEntity): boolean { return this.handle.has_name(entity.toBits()); }")
    emit_jsdoc(lines, _method_docs.get("remove_name"))
    lines.append("  removeName(entity: IEntity): boolean { return this.handle.remove_name(entity.toBits()); }")
    lines.append("")

    # Scene management wrappers are exposed for API parity with Node/native SDKs.
    # Current WASM bridge does not yet expose scene loader exports.
    lines.append("  loadScene(_name: string, _json: string): number {")
    lines.append("    throw new Error('Not supported in WASM mode');")
    lines.append("  }")
    lines.append("  unloadScene(_name: string): boolean {")
    lines.append("    throw new Error('Not supported in WASM mode');")
    lines.append("  }")
    lines.append("  setActiveScene(_sceneId: number, _active: boolean): boolean {")
    lines.append("    throw new Error('Not supported in WASM mode');")
    lines.append("  }")
    lines.append("")
    lines.append("  networkHost(protocol: number, port: number): number {")
    lines.append("    return this.handle.network_host(protocol, port);")
    lines.append("  }")
    lines.append("  networkConnect(protocol: number, address: string, port: number): number {")
    lines.append("    return this.handle.network_connect(protocol, address, port);")
    lines.append("  }")
    lines.append("  networkConnectWithPeer(protocol: number, address: string, port: number): INetworkConnectResult {")
    lines.append("    const result = this.handle.network_connect_with_peer(protocol, address, port);")
    lines.append("    return { handle: result.handle, peerId: Number(result.peer_id) };")
    lines.append("  }")
    lines.append("  networkDisconnect(handle: number): number {")
    lines.append("    return this.handle.network_disconnect(handle);")
    lines.append("  }")
    lines.append("  networkSend(handle: number, peerId: number, data: Uint8Array, channel: number): number {")
    lines.append("    return this.handle.network_send(handle, BigInt(peerId), data, channel);")
    lines.append("  }")
    lines.append("  networkReceive(handle: number): Uint8Array {")
    lines.append("    return this.handle.network_receive(handle);")
    lines.append("  }")
    lines.append("  networkReceivePacket(handle: number): INetworkPacket | null {")
    lines.append("    const packet = this.handle.network_receive_packet(handle);")
    lines.append("    if (!packet) return null;")
    lines.append("    return { peerId: Number(packet.peer_id), data: packet.data };")
    lines.append("  }")
    lines.append("  networkPoll(handle: number): number {")
    lines.append("    return this.handle.network_poll(handle);")
    lines.append("  }")
    lines.append("  getNetworkStats(handle: number): INetworkStats {")
    lines.append("    const stats = this.handle.get_network_stats(handle);")
    lines.append("    return {")
    lines.append("      bytesSent: Number(stats.bytes_sent),")
    lines.append("      bytesReceived: Number(stats.bytes_received),")
    lines.append("      packetsSent: Number(stats.packets_sent),")
    lines.append("      packetsReceived: Number(stats.packets_received),")
    lines.append("      packetsLost: Number(stats.packets_lost),")
    lines.append("      rttMs: stats.rtt_ms,")
    lines.append("      sendBandwidthBytesPerSec: stats.send_bandwidth_bytes_per_sec,")
    lines.append("      receiveBandwidthBytesPerSec: stats.receive_bandwidth_bytes_per_sec,")
    lines.append("      packetLossPercent: stats.packet_loss_percent,")
    lines.append("      jitterMs: stats.jitter_ms,")
    lines.append("    };")
    lines.append("  }")
    lines.append("  networkPeerCount(handle: number): number {")
    lines.append("    return this.handle.network_peer_count(handle);")
    lines.append("  }")
    lines.append("  setNetworkSimulation(handle: number, config: INetworkSimulationConfig): number {")
    lines.append("    return this.handle.set_network_simulation(handle, config.oneWayLatencyMs, config.jitterMs, config.packetLossPercent);")
    lines.append("  }")
    lines.append("  clearNetworkSimulation(handle: number): number {")
    lines.append("    return this.handle.clear_network_simulation(handle);")
    lines.append("  }")
    lines.append("  setNetworkOverlayHandle(handle: number): number {")
    lines.append("    return this.handle.set_network_overlay_handle(handle);")
    lines.append("  }")
    lines.append("  clearNetworkOverlayHandle(): number {")
    lines.append("    return this.handle.clear_network_overlay_handle();")
    lines.append("  }")
    lines.append("")

    # P2P mesh stubs (not supported in WASM mode)
    lines.append("  p2pCreateMesh(_protocol: number, _port: number, _config: IP2pMeshConfig): number {")
    lines.append("    throw new Error('P2P mesh is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  p2pJoinMesh(_protocol: number, _address: string, _port: number, _config: IP2pMeshConfig): number {")
    lines.append("    throw new Error('P2P mesh is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  p2pLeaveMesh(_handle: number): number {")
    lines.append("    throw new Error('P2P mesh is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  p2pGetPeers(_handle: number): number {")
    lines.append("    throw new Error('P2P mesh is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  p2pGetHost(_handle: number): number {")
    lines.append("    throw new Error('P2P mesh is not supported in WASM mode');")
    lines.append("  }")
    lines.append("")

    # Rollback stubs (not supported in WASM mode)
    lines.append("  rollbackCreate(_config: IRollbackConfig, _localPlayer: number, _playerIds: Uint8Array, _statePtr: number, _advanceFn: number, _hashFn: number, _cloneFn: number, _freeFn: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackDestroy(_handle: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackAdvanceFrame(_handle: number, _input: Uint8Array): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackReceiveRemoteInput(_handle: number, _playerId: number, _frame: number, _input: Uint8Array): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackShouldRollback(_handle: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackResimulate(_handle: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackConfirmedFrame(_handle: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackCurrentFrame(_handle: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rollbackCheckDesync(_handle: number, _remoteHash: number, _frame: number): number {")
    lines.append("    throw new Error('Rollback netcode is not supported in WASM mode');")
    lines.append("  }")
    lines.append("")

    # RPC stubs (not supported in WASM mode)
    lines.append("  rpcCreate(_timeoutMs: number, _maxPayload: number): number {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcDestroy(_handle: number): number {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcRegister(_handle: number, _rpcId: number, _name: string, _direction: number): number {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcCall(_handle: number, _peerId: number, _rpcId: number, _payload: Uint8Array): number {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcPoll(_handle: number, _deltaSecs: number): number {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcProcessIncoming(_handle: number, _peerId: number, _data: Uint8Array): number {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcReceiveResponse(_handle: number, _callId: number): Uint8Array {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("  rpcDrainOne(_handle: number): Uint8Array {")
    lines.append("    throw new Error('RPC framework is not supported in WASM mode');")
    lines.append("  }")
    lines.append("")

    # Collision methods
    emit_jsdoc(lines, _method_docs.get("collision_aabb_aabb"))
    lines.append("  collisionAabbAabb(centerAx: number, centerAy: number, halfWa: number, halfHa: number, centerBx: number, centerBy: number, halfWb: number, halfHb: number): IContact | null {")
    lines.append("    const c = this.handle.collision_aabb_aabb(centerAx, centerAy, halfWa, halfHa, centerBx, centerBy, halfWb, halfHb);")
    lines.append("    if (!c) return null;")
    lines.append("    return { pointX: c.point_x, pointY: c.point_y, normalX: c.normal_x, normalY: c.normal_y, penetration: c.penetration };")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("collision_circle_circle"))
    lines.append("  collisionCircleCircle(centerAx: number, centerAy: number, radiusA: number, centerBx: number, centerBy: number, radiusB: number): IContact | null {")
    lines.append("    const c = this.handle.collision_circle_circle(centerAx, centerAy, radiusA, centerBx, centerBy, radiusB);")
    lines.append("    if (!c) return null;")
    lines.append("    return { pointX: c.point_x, pointY: c.point_y, normalX: c.normal_x, normalY: c.normal_y, penetration: c.penetration };")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("collision_circle_aabb"))
    lines.append("  collisionCircleAabb(circleX: number, circleY: number, circleRadius: number, boxX: number, boxY: number, boxHw: number, boxHh: number): IContact | null {")
    lines.append("    const c = this.handle.collision_circle_aabb(circleX, circleY, circleRadius, boxX, boxY, boxHw, boxHh);")
    lines.append("    if (!c) return null;")
    lines.append("    return { pointX: c.point_x, pointY: c.point_y, normalX: c.normal_x, normalY: c.normal_y, penetration: c.penetration };")
    lines.append("  }")
    emit_jsdoc(lines, _method_docs.get("point_in_rect"))
    lines.append("  pointInRect(px: number, py: number, rx: number, ry: number, rw: number, rh: number): boolean { return this.handle.point_in_rect(px, py, rx, ry, rw, rh); }")
    emit_jsdoc(lines, _method_docs.get("point_in_circle"))
    lines.append("  pointInCircle(px: number, py: number, cx: number, cy: number, radius: number): boolean { return this.handle.point_in_circle(px, py, cx, cy, radius); }")
    emit_jsdoc(lines, _method_docs.get("aabb_overlap"))
    lines.append("  aabbOverlap(minAx: number, minAy: number, maxAx: number, maxAy: number, minBx: number, minBy: number, maxBx: number, maxBy: number): boolean { return this.handle.aabb_overlap(minAx, minAy, maxAx, maxAy, minBx, minBy, maxBx, maxBy); }")
    emit_jsdoc(lines, _method_docs.get("circle_overlap"))
    lines.append("  circleOverlap(x1: number, y1: number, r1: number, x2: number, y2: number, r2: number): boolean { return this.handle.circle_overlap(x1, y1, r1, x2, y2, r2); }")
    emit_jsdoc(lines, _method_docs.get("distance"))
    lines.append("  distance(x1: number, y1: number, x2: number, y2: number): number { return this.handle.distance(x1, y1, x2, y2); }")
    emit_jsdoc(lines, _method_docs.get("distance_squared"))
    lines.append("  distanceSquared(x1: number, y1: number, x2: number, y2: number): number { return this.handle.distance_squared(x1, y1, x2, y2); }")
    lines.append("")

    # 3D methods - stub implementations for wasm (3D rendering not available in wasm build)
    lines.append("  // TODO: wasm 3D -- these stub methods satisfy the IGoudGame interface")
    lines.append("  createCube(_textureId: number, _width: number, _height: number, _depth: number): number { return 0; }")
    lines.append("  createPlane(_textureId: number, _width: number, _depth: number): number { return 0; }")
    lines.append("  createSphere(_textureId: number, _diameter: number, _segments = 16): number { return 0; }")
    lines.append("  createCylinder(_textureId: number, _radius: number, _height: number, _segments = 16): number { return 0; }")
    lines.append("  setObjectPosition(_objectId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setObjectRotation(_objectId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setObjectScale(_objectId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  destroyObject(_objectId: number): boolean { return false; }")
    lines.append("  addLight(_lightType: number, _posX: number, _posY: number, _posZ: number, _dirX: number, _dirY: number, _dirZ: number, _r: number, _g: number, _b: number, _intensity: number, _range: number, _spotAngle: number): number { return 0; }")
    lines.append("  updateLight(_lightId: number, _lightType: number, _posX: number, _posY: number, _posZ: number, _dirX: number, _dirY: number, _dirZ: number, _r: number, _g: number, _b: number, _intensity: number, _range: number, _spotAngle: number): boolean { return false; }")
    lines.append("  removeLight(_lightId: number): boolean { return false; }")
    lines.append("  setCameraPosition3D(_x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setCameraRotation3D(_pitch: number, _yaw: number, _roll: number): boolean { return false; }")
    lines.append("  configureGrid(_enabled: boolean, _size: number, _divisions: number): boolean { return false; }")
    lines.append("  setGridEnabled(_enabled: boolean): boolean { return false; }")
    lines.append("  configureSkybox(_enabled: boolean, _r: number, _g: number, _b: number, _a: number): boolean { return false; }")
    lines.append("  configureFog(_enabled: boolean, _r: number, _g: number, _b: number, _density: number): boolean { return false; }")
    lines.append("  setFogEnabled(_enabled: boolean): boolean { return false; }")
    lines.append("  render3D(): boolean { return false; }")
    lines.append("  createMaterial(_materialType: number, _r: number, _g: number, _b: number, _a: number, _shininess: number, _metallic: number, _roughness: number, _ao: number): number { return 0; }")
    lines.append("  updateMaterial(_materialId: number, _materialType: number, _r: number, _g: number, _b: number, _a: number, _shininess: number, _metallic: number, _roughness: number, _ao: number): boolean { return false; }")
    lines.append("  removeMaterial(_materialId: number): boolean { return false; }")
    lines.append("  setObjectMaterial(_objectId: number, _materialId: number): boolean { return false; }")
    lines.append("  getObjectMaterial(_objectId: number): number { return 0; }")
    lines.append("  createSkinnedMesh(_verticesPtr: number, _vertexCount: number): number { return 0; }")
    lines.append("  removeSkinnedMesh(_meshId: number): boolean { return false; }")
    lines.append("  setSkinnedMeshPosition(_meshId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setSkinnedMeshRotation(_meshId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setSkinnedMeshScale(_meshId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setSkinnedMeshBones(_meshId: number, _matricesPtr: number, _boneCount: number): boolean { return false; }")
    lines.append("  addBloomPass(_threshold: number, _intensity: number): number { return 0; }")
    lines.append("  addBlurPass(_radius: number): number { return 0; }")
    lines.append("  addColorGradePass(_exposure: number, _contrast: number, _saturation: number): number { return 0; }")
    lines.append("  removePostprocessPass(_index: number): boolean { return false; }")
    lines.append("  postprocessPassCount(): number { return 0; }")
    lines.append("  loadModel(_path: string): number { return 0; }")
    lines.append("  destroyModel(_modelId: number): boolean { return false; }")
    lines.append("  instantiateModel(_sourceModelId: number): number { return 0; }")
    lines.append("  setModelMaterial(_modelId: number, _meshIndex: number, _materialId: number): boolean { return false; }")
    lines.append("  getModelMeshCount(_modelId: number): number { return 0; }")
    lines.append("  getModelBoundingBox(_modelId: number): { minX: number; minY: number; minZ: number; maxX: number; maxY: number; maxZ: number } { return { minX: 0, minY: 0, minZ: 0, maxX: 0, maxY: 0, maxZ: 0 }; }")
    lines.append("  setModelPosition(_modelId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setModelRotation(_modelId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  setModelScale(_modelId: number, _x: number, _y: number, _z: number): boolean { return false; }")
    lines.append("  getAnimationCount(_modelId: number): number { return 0; }")
    lines.append("  getAnimationName(_modelId: number, _animIndex: number): string { return ''; }")
    lines.append("  playAnimation(_instanceId: number, _animIndex: number, _looping: boolean): boolean { return false; }")
    lines.append("  stopAnimation(_instanceId: number): boolean { return false; }")
    lines.append("  updateAnimations(_deltaTime: number): boolean { return false; }")
    lines.append("  createScene(_name: string): number { return 0; }")
    lines.append("  destroyScene(_sceneId: number): boolean { return false; }")
    lines.append("  setCurrentScene(_sceneId: number): boolean { return false; }")
    lines.append("  getCurrentScene(): number { return 0; }")
    lines.append("  clearCurrentScene(): boolean { return false; }")
    lines.append("  addObjectToScene(_sceneId: number, _objectId: number): boolean { return false; }")
    lines.append("  removeObjectFromScene(_sceneId: number, _objectId: number): boolean { return false; }")
    lines.append("  addModelToScene(_sceneId: number, _modelId: number): boolean { return false; }")
    lines.append("  removeModelFromScene(_sceneId: number, _modelId: number): boolean { return false; }")
    lines.append("  addLightToScene(_sceneId: number, _lightId: number): boolean { return false; }")
    lines.append("  removeLightFromScene(_sceneId: number, _lightId: number): boolean { return false; }")
    lines.append("")

    lines.append("  setViewport(_x: number, _y: number, _width: number, _height: number): void {}")
    lines.append("  enableDepthTest(): void {}")
    lines.append("  disableDepthTest(): void {}")
    lines.append("  clearDepth(): void {}")
    lines.append("  disableBlending(): void {}")
    lines.append("")
    # FPS overlay methods - stub implementations for wasm (overlay not available in wasm build)
    lines.append("  // TODO: wasm FPS overlay -- these stub methods satisfy the IGoudGame interface")
    lines.append("  getFpsStats(): IFpsStats { return { currentFps: this.handle.fps, minFps: 0, maxFps: 0, avgFps: 0, frameTimeMs: 0 }; }")
    lines.append("  getRenderMetrics(): IRenderMetrics { return { drawCallCount: 0, spritesSubmitted: 0, spritesDrawn: 0, spritesCulled: 0, batchesSubmitted: 0, avgSpritesPerBatch: 0, spriteRenderMs: 0, textRenderMs: 0, uiRenderMs: 0, totalRenderMs: 0, textDrawCalls: 0, textGlyphCount: 0, uiDrawCalls: 0 }; }")
    lines.append("  setFpsOverlayEnabled(_enabled: boolean): void {}")
    lines.append("  setFpsUpdateInterval(_interval: number): void {}")
    lines.append("  setFpsOverlayCorner(_corner: number): void {}")
    # Debugger methods -- delegate to WASM debugger runtime via dispatchDebuggerRequest
    lines.append("  getDebuggerSnapshotJson(): string { return this.handle.getDebuggerSnapshotJson(); }")
    lines.append("  getDebuggerManifestJson(): string { return '{}'; }")
    lines.append("  setDebuggerPaused(paused: boolean): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'set_paused', paused })); }")
    lines.append("  stepDebugger(kind: number, count: number): void { this.handle.dispatchDebuggerRequest(JSON.stringify(kind === 0 ? { verb: 'step', frames: count } : { verb: 'step', frames: 0, ticks: count })); }")
    lines.append("  setDebuggerTimeScale(scale: number): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'set_time_scale', time_scale: scale })); }")
    lines.append("  setDebuggerDebugDrawEnabled(enabled: boolean): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'set_debug_draw_enabled', enabled })); }")
    lines.append("  injectDebuggerKeyEvent(key: number, pressed: boolean): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'inject_input', events: [{ device: 'keyboard', action: pressed ? 'press' : 'release', key: String(key) }] })); }")
    lines.append("  injectDebuggerMouseButton(button: number, pressed: boolean): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'inject_input', events: [{ device: 'mouse', action: pressed ? 'press' : 'release', button: String(button) }] })); }")
    lines.append("  injectDebuggerMousePosition(position: IVec2): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'inject_input', events: [{ device: 'mouse', action: 'move', position: [position.x, position.y] }] })); }")
    lines.append("  injectDebuggerScroll(delta: IVec2): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'inject_input', events: [{ device: 'mouse', action: 'scroll', delta: [delta.x, delta.y] }] })); }")
    lines.append("  setDebuggerProfilingEnabled(enabled: boolean): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'set_profiling_enabled', enabled })); }")
    lines.append("  setDebuggerSelectedEntity(entityId: number): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'set_selected_entity', entity_id: entityId })); }")
    lines.append("  clearDebuggerSelectedEntity(): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'clear_selected_entity' })); }")
    lines.append("  getMemorySummary(): IMemorySummary { return { rendering: { currentBytes: 0, peakBytes: 0 }, assets: { currentBytes: 0, peakBytes: 0 }, ecs: { currentBytes: 0, peakBytes: 0 }, ui: { currentBytes: 0, peakBytes: 0 }, audio: { currentBytes: 0, peakBytes: 0 }, network: { currentBytes: 0, peakBytes: 0 }, debugger: { currentBytes: 0, peakBytes: 0 }, other: { currentBytes: 0, peakBytes: 0 }, totalCurrentBytes: 0, totalPeakBytes: 0 }; }")
    lines.append("  captureDebuggerFrame(): IDebuggerCapture {")
    lines.append("    const snapshot = this.handle.getDebuggerSnapshotJson();")
    lines.append("    let imagePng = new Uint8Array(0);")
    lines.append("    try {")
    lines.append("      const dataUrl = this.canvas.toDataURL('image/png');")
    lines.append("      const base64 = dataUrl.split(',')[1] ?? '';")
    lines.append("      const binary = atob(base64);")
    lines.append("      imagePng = new Uint8Array(binary.length);")
    lines.append("      for (let i = 0; i < binary.length; i++) imagePng[i] = binary.charCodeAt(i);")
    lines.append("    } catch {}")
    lines.append("    return { imagePng, metadataJson: '{}', snapshotJson: snapshot, metricsTraceJson: '{}' };")
    lines.append("  }")
    lines.append("  startDebuggerRecording(): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'start_recording' })); }")
    lines.append("  stopDebuggerRecording(): IDebuggerReplayArtifact { const r = this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'stop_recording' })); const parsed = JSON.parse(r); return { manifestJson: parsed?.result?.manifest_json ?? '{}', data: new Uint8Array(parsed?.result?.data ?? []) }; }")
    lines.append("  startDebuggerReplay(recording: Uint8Array): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'start_replay', data: Array.from(recording) })); }")
    lines.append("  stopDebuggerReplay(): void { this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'stop_replay' })); }")
    lines.append("  getDebuggerReplayStatusJson(): string { return this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'get_replay_status' })); }")
    lines.append("  getDebuggerMetricsTraceJson(): string { return this.handle.dispatchDebuggerRequest(JSON.stringify({ verb: 'get_metrics_trace' })); }")
    lines.append("")
    lines.append("  // TODO: wasm animation -- these stub methods satisfy the IGoudGame interface")
    lines.append("  play(_entity: IEntity): number { return 0; }")
    lines.append("  setState(_entity: IEntity, _stateName: string): number { return 0; }")
    lines.append("  setParameterBool(_entity: IEntity, _name: string, _value: boolean): number { return 0; }")
    lines.append("  setParameterFloat(_entity: IEntity, _name: string, _value: number): number { return 0; }")
    lines.append("  animationLayerStackCreate(_entity: IEntity): number { return 0; }")
    lines.append("  animationLayerAdd(_entity: IEntity, _name: string, _blendMode: number): number { return 0; }")
    lines.append("  animationLayerSetWeight(_entity: IEntity, _layerIndex: number, _weight: number): number { return 0; }")
    lines.append("  animationLayerPlay(_entity: IEntity, _layerIndex: number): number { return 0; }")
    lines.append("  animationLayerSetClip(_entity: IEntity, _layerIndex: number, _frameCount: number, _frameDuration: number, _mode: number): number { return 0; }")
    lines.append("  animationLayerAddFrame(_entity: IEntity, _layerIndex: number, _x: number, _y: number, _w: number, _h: number): number { return 0; }")
    lines.append("  animationLayerReset(_entity: IEntity, _layerIndex: number): number { return 0; }")
    lines.append("  animationClipAddEvent(_entity: IEntity, _frameIndex: number, _name: string, _payloadType: number, _payloadInt: number, _payloadFloat: number, _payloadString?: string | null): number { return 0; }")
    lines.append("  animationEventsCount(): number { return 0; }")
    lines.append("  animationEventsRead(_index: number): IAnimationEventData { return { entity: 0, name: '', frameIndex: 0, payloadType: 0, payloadInt: 0, payloadFloat: 0, payloadString: '' }; }")
    lines.append("  setAnimationSpeed(_instanceId: number, _speed: number): boolean { return false; }")
    lines.append("  blendAnimations(_instanceId: number, _animA: number, _animB: number, _blendFactor: number): boolean { return false; }")
    lines.append("  transitionAnimation(_instanceId: number, _targetAnim: number, _duration: number): boolean { return false; }")
    lines.append("  isAnimationPlaying(_instanceId: number): boolean { return false; }")
    lines.append("  getAnimationProgress(_instanceId: number): number { return 0; }")
    lines.append("  createCharacterController(_radius: number, _halfHeight: number, _x: number, _y: number, _z: number, _maxSlopeAngle: number, _stepHeight: number): number { return 0; }")
    lines.append("  destroyCharacterController(_controllerId: number): number { return 0; }")
    lines.append("  moveCharacter(_controllerId: number, _dx: number, _dy: number, _dz: number, _dt: number): any { return {}; }")
    lines.append("  getCharacterPosition(_controllerId: number): any { return { x: 0, y: 0, z: 0 }; }")
    lines.append("  isCharacterGrounded(_controllerId: number): boolean { return false; }")
    emit_jsdoc(lines, _method_docs.get("physics_raycast_ex"))
    lines.append("  physicsRaycastEx(_originX: number, _originY: number, _dirX: number, _dirY: number, _maxDist: number, _layerMask: number): IPhysicsRaycastHit2D | null { return null; }")
    emit_jsdoc(lines, _method_docs.get("physics_collision_events_count"))
    lines.append("  physicsCollisionEventsCount(): number { throw new Error('Not supported in WASM mode'); }")
    emit_jsdoc(lines, _method_docs.get("physics_collision_events_read"))
    lines.append("  physicsCollisionEventsRead(_index: number): IPhysicsCollisionEvent2D | null { return null; }")
    emit_jsdoc(lines, _method_docs.get("physics_set_collision_callback"))
    lines.append("  physicsSetCollisionCallback(_callbackPtr: number, _userData: number): number { throw new Error('Not supported in WASM mode'); }")
    lines.append("  // Generic component operations (WASM stubs — full support tracked in #606)")
    lines.append("  componentRegisterType(_typeIdHash: number, _name: string, _size: number, _align: number): boolean { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentAdd(_entity: IEntity, _typeIdHash: number, _dataPtr: number, _dataSize: number): boolean { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentRemove(_entity: IEntity, _typeIdHash: number): boolean { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentHas(_entity: IEntity, _typeIdHash: number): boolean { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentGet(_entity: IEntity, _typeIdHash: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentGetMut(_entity: IEntity, _typeIdHash: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentAddBatch(_entities: IEntity[], _typeIdHash: number, _dataPtr: number, _componentSize: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentRemoveBatch(_entities: IEntity[], _typeIdHash: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentHasBatch(_entities: IEntity[], _typeIdHash: number, _outResults: number[]): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentCount(_typeIdHash: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentGetEntities(_typeIdHash: number, _outEntities: number, _maxCount: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  componentGetAll(_typeIdHash: number, _outEntities: number, _outDataPtrs: number, _maxCount: number): number { throw new Error('Generic components not yet supported in WASM mode'); }")
    lines.append("  // Provider capability queries")
    lines.append("  getRenderCapabilities(): IRenderCapabilities { return WEB_RENDER_CAPABILITIES; }")
    lines.append("  getPhysicsCapabilities(): IPhysicsCapabilities { return WEB_PHYSICS_CAPABILITIES; }")
    lines.append("  getAudioCapabilities(): IAudioCapabilities { return WEB_AUDIO_CAPABILITIES; }")
    lines.append("  getInputCapabilities(): IInputCapabilities { return WEB_INPUT_CAPABILITIES; }")
    lines.append("  getNetworkCapabilities(): INetworkCapabilities {")
    lines.append("    const caps = this.handle.get_network_capabilities();")
    lines.append("    return {")
    lines.append("      supportsHosting: caps.supports_hosting,")
    lines.append("      maxConnections: caps.max_connections,")
    lines.append("      maxChannels: caps.max_channels,")
    lines.append("      maxMessageSize: caps.max_message_size,")
    lines.append("    };")
    lines.append("  }")
    lines.append("")
    lines.append("  // TODO: wasm coordinate origin -- these stub methods satisfy the IGoudGame interface")
    lines.append("  setCoordinateOrigin(_origin: number): boolean { return true; }")
    lines.append("  getCoordinateOrigin(): number { return 0; }")
    lines.append("")
    emit_jsdoc(lines, _method_docs.get("check_hot_swap_shortcut"))
    lines.append("  checkHotSwapShortcut(): boolean { throw new Error('Not supported in WASM mode'); }")
    lines.append("")
    lines.append("  /** Connect to a WebSocket debugger relay for MCP tool support. */")
    lines.append("  connectDebugger(wsUrl?: string): void {")
    lines.append("    const url = wsUrl ?? 'ws://127.0.0.1:9229';")
    lines.append("    (this.handle as any).initDebugger?.('web-game');")
    lines.append("    const ws = new WebSocket(url);")
    lines.append("    ws.onopen = () => {")
    lines.append("      ws.send(JSON.stringify({ route_label: 'web-game', surface_kind: 'wasm_browser', capabilities: ['capture', 'control_plane', 'replay'] }));")
    lines.append("    };")
    lines.append("    ws.onmessage = (event) => {")
    lines.append("      const request = JSON.parse(event.data);")
    lines.append("      if (request.type === 'registration_ack') return;")
    lines.append("      if (request.verb === 'capture_frame') {")
    lines.append("        const snapshot = (this.handle as any).getDebuggerSnapshotJson?.() ?? '{}';")
    lines.append("        let imagePngBase64 = '';")
    lines.append("        try { imagePngBase64 = this.canvas.toDataURL('image/png').split(',')[1] ?? ''; } catch {}")
    lines.append("        ws.send(JSON.stringify({ ok: true, result: { image_png_base64: imagePngBase64, snapshot_json: snapshot, metadata_json: '{}', metrics_trace_json: '{}' } }));")
    lines.append("        return;")
    lines.append("      }")
    lines.append("      const response = (this.handle as any).dispatchDebuggerRequest?.(JSON.stringify(request)) ?? '{}';")
    lines.append("      ws.send(response);")
    lines.append("    };")
    lines.append("  }")
    lines.append("}")
    lines.append("")

    # EngineConfig stub for web (not fully supported in WASM)
    if "EngineConfig" in schema.get("tools", {}) and "EngineConfig" in mapping.get("tools", {}):
        ec_tool = schema["tools"]["EngineConfig"]
        emit_jsdoc(lines, ec_tool.get("doc"), "")
        lines.append("export class EngineConfig {")
        lines.append("  private _config: WebGameConfig;")
        lines.append("")
        lines.append("  constructor() {")
        lines.append("    this._config = {};")
        lines.append("  }")
        lines.append("")
        for method in ec_tool.get("methods", []):
            mn = to_camel(method["name"])
            params = method.get("params", [])
            if mn in ("setVsync", "setFullscreen", "setTargetFps"):
                # WASM target does not support vsync, fullscreen, or target FPS.
                # These settings are silently ignored so we omit the setters entirely.
                continue
            emit_jsdoc(lines, method.get("doc"))
            if mn == "build":
                lines.append("  async build(): Promise<GoudGame> {")
                lines.append("    const game = await GoudGame.create(this._config);")
                lines.append("    if ((this._config as any).debugger?.enabled) {")
                lines.append("      (game as any).handle.initDebugger?.((this._config as any).debugger.routeLabel ?? 'web');")
                lines.append("    }")
                lines.append("    return game;")
                lines.append("  }")
            elif mn == "destroy":
                lines.append("  destroy(): void {}")
            elif mn == "setTitle":
                lines.append("  setTitle(title: string): EngineConfig {")
                lines.append("    this._config.title = title;")
                lines.append("    return this;")
                lines.append("  }")
            elif mn == "setSize":
                lines.append("  setSize(width: number, height: number): EngineConfig {")
                lines.append("    this._config.width = width;")
                lines.append("    this._config.height = height;")
                lines.append("    return this;")
                lines.append("  }")
            elif mn == "setFpsOverlay":
                lines.append("  setFpsOverlay(_enabled: boolean): EngineConfig {")
                lines.append("    // FPS overlay is not yet supported in WASM; accepted for API parity.")
                lines.append("    return this;")
                lines.append("  }")
            elif mn == "setDebugger":
                lines.append("  setDebugger(debuggerConfig: IDebuggerConfig): EngineConfig {")
                lines.append("    (this._config as any).debugger = debuggerConfig;")
                lines.append("    return this;")
                lines.append("  }")
            else:
                ps = ", ".join(f"_{to_camel(p['name'])}: {ts_type(p['type'])}" for p in params)
                lines.append(f"  {mn}({ps}): EngineConfig {{")
                lines.append("    return this;")
                lines.append("  }")
            lines.append("")
        lines.append("}")
        lines.append("")

    write_generated(GEN / "web" / "index.g.ts", "\n".join(lines))


def gen_errors():
    """Generate errors.g.ts for the web target.

    The output is identical to the Node target since error classes are
    pure TypeScript with no napi/wasm-specific code.
    """
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

    for cat in categories:
        lines.append(f"export class {cat['base_class']} extends GoudError {{}}")
    lines.append("")

    lines.append("const CATEGORY_CLASS_MAP: Record<string, typeof GoudError> = {")
    for cat in categories:
        lines.append(f"  {cat['name']}: {cat['base_class']},")
    lines += ["};", ""]

    lines.append("function categoryFromCode(code: number): string {")
    sorted_cats = sorted(categories, key=lambda c: c["range_start"], reverse=True)
    for cat in sorted_cats:
        lines.append(f'  if (code >= {cat["range_start"]}) return "{cat["name"]}";')
    lines += ['  return "Unknown";', "}", ""]

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

    lines.append("/** Static hint lookup matching the codegen schema. */")
    lines.append("function hintFromCode(code: number): string {")
    lines.append('  return HINTS[code] ?? "";')
    lines += ["}", ""]

    lines.append("const HINTS: Record<number, string> = {")
    for c in codes:
        lines.append(f'  {c["code"]}: "{c["hint"]}",')
    lines += ["};", ""]

    write_generated(GEN / "web" / "errors.g.ts", "\n".join(lines))


if __name__ == "__main__":
    print("Generating TypeScript Web SDK...")
    gen_network_shared_wrapper()
    gen_web_input()
    gen_web_wrapper()
    gen_errors()
    print("TypeScript Web SDK generation complete.")
