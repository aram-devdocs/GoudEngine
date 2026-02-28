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
    HEADER_COMMENT, SDKS_DIR, load_schema, load_ffi_mapping,
    to_camel, to_pascal, write_generated, TYPESCRIPT_TYPES,
)

TS = SDKS_DIR / "typescript"
GEN = TS / "src" / "generated"
schema = load_schema()
mapping = load_ffi_mapping()


def ts_type(t: str) -> str:
    base = t.rstrip("?")
    mapped = TYPESCRIPT_TYPES.get(base, base)
    if t.endswith("?"):
        return f"{mapped} | null"
    return mapped


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
        "import type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact } from '../types/engine.g.js';",
        "import { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "import { attachInputHandlers } from './input.g.js';",
        "",
        "export { Color, Vec2, Vec3 } from '../types/math.g.js';",
        "export { Key, MouseButton } from '../types/input.g.js';",
        "export { Rect } from '../types/math.g.js';",
        "export type { IGoudGame, IEntity, IColor, IVec2, ITransform2DData, ISpriteData, IRenderStats, IContact } from '../types/engine.g.js';",
        "",
    ]

    lines.append("interface WasmGameHandle {")
    lines.append("  free(): void;")
    lines.append("  begin_frame(dt: number): void;")
    lines.append("  end_frame(): void;")
    lines.append("  set_clear_color(r: number, g: number, b: number, a: number): void;")
    lines.append("  readonly delta_time: number;")
    lines.append("  readonly total_time: number;")
    lines.append("  readonly fps: number;")
    lines.append("  readonly frame_count: bigint;")
    lines.append("  readonly window_width: number;")
    lines.append("  readonly window_height: number;")
    lines.append("  set_canvas_size(w: number, h: number): void;")
    lines.append("  has_renderer(): boolean;")
    lines.append("  load_texture(url: string): Promise<number>;")
    lines.append("  destroy_texture(handle: number): void;")
    lines.append("  draw_sprite(t: number, x: number, y: number, w: number, h: number, rot: number, r: number, g: number, b: number, a: number): void;")
    lines.append("  draw_sprite_rect(t: number, x: number, y: number, w: number, h: number, rot: number, src_x: number, src_y: number, src_w: number, src_h: number, r: number, g: number, b: number, a: number): boolean;")
    lines.append("  draw_quad(x: number, y: number, w: number, h: number, r: number, g: number, b: number, a: number): void;")
    lines.append("  spawn_empty(): bigint;")
    lines.append("  spawn_batch(count: number): BigUint64Array;")
    lines.append("  despawn(bits: bigint): boolean;")
    lines.append("  despawn_batch(bits: BigUint64Array): number;")
    lines.append("  entity_count(): number;")
    lines.append("  is_alive(bits: bigint): boolean;")
    lines.append("  add_transform2d(bits: bigint, px: number, py: number, r: number, sx: number, sy: number): void;")
    lines.append("  get_transform2d(bits: bigint): { position_x: number; position_y: number; rotation: number; scale_x: number; scale_y: number } | undefined;")
    lines.append("  set_transform2d(bits: bigint, px: number, py: number, r: number, sx: number, sy: number): void;")
    lines.append("  has_transform2d(bits: bigint): boolean;")
    lines.append("  remove_transform2d(bits: bigint): boolean;")
    lines.append("  add_sprite(bits: bigint, texture_handle: number, r: number, g: number, b: number, a: number, flip_x: boolean, flip_y: boolean, anchor_x: number, anchor_y: number): void;")
    lines.append("  get_sprite(bits: bigint): { texture_handle: number; r: number; g: number; b: number; a: number; flip_x: boolean; flip_y: boolean; anchor_x: number; anchor_y: number } | undefined;")
    lines.append("  set_sprite(bits: bigint, texture_handle: number, r: number, g: number, b: number, a: number, flip_x: boolean, flip_y: boolean, anchor_x: number, anchor_y: number): void;")
    lines.append("  has_sprite(bits: bigint): boolean;")
    lines.append("  remove_sprite(bits: bigint): boolean;")
    lines.append("  add_name(bits: bigint, name: string): void;")
    lines.append("  get_name(bits: bigint): string | undefined;")
    lines.append("  has_name(bits: bigint): boolean;")
    lines.append("  remove_name(bits: bigint): boolean;")
    lines.append("  press_key(kc: number): void;")
    lines.append("  release_key(kc: number): void;")
    lines.append("  press_mouse_button(btn: number): void;")
    lines.append("  release_mouse_button(btn: number): void;")
    lines.append("  set_mouse_position(x: number, y: number): void;")
    lines.append("  add_scroll_delta(dx: number, dy: number): void;")
    lines.append("  is_key_pressed(kc: number): boolean;")
    lines.append("  is_key_just_pressed(kc: number): boolean;")
    lines.append("  is_key_just_released(kc: number): boolean;")
    lines.append("  is_mouse_button_pressed(btn: number): boolean;")
    lines.append("  is_mouse_button_just_pressed(btn: number): boolean;")
    lines.append("  mouse_x(): number;")
    lines.append("  mouse_y(): number;")
    lines.append("  scroll_dx(): number;")
    lines.append("  scroll_dy(): number;")
    lines.append("  map_action_key(action: string, key: number): boolean;")
    lines.append("  is_action_pressed(action: string): boolean;")
    lines.append("  is_action_just_pressed(action: string): boolean;")
    lines.append("  is_action_just_released(action: string): boolean;")
    lines.append("  get_render_stats(): { draw_calls: number; triangles: number; texture_binds: number } | undefined;")
    lines.append("  collision_aabb_aabb(cax: number, cay: number, hwa: number, hha: number, cbx: number, cby: number, hwb: number, hhb: number): { point_x: number; point_y: number; normal_x: number; normal_y: number; penetration: number } | undefined;")
    lines.append("  collision_circle_circle(cax: number, cay: number, ra: number, cbx: number, cby: number, rb: number): { point_x: number; point_y: number; normal_x: number; normal_y: number; penetration: number } | undefined;")
    lines.append("  collision_circle_aabb(cx: number, cy: number, cr: number, bx: number, by: number, bhw: number, bhh: number): { point_x: number; point_y: number; normal_x: number; normal_y: number; penetration: number } | undefined;")
    lines.append("  point_in_rect(px: number, py: number, rx: number, ry: number, rw: number, rh: number): boolean;")
    lines.append("  point_in_circle(px: number, py: number, cx: number, cy: number, r: number): boolean;")
    lines.append("  aabb_overlap(minax: number, minay: number, maxax: number, maxay: number, minbx: number, minby: number, maxbx: number, maxby: number): boolean;")
    lines.append("  circle_overlap(x1: number, y1: number, r1: number, x2: number, y2: number, r2: number): boolean;")
    lines.append("  distance(x1: number, y1: number, x2: number, y2: number): number;")
    lines.append("  distance_squared(x1: number, y1: number, x2: number, y2: number): number;")
    lines.append("}")
    lines.append("")

    lines.append("interface WasmExports {")
    lines.append("  WasmGame: {")
    lines.append("    new(width: number, height: number, title: string): WasmGameHandle;")
    lines.append("    createWithCanvas(canvas: HTMLCanvasElement, w: number, h: number, title: string): Promise<WasmGameHandle>;")
    lines.append("  };")
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

    lines.append("export class GoudGame implements IGoudGame {")
    lines.append("  private handle: WasmGameHandle;")
    lines.append("  private canvas: HTMLCanvasElement;")
    lines.append("  private detachInput: (() => void) | null = null;")
    lines.append("  private rafId = 0;")
    lines.append("  private running = false;")
    lines.append("  private lastTs = 0;")
    lines.append("  private _shouldClose = false;")
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

    for prop in tool["properties"]:
        pn = to_camel(prop["name"])
        if pn == "deltaTime":
            lines.append(f"  get {pn}(): number {{ return this.handle.delta_time; }}")
        elif pn == "fps":
            lines.append(f"  get {pn}(): number {{ return this.handle.fps; }}")
        elif pn == "windowWidth":
            lines.append(f"  get {pn}(): number {{ return this.handle.window_width; }}")
        elif pn == "windowHeight":
            lines.append(f"  get {pn}(): number {{ return this.handle.window_height; }}")
    lines.append("")

    lines.append("  setClearColor(r: number, g: number, b: number, a: number): void { this.handle.set_clear_color(r, g, b, a); }")
    lines.append("")
    lines.append("  shouldClose(): boolean { return this._shouldClose; }")
    lines.append("  close(): void { this._shouldClose = true; this.stop(); }")
    lines.append("  destroy(): void { this.stop(); this.handle.free(); }")
    lines.append("")
    lines.append("  beginFrame(r = 0, g = 0, b = 0, a = 1): void {")
    lines.append("    this.handle.set_clear_color(r, g, b, a);")
    lines.append("    const now = performance.now();")
    lines.append("    const dtMs = now - (this.lastTs || now);")
    lines.append("    this.lastTs = now;")
    lines.append("    this.handle.begin_frame(dtMs / 1000);")
    lines.append("  }")
    lines.append("  endFrame(): void { this.handle.end_frame(); }")
    lines.append("")
    lines.append("  run(update: (dt: number) => void): void {")
    lines.append("    if (this.running) return;")
    lines.append("    this.running = true; this.lastTs = performance.now();")
    lines.append("    this.detachInput = attachInputHandlers(this.canvas, this.handle);")
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
    lines.append("  stop(): void {")
    lines.append("    this.running = false;")
    lines.append("    if (this.rafId) { cancelAnimationFrame(this.rafId); this.rafId = 0; }")
    lines.append("    if (this.detachInput) { this.detachInput(); this.detachInput = null; }")
    lines.append("  }")
    lines.append("")

    lines.append("  async loadTexture(path: string): Promise<number> { return this.handle.load_texture(path); }")
    lines.append("  destroyTexture(handle: number): void { this.handle.destroy_texture(handle); }")
    lines.append("  drawSprite(texture: number, x: number, y: number, width: number, height: number, rotation = 0, color?: IColor): void {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    this.handle.draw_sprite(texture, x, y, width, height, rotation, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    lines.append("  drawSpriteRect(texture: number, x: number, y: number, width: number, height: number, rotation = 0, srcX = 0, srcY = 0, srcW = 1, srcH = 1, color?: IColor): boolean {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    return this.handle.draw_sprite_rect(texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    lines.append("  drawQuad(x: number, y: number, width: number, height: number, color?: IColor): void {")
    lines.append("    const c = color ?? Color.white();")
    lines.append("    this.handle.draw_quad(x, y, width, height, c.r, c.g, c.b, c.a);")
    lines.append("  }")
    lines.append("")

    lines.append("  isKeyPressed(key: number): boolean { return this.handle.is_key_pressed(key); }")
    lines.append("  isKeyJustPressed(key: number): boolean { return this.handle.is_key_just_pressed(key); }")
    lines.append("  isKeyJustReleased(key: number): boolean { return this.handle.is_key_just_released(key); }")
    lines.append("  isMouseButtonPressed(button: number): boolean { return this.handle.is_mouse_button_pressed(button); }")
    lines.append("  isMouseButtonJustPressed(button: number): boolean { return this.handle.is_mouse_button_just_pressed(button); }")
    lines.append("  isMouseButtonJustReleased(button: number): boolean { return false; }")
    lines.append("  getMousePosition(): IVec2 { return { x: this.handle.mouse_x(), y: this.handle.mouse_y() }; }")
    lines.append("  getMouseDelta(): IVec2 { return { x: 0, y: 0 }; }")
    lines.append("  getScrollDelta(): IVec2 { return { x: this.handle.scroll_dx(), y: this.handle.scroll_dy() }; }")
    lines.append("")

    lines.append("  mapActionKey(action: string, key: number): boolean { return this.handle.map_action_key(action, key); }")
    lines.append("  isActionPressed(action: string): boolean { return this.handle.is_action_pressed(action); }")
    lines.append("  isActionJustPressed(action: string): boolean { return this.handle.is_action_just_pressed(action); }")
    lines.append("  isActionJustReleased(action: string): boolean { return this.handle.is_action_just_released(action); }")
    lines.append("")

    lines.append("  getRenderStats(): IRenderStats {")
    lines.append("    const s = this.handle.get_render_stats();")
    lines.append("    if (!s) return { drawCalls: 0, triangles: 0, textureBinds: 0 };")
    lines.append("    return { drawCalls: s.draw_calls, triangles: s.triangles, textureBinds: s.texture_binds };")
    lines.append("  }")
    lines.append("")

    lines.append("  spawnEmpty(): IEntity { return new WebEntity(this.handle.spawn_empty()); }")
    lines.append("  spawnBatch(count: number): IEntity[] {")
    lines.append("    const arr = this.handle.spawn_batch(count);")
    lines.append("    return Array.from(arr, bits => new WebEntity(bits));")
    lines.append("  }")
    lines.append("  despawn(entity: IEntity): boolean { return this.handle.despawn(entity.toBits()); }")
    lines.append("  despawnBatch(entities: IEntity[]): number {")
    lines.append("    const bits = BigUint64Array.from(entities.map(e => e.toBits()));")
    lines.append("    return this.handle.despawn_batch(bits);")
    lines.append("  }")
    lines.append("  entityCount(): number { return this.handle.entity_count(); }")
    lines.append("  isAlive(entity: IEntity): boolean { return this.handle.is_alive(entity.toBits()); }")
    lines.append("")

    lines.append("  addTransform2d(entity: IEntity, t: ITransform2DData): void { this.handle.add_transform2d(entity.toBits(), t.positionX, t.positionY, t.rotation, t.scaleX, t.scaleY); }")
    lines.append("  getTransform2d(entity: IEntity): ITransform2DData | null {")
    lines.append("    const t = this.handle.get_transform2d(entity.toBits());")
    lines.append("    if (!t) return null;")
    lines.append("    return { positionX: t.position_x, positionY: t.position_y, rotation: t.rotation, scaleX: t.scale_x, scaleY: t.scale_y };")
    lines.append("  }")
    lines.append("  setTransform2d(entity: IEntity, t: ITransform2DData): void { this.handle.set_transform2d(entity.toBits(), t.positionX, t.positionY, t.rotation, t.scaleX, t.scaleY); }")
    lines.append("  hasTransform2d(entity: IEntity): boolean { return this.handle.has_transform2d(entity.toBits()); }")
    lines.append("  removeTransform2d(entity: IEntity): boolean { return this.handle.remove_transform2d(entity.toBits()); }")
    lines.append("")

    lines.append("  addSprite(entity: IEntity, s: ISpriteData): void { this.handle.add_sprite(entity.toBits(), s.textureHandle, s.color.r, s.color.g, s.color.b, s.color.a, s.flipX, s.flipY, s.anchorX, s.anchorY); }")
    lines.append("  getSprite(entity: IEntity): ISpriteData | null {")
    lines.append("    const s = this.handle.get_sprite(entity.toBits());")
    lines.append("    if (!s) return null;")
    lines.append("    return { textureHandle: s.texture_handle, color: { r: s.r, g: s.g, b: s.b, a: s.a }, flipX: s.flip_x, flipY: s.flip_y, anchorX: s.anchor_x, anchorY: s.anchor_y };")
    lines.append("  }")
    lines.append("  setSprite(entity: IEntity, s: ISpriteData): void { this.handle.set_sprite(entity.toBits(), s.textureHandle, s.color.r, s.color.g, s.color.b, s.color.a, s.flipX, s.flipY, s.anchorX, s.anchorY); }")
    lines.append("  hasSprite(entity: IEntity): boolean { return this.handle.has_sprite(entity.toBits()); }")
    lines.append("  removeSprite(entity: IEntity): boolean { return this.handle.remove_sprite(entity.toBits()); }")
    lines.append("")

    lines.append("  addName(entity: IEntity, name: string): void { this.handle.add_name(entity.toBits(), name); }")
    lines.append("  getName(entity: IEntity): string | null { return this.handle.get_name(entity.toBits()) ?? null; }")
    lines.append("  hasName(entity: IEntity): boolean { return this.handle.has_name(entity.toBits()); }")
    lines.append("  removeName(entity: IEntity): boolean { return this.handle.remove_name(entity.toBits()); }")
    lines.append("")

    # Collision methods
    lines.append("  collisionAabbAabb(centerAx: number, centerAy: number, halfWa: number, halfHa: number, centerBx: number, centerBy: number, halfWb: number, halfHb: number): IContact | null {")
    lines.append("    const c = this.handle.collision_aabb_aabb(centerAx, centerAy, halfWa, halfHa, centerBx, centerBy, halfWb, halfHb);")
    lines.append("    if (!c) return null;")
    lines.append("    return { pointX: c.point_x, pointY: c.point_y, normalX: c.normal_x, normalY: c.normal_y, penetration: c.penetration };")
    lines.append("  }")
    lines.append("  collisionCircleCircle(centerAx: number, centerAy: number, radiusA: number, centerBx: number, centerBy: number, radiusB: number): IContact | null {")
    lines.append("    const c = this.handle.collision_circle_circle(centerAx, centerAy, radiusA, centerBx, centerBy, radiusB);")
    lines.append("    if (!c) return null;")
    lines.append("    return { pointX: c.point_x, pointY: c.point_y, normalX: c.normal_x, normalY: c.normal_y, penetration: c.penetration };")
    lines.append("  }")
    lines.append("  collisionCircleAabb(circleX: number, circleY: number, circleRadius: number, boxX: number, boxY: number, boxHw: number, boxHh: number): IContact | null {")
    lines.append("    const c = this.handle.collision_circle_aabb(circleX, circleY, circleRadius, boxX, boxY, boxHw, boxHh);")
    lines.append("    if (!c) return null;")
    lines.append("    return { pointX: c.point_x, pointY: c.point_y, normalX: c.normal_x, normalY: c.normal_y, penetration: c.penetration };")
    lines.append("  }")
    lines.append("  pointInRect(px: number, py: number, rx: number, ry: number, rw: number, rh: number): boolean { return this.handle.point_in_rect(px, py, rx, ry, rw, rh); }")
    lines.append("  pointInCircle(px: number, py: number, cx: number, cy: number, radius: number): boolean { return this.handle.point_in_circle(px, py, cx, cy, radius); }")
    lines.append("  aabbOverlap(minAx: number, minAy: number, maxAx: number, maxAy: number, minBx: number, minBy: number, maxBx: number, maxBy: number): boolean { return this.handle.aabb_overlap(minAx, minAy, maxAx, maxAy, minBx, minBy, maxBx, maxBy); }")
    lines.append("  circleOverlap(x1: number, y1: number, r1: number, x2: number, y2: number, r2: number): boolean { return this.handle.circle_overlap(x1, y1, r1, x2, y2, r2); }")
    lines.append("  distance(x1: number, y1: number, x2: number, y2: number): number { return this.handle.distance(x1, y1, x2, y2); }")
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
    lines.append("")

    lines.append("  setViewport(_x: number, _y: number, _width: number, _height: number): void {}")
    lines.append("  enableDepthTest(): void {}")
    lines.append("  disableDepthTest(): void {}")
    lines.append("  clearDepth(): void {}")
    lines.append("  disableBlending(): void {}")
    lines.append("}")
    lines.append("")

    write_generated(GEN / "web" / "index.g.ts", "\n".join(lines))


if __name__ == "__main__":
    print("Generating TypeScript Web SDK...")
    gen_web_input()
    gen_web_wrapper()
    print("TypeScript Web SDK generation complete.")
