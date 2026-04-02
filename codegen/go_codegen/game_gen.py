"""Generator for `goud/game.go` -- Game struct and methods.

Generates the main Game type that wraps GoudEngine FFI calls.
All methods call bridge functions from internal/ffi/helpers.go which
handle C type conversions internally.
"""

from .context import (
    GO_HEADER,
    GO_TYPES,
    OUT,
    schema,
    mapping,
    to_go_field,
    to_go_local,
    to_go_name,
    to_snake,
    write_generated,
)


def _go_return_type(schema_type: str) -> str:
    if schema_type == "void":
        return ""
    # Handle array types like "Entity[]"
    if schema_type.endswith("[]"):
        inner = schema_type[:-2]
        inner_mapped = _go_return_type(inner)
        return f"[]{inner_mapped}"
    nullable = schema_type.endswith("?")
    base = schema_type.rstrip("?")
    if base in GO_TYPES:
        mapped = GO_TYPES[base]
        return f"*{mapped}" if nullable else mapped
    if base == "Entity":
        return "*EntityID" if nullable else "EntityID"
    if base in schema.get("types", {}):
        return f"*{base}" if nullable else base
    # Fallback: use base name as-is
    return f"*{base}" if nullable else base


def _go_param_type(schema_type: str) -> str:
    nullable = schema_type.endswith("?")
    base = schema_type.rstrip("?")
    # Handle array types like "Entity[]", "u8[]"
    if base.endswith("[]"):
        inner = base[:-2]
        inner_mapped = _go_param_type(inner)
        return f"[]{inner_mapped}"
    if base in GO_TYPES:
        mapped = GO_TYPES[base]
        return f"*{mapped}" if nullable else mapped
    if base == "Entity":
        return "*EntityID" if nullable else "EntityID"
    if base in schema.get("enums", {}):
        return f"*{base}" if nullable else base
    if base in schema.get("types", {}):
        return f"*{base}" if nullable else base
    if base == "callback(f32)":
        return "func(float32)"
    # Fallback: use the base name as-is rather than interface{}
    return f"*{base}" if nullable else base


def _go_param_sig(params: list[dict]) -> str:
    parts = []
    for p in params:
        pname = to_go_local(p["name"])
        ptype = _go_param_type(p["type"])
        parts.append(f"{pname} {ptype}")
    return ", ".join(parts)


# Map schema method names to bridge function calls.
# Methods not in this map get generated as stubs.
_BRIDGE_MAP = {
    "shouldClose": "ffi.WindowShouldClose(g.ctx)",
    "close": "ffi.WindowSetShouldClose(g.ctx, true)",
    "setWindowSize": "ffi.WindowSetSize(g.ctx, {width}, {height})",
    "isKeyPressed": "ffi.InputKeyPressed(g.ctx, int32({key}))",
    "isKeyJustPressed": "ffi.InputKeyJustPressed(g.ctx, int32({key}))",
    "isKeyJustReleased": "ffi.InputKeyJustReleased(g.ctx, int32({key}))",
    "isMouseButtonPressed": "ffi.InputMouseButtonPressed(g.ctx, int32({button}))",
    "isMouseButtonJustPressed": "ffi.InputMouseButtonJustPressed(g.ctx, int32({button}))",
    "isMouseButtonJustReleased": "ffi.InputMouseButtonJustReleased(g.ctx, int32({button}))",
    "entityCount": "ffi.EntityCount(g.ctx)",
    "isAlive": "ffi.EntityIsAlive(g.ctx, uint64({entity}))",
    "destroyTexture": "ffi.TextureDestroy(g.ctx, {handle})",
    "destroyFont": "ffi.FontDestroy(g.ctx, {handle})",
    "audioActivate": "ffi.AudioActivate(g.ctx)",
    "audioStop": "ffi.AudioStop(g.ctx, {playerId})",
    "audioStopAll": "ffi.AudioStopAll(g.ctx)",
    "audioPause": "ffi.AudioPause(g.ctx, {playerId})",
    "audioResume": "ffi.AudioResume(g.ctx, {playerId})",
    "audioSetGlobalVolume": "ffi.AudioSetGlobalVolume(g.ctx, {volume})",
    "audioSetPlayerVolume": "ffi.AudioSetPlayerVolume(g.ctx, {playerId}, {volume})",
    "audioIsPlaying": "ffi.AudioIsPlaying(g.ctx, {playerId})",
    "enableDepthTest": "ffi.RendererEnableDepthTest(g.ctx)",
    "disableDepthTest": "ffi.RendererDisableDepthTest(g.ctx)",
    "clearDepth": "ffi.RendererClearDepth(g.ctx)",
    "disableBlending": "ffi.RendererDisableBlending(g.ctx)",
    "enableBlending": "ffi.RendererEnableBlending(g.ctx)",
    "setViewport": "ffi.RendererSetViewport(g.ctx, {x}, {y}, {width}, {height})",
    "mapActionKey": "ffi.InputMapActionKey(g.ctx, {actionName}, int32({key}))",
    "isActionPressed": "ffi.InputActionPressed(g.ctx, {actionName})",
    "isActionJustPressed": "ffi.InputActionJustPressed(g.ctx, {actionName})",
    "isActionJustReleased": "ffi.InputActionJustReleased(g.ctx, {actionName})",
}


def gen_game() -> None:
    tool = schema["tools"]["GoudGame"]
    tool_mapping = mapping["tools"]["GoudGame"]

    lines = [
        GO_HEADER,
        "",
        "package goud",
        "",
        "import (",
        '\t"hash/fnv"',
        '\t"unsafe"',
        "",
        '\tffi "github.com/aram-devdocs/GoudEngine/sdks/go/internal/ffi"',
        ")",
        "",
        "// Ensure unsafe is used (needed for component marshalling).",
        "var _ = unsafe.Sizeof(0)",
        "",
        "// typeIDHash computes a stable hash for component type registration.",
        "func typeIDHash(name string) uint64 {",
        "\th := fnv.New64a()",
        "\th.Write([]byte(name))",
        "\treturn h.Sum64()",
        "}",
        "",
        "var (",
        '\ttypeIDTransform2D = typeIDHash("Transform2D")',
        '\ttypeIDSprite      = typeIDHash("Sprite")',
        ")",
        "",
        "// Game is the main GoudEngine game context.",
        f"// {tool.get('doc', 'Main game context.')}",
        "type Game struct {",
        "\tctx       ffi.ContextBits",
        "\tdeltaTime float32",
        "}",
        "",
        "// NewGame creates a new Game window.",
        "func NewGame(width, height uint32, title string) *Game {",
        "\tctx := ffi.WindowCreate(width, height, title)",
        "\treturn &Game{ctx: ctx}",
        "}",
        "",
        "// Destroy closes the game window and releases resources.",
        "func (g *Game) Destroy() {",
        "\tif g == nil {",
        "\t\treturn",
        "\t}",
        "\tffi.WindowDestroy(g.ctx)",
        "}",
        "",
        "// DeltaTime returns the time elapsed since the last frame in seconds.",
        "func (g *Game) DeltaTime() float32 {",
        "\treturn g.deltaTime",
        "}",
        "",
    ]

    for method in tool.get("methods", []):
        mname = method["name"]
        go_name = to_go_name(mname)
        mmap = tool_mapping["methods"].get(mname, {})
        params = method.get("params", [])
        ret = method.get("returns", "void")

        if mname == "destroy":
            continue

        if mname == "beginFrame":
            _gen_begin_frame(lines)
            continue
        if mname == "endFrame":
            _gen_end_frame(lines)
            continue
        if mname == "run":
            _gen_run_method(go_name, method, lines)
            continue
        if mname == "updateFrame":
            _gen_update_frame(lines)
            continue

        # Special hand-coded methods
        if mname == "loadTexture":
            _gen_load_texture(lines)
            continue
        if mname == "loadFont":
            _gen_load_font(lines)
            continue
        if mname == "drawSprite":
            _gen_draw_sprite(lines)
            continue
        if mname == "drawQuad":
            _gen_draw_quad(lines)
            continue
        if mname == "drawText":
            _gen_draw_text(lines)
            continue
        if mname == "drawSpriteRect":
            _gen_draw_sprite_rect(lines)
            continue
        if mname == "spawnEmpty":
            _gen_spawn_empty(lines)
            continue
        if mname == "despawn":
            _gen_despawn(lines)
            continue
        if mname == "cloneEntity":
            _gen_clone_entity(lines)
            continue
        if mname == "cloneEntityRecursive":
            _gen_clone_entity_recursive(lines)
            continue
        if mname == "audioPlay":
            _gen_audio_play(lines)
            continue
        if mname == "audioPlayOnChannel":
            _gen_audio_play_on_channel(lines)
            continue

        # Mouse/scroll position methods
        if mname == "getMousePosition":
            _gen_get_mouse_position(lines)
            continue
        if mname == "getMouseDelta":
            _gen_get_mouse_delta(lines)
            continue
        if mname == "getScrollDelta":
            _gen_get_scroll_delta(lines)
            continue

        # Window size getters
        if mname == "getWindowSize":
            _gen_get_window_size(lines)
            continue

        # Input action methods
        if mname == "mapActionKey":
            _gen_map_action_key(lines)
            continue
        if mname == "isActionPressed":
            _gen_action_method("IsActionPressed", "InputActionPressed", lines)
            continue
        if mname == "isActionJustPressed":
            _gen_action_method("IsActionJustPressed", "InputActionJustPressed", lines)
            continue
        if mname == "isActionJustReleased":
            _gen_action_method("IsActionJustReleased", "InputActionJustReleased", lines)
            continue

        # Component strategy methods
        if "ffi_strategy" in mmap:
            _gen_strategy_method(go_name, method, mmap, params, ret, lines)
            continue

        # Simple bridge-mapped methods
        if mname in _BRIDGE_MAP:
            _gen_bridge_method(go_name, method, mname, params, ret, lines)
            continue

        # Everything else is a stub
        _gen_stub_method(go_name, method, params, ret, lines)

    # lastError helper
    lines.extend([
        "// lastError queries FFI error state and returns a Go error, or nil.",
        "func (g *Game) lastError() error {",
        "\tcode := ffi.LastErrorCode()",
        "\tif code == 0 {",
        "\t\treturn nil",
        "\t}",
        "\tmsg := ffi.LastErrorMessage()",
        "\tsubsys := ffi.LastErrorSubsystem()",
        "\top := ffi.LastErrorOperation()",
        "\trecovery := RecoveryClass(ffi.ErrorRecoveryClass(code))",
        "\treturn &GoudError{",
        "\t\tCode:      code,",
        "\t\tMessage:   msg,",
        "\t\tCategory:  categoryFromCode(code),",
        "\t\tSubsystem: subsys,",
        "\t\tOperation: op,",
        "\t\tRecovery:  recovery,",
        "\t}",
        "}",
        "",
    ])

    write_generated(OUT / "game.go", "\n".join(lines))


# ---- Hand-coded methods ----


def _gen_begin_frame(lines: list[str]) -> None:
    lines.extend([
        "// BeginFrame starts a new frame: polls events, clears the screen, and begins rendering.",
        "func (g *Game) BeginFrame(r, gr, b, a float32) {",
        "\tg.deltaTime = ffi.WindowPollEvents(g.ctx)",
        "\tffi.WindowClear(g.ctx, r, gr, b, a)",
        "\tffi.RendererBegin(g.ctx)",
        "\tffi.RendererEnableBlending(g.ctx)",
        "}",
        "",
    ])


def _gen_end_frame(lines: list[str]) -> None:
    lines.extend([
        "// EndFrame finishes rendering and swaps buffers.",
        "func (g *Game) EndFrame() {",
        "\tffi.RendererEnd(g.ctx)",
        "\tffi.WindowSwapBuffers(g.ctx)",
        "}",
        "",
    ])


def _gen_run_method(go_name: str, method: dict, lines: list[str]) -> None:
    doc = method.get("doc", "Run the game loop.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(update func(float32)) {{",
        "\tfor !ffi.WindowShouldClose(g.ctx) {",
        "\t\tg.BeginFrame(0, 0, 0, 1)",
        "\t\tupdate(g.deltaTime)",
        "\t\tg.EndFrame()",
        "\t}",
        "}",
        "",
    ])


def _gen_update_frame(lines: list[str]) -> None:
    lines.extend([
        "// UpdateFrame updates internal frame state with a given delta time.",
        "func (g *Game) UpdateFrame(dt float32) {",
        "\tg.deltaTime = dt",
        "}",
        "",
    ])


def _gen_load_texture(lines: list[str]) -> None:
    lines.extend([
        "// LoadTexture loads a texture from a file path and returns its handle.",
        "func (g *Game) LoadTexture(path string) uint64 {",
        "\treturn ffi.TextureLoad(g.ctx, path)",
        "}",
        "",
    ])


def _gen_load_font(lines: list[str]) -> None:
    lines.extend([
        "// LoadFont loads a font from a file path and returns its handle.",
        "func (g *Game) LoadFont(path string) uint64 {",
        "\treturn ffi.FontLoad(g.ctx, path)",
        "}",
        "",
    ])


def _gen_draw_sprite(lines: list[str]) -> None:
    lines.extend([
        "// DrawSprite draws a textured sprite.",
        "func (g *Game) DrawSprite(texture uint64, x, y, width, height, rotation float32, color Color) {",
        "\tffi.RendererDrawSprite(g.ctx, texture, x, y, width, height, rotation, color.R, color.G, color.B, color.A)",
        "}",
        "",
    ])


def _gen_draw_sprite_rect(lines: list[str]) -> None:
    lines.extend([
        "// DrawSpriteRect draws a sprite with a source rectangle.",
        "// srcMode: 0 = normalized UVs (0.0-1.0), 1 = pixel coordinates (default).",
        "func (g *Game) DrawSpriteRect(texture uint64, x, y, width, height, rotation, srcX, srcY, srcW, srcH float32, color Color, srcMode ...uint32) bool {",
        "\tmode := uint32(1)",
        "\tif len(srcMode) > 0 {",
        "\t\tmode = srcMode[0]",
        "\t}",
        "\treturn ffi.RendererDrawSpriteRect(g.ctx, texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, mode, color.R, color.G, color.B, color.A)",
        "}",
        "",
    ])


def _gen_draw_quad(lines: list[str]) -> None:
    lines.extend([
        "// DrawQuad draws a colored rectangle.",
        "func (g *Game) DrawQuad(x, y, width, height float32, color Color) {",
        "\tffi.RendererDrawQuad(g.ctx, x, y, width, height, color.R, color.G, color.B, color.A)",
        "}",
        "",
    ])


def _gen_draw_text(lines: list[str]) -> None:
    lines.extend([
        "// DrawText draws text with a font.",
        "func (g *Game) DrawText(fontHandle uint64, text string, x, y, fontSize float32, alignment TextAlignment, maxWidth, lineSpacing float32, direction TextDirection, color Color) bool {",
        "\treturn ffi.RendererDrawText(g.ctx, fontHandle, text, x, y, fontSize, uint8(alignment), maxWidth, lineSpacing, uint8(direction), color.R, color.G, color.B, color.A)",
        "}",
        "",
    ])


def _gen_spawn_empty(lines: list[str]) -> None:
    lines.extend([
        "// SpawnEmpty creates a new empty entity.",
        "func (g *Game) SpawnEmpty() EntityID {",
        "\treturn NewEntityID(ffi.EntitySpawnEmpty(g.ctx))",
        "}",
        "",
    ])


def _gen_despawn(lines: list[str]) -> None:
    lines.extend([
        "// Despawn destroys an entity and all its components.",
        "func (g *Game) Despawn(entity EntityID) bool {",
        "\treturn ffi.EntityDespawn(g.ctx, uint64(entity)) == 0",
        "}",
        "",
    ])


def _gen_clone_entity(lines: list[str]) -> None:
    lines.extend([
        "// CloneEntity clones an entity.",
        "func (g *Game) CloneEntity(entity EntityID) EntityID {",
        "\treturn NewEntityID(ffi.EntityClone(g.ctx, uint64(entity)))",
        "}",
        "",
    ])


def _gen_clone_entity_recursive(lines: list[str]) -> None:
    lines.extend([
        "// CloneEntityRecursive clones an entity and its children.",
        "func (g *Game) CloneEntityRecursive(entity EntityID) EntityID {",
        "\treturn NewEntityID(ffi.EntityCloneRecursive(g.ctx, uint64(entity)))",
        "}",
        "",
    ])


def _gen_audio_play(lines: list[str]) -> None:
    lines.extend([
        "// AudioPlay plays audio from raw bytes and returns a player ID.",
        "func (g *Game) AudioPlay(data []byte) int64 {",
        "\treturn ffi.AudioPlay(g.ctx, data)",
        "}",
        "",
    ])


def _gen_audio_play_on_channel(lines: list[str]) -> None:
    lines.extend([
        "// AudioPlayOnChannel plays audio on a specific channel.",
        "func (g *Game) AudioPlayOnChannel(data []byte, channel uint8) int64 {",
        "\treturn ffi.AudioPlayOnChannel(g.ctx, data, channel)",
        "}",
        "",
    ])


def _gen_get_mouse_position(lines: list[str]) -> None:
    lines.extend([
        "// GetMousePosition returns the current mouse position.",
        "func (g *Game) GetMousePosition() Vec2 {",
        "\tx, y := ffi.InputGetMousePosition(g.ctx)",
        "\treturn NewVec2(x, y)",
        "}",
        "",
    ])


def _gen_get_mouse_delta(lines: list[str]) -> None:
    lines.extend([
        "// GetMouseDelta returns the mouse movement delta.",
        "func (g *Game) GetMouseDelta() Vec2 {",
        "\tdx, dy := ffi.InputGetMouseDelta(g.ctx)",
        "\treturn NewVec2(dx, dy)",
        "}",
        "",
    ])


def _gen_get_scroll_delta(lines: list[str]) -> None:
    lines.extend([
        "// GetScrollDelta returns the scroll delta.",
        "func (g *Game) GetScrollDelta() Vec2 {",
        "\tdx, dy := ffi.InputGetScrollDelta(g.ctx)",
        "\treturn NewVec2(dx, dy)",
        "}",
        "",
    ])


def _gen_get_window_size(lines: list[str]) -> None:
    lines.extend([
        "// GetWindowSize returns the window width and height.",
        "func (g *Game) GetWindowSize() (uint32, uint32) {",
        "\treturn ffi.WindowGetSize(g.ctx)",
        "}",
        "",
    ])


def _gen_map_action_key(lines: list[str]) -> None:
    lines.extend([
        "// MapActionKey maps an action name to a key code.",
        "func (g *Game) MapActionKey(actionName string, key Key) bool {",
        "\treturn ffi.InputMapActionKey(g.ctx, actionName, int32(key))",
        "}",
        "",
    ])


def _gen_action_method(go_name: str, bridge_fn: str, lines: list[str]) -> None:
    lines.extend([
        f"// {go_name} returns true if a named action meets the condition.",
        f"func (g *Game) {go_name}(actionName string) bool {{",
        f"\treturn ffi.{bridge_fn}(g.ctx, actionName)",
        "}",
        "",
    ])


def _gen_strategy_method(
    go_name: str, method: dict, mmap: dict, params: list[dict], ret: str, lines: list[str]
) -> None:
    strategy = mmap["ffi_strategy"]
    comp_type = mmap.get("component_type", "")

    if strategy == "component_add" and comp_type == "Transform2D":
        _gen_add_transform2d(go_name, method, mmap, params, lines)
    elif strategy == "component_get" and comp_type == "Transform2D":
        _gen_get_transform2d(go_name, method, mmap, params, lines)
    elif strategy == "component_set" and comp_type == "Transform2D":
        _gen_set_transform2d(go_name, method, mmap, params, lines)
    elif strategy == "component_has" and comp_type in ("Transform2D", "Sprite"):
        _gen_has_component(go_name, method, mmap, params, comp_type, lines)
    elif strategy == "component_remove" and comp_type in ("Transform2D", "Sprite"):
        _gen_remove_component(go_name, method, mmap, params, comp_type, lines)
    elif strategy == "component_add" and comp_type == "Sprite":
        _gen_add_sprite(go_name, method, mmap, params, lines)
    elif strategy == "component_get" and comp_type == "Sprite":
        _gen_get_sprite(go_name, method, mmap, params, lines)
    elif strategy == "component_set" and comp_type == "Sprite":
        _gen_set_sprite(go_name, method, mmap, params, lines)
    else:
        _gen_stub_method(go_name, method, params, ret, lines)


def _gen_add_transform2d(go_name, method, mmap, params, lines):
    doc = method.get("doc", "Adds a Transform2D component to an entity.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID, transform Transform2D) {{",
        "\tdata := ffi.Transform2DToBytes(transform.PositionX, transform.PositionY, transform.Rotation, transform.ScaleX, transform.ScaleY)",
        "\tffi.ComponentAdd(g.ctx, uint64(entity), typeIDTransform2D, data)",
        "}",
        "",
    ])


def _gen_get_transform2d(go_name, method, mmap, params, lines):
    doc = method.get("doc", "Gets the Transform2D component of an entity.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID) *Transform2D {{",
        "\tptr := ffi.ComponentGet(g.ctx, uint64(entity), typeIDTransform2D)",
        "\tif ptr == nil {",
        "\t\treturn nil",
        "\t}",
        "\tpx, py, rot, sx, sy := ffi.Transform2DFromPtr(ptr)",
        "\tt := &Transform2D{",
        "\t\tPositionX: px,",
        "\t\tPositionY: py,",
        "\t\tRotation:  rot,",
        "\t\tScaleX:    sx,",
        "\t\tScaleY:    sy,",
        "\t}",
        "\treturn t",
        "}",
        "",
    ])


def _gen_set_transform2d(go_name, method, mmap, params, lines):
    doc = method.get("doc", "Sets the Transform2D component of an entity.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID, transform Transform2D) {{",
        "\tdata := ffi.Transform2DToBytes(transform.PositionX, transform.PositionY, transform.Rotation, transform.ScaleX, transform.ScaleY)",
        "\tptr := ffi.ComponentGetMut(g.ctx, uint64(entity), typeIDTransform2D)",
        "\tif ptr != nil && len(data) > 0 {",
        "\t\t// SAFETY: ptr is a valid mutable pointer to component data of at least len(data) bytes.",
        "\t\tdst := unsafe.Slice((*byte)(ptr), len(data))",
        "\t\tcopy(dst, data)",
        "\t}",
        "}",
        "",
    ])


def _gen_add_sprite(go_name, method, mmap, params, lines):
    doc = method.get("doc", "Adds a Sprite component to an entity.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID, sprite Sprite) {{",
        "\tdata := ffi.SpriteToBytes(",
        "\t\tsprite.TextureHandle, sprite.ColorR, sprite.ColorG, sprite.ColorB, sprite.ColorA,",
        "\t\tsprite.SourceRectX, sprite.SourceRectY, sprite.SourceRectWidth, sprite.SourceRectHeight,",
        "\t\tsprite.HasSourceRect, sprite.FlipX, sprite.FlipY, sprite.ZLayer,",
        "\t\tsprite.AnchorX, sprite.AnchorY, sprite.CustomSizeX, sprite.CustomSizeY,",
        "\t\tsprite.HasCustomSize,",
        "\t)",
        "\tffi.ComponentAdd(g.ctx, uint64(entity), typeIDSprite, data)",
        "}",
        "",
    ])


def _gen_get_sprite(go_name, method, mmap, params, lines):
    doc = method.get("doc", "Gets the Sprite component of an entity.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID) *Sprite {{",
        "\tptr := ffi.ComponentGet(g.ctx, uint64(entity), typeIDSprite)",
        "\tif ptr == nil {",
        "\t\treturn nil",
        "\t}",
        "\tth, cr, cg, cb, ca, srx, sry, srw, srh, hsr, fx, fy, zl, ax, ay, csx, csy, hcs := ffi.SpriteFromPtr(ptr)",
        "\treturn &Sprite{",
        "\t\tTextureHandle:    th,",
        "\t\tColorR: cr, ColorG: cg, ColorB: cb, ColorA: ca,",
        "\t\tSourceRectX: srx, SourceRectY: sry, SourceRectWidth: srw, SourceRectHeight: srh,",
        "\t\tHasSourceRect: hsr,",
        "\t\tFlipX: fx, FlipY: fy, ZLayer: zl,",
        "\t\tAnchorX: ax, AnchorY: ay,",
        "\t\tCustomSizeX: csx, CustomSizeY: csy,",
        "\t\tHasCustomSize: hcs,",
        "\t}",
        "}",
        "",
    ])


def _gen_set_sprite(go_name, method, mmap, params, lines):
    doc = method.get("doc", "Sets the Sprite component of an entity.")
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID, sprite Sprite) {{",
        "\tdata := ffi.SpriteToBytes(",
        "\t\tsprite.TextureHandle, sprite.ColorR, sprite.ColorG, sprite.ColorB, sprite.ColorA,",
        "\t\tsprite.SourceRectX, sprite.SourceRectY, sprite.SourceRectWidth, sprite.SourceRectHeight,",
        "\t\tsprite.HasSourceRect, sprite.FlipX, sprite.FlipY, sprite.ZLayer,",
        "\t\tsprite.AnchorX, sprite.AnchorY, sprite.CustomSizeX, sprite.CustomSizeY,",
        "\t\tsprite.HasCustomSize,",
        "\t)",
        "\tptr := ffi.ComponentGetMut(g.ctx, uint64(entity), typeIDSprite)",
        "\tif ptr != nil && len(data) > 0 {",
        "\t\t// SAFETY: ptr is a valid mutable pointer to component data of at least len(data) bytes.",
        "\t\tdst := unsafe.Slice((*byte)(ptr), len(data))",
        "\t\tcopy(dst, data)",
        "\t}",
        "}",
        "",
    ])


def _gen_has_component(go_name, method, mmap, params, comp_type, lines):
    doc = method.get("doc", f"Returns true if the entity has a {comp_type} component.")
    type_id_var = f"typeID{comp_type}"
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID) bool {{",
        f"\treturn ffi.ComponentHas(g.ctx, uint64(entity), {type_id_var})",
        "}",
        "",
    ])


def _gen_remove_component(go_name, method, mmap, params, comp_type, lines):
    doc = method.get("doc", f"Removes the {comp_type} component from an entity.")
    type_id_var = f"typeID{comp_type}"
    lines.extend([
        f"// {go_name} {doc}",
        f"func (g *Game) {go_name}(entity EntityID) bool {{",
        f"\treturn ffi.ComponentRemove(g.ctx, uint64(entity), {type_id_var})",
        "}",
        "",
    ])


def _gen_bridge_method(
    go_name: str, method: dict, mname: str, params: list[dict], ret: str, lines: list[str]
) -> None:
    doc = method.get("doc", f"{go_name}.")
    go_ret = _go_return_type(ret)
    sig = _go_param_sig(params)
    bridge_call = _BRIDGE_MAP[mname]

    # Substitute parameter names into the bridge call
    for p in params:
        pname = to_go_local(p["name"])
        bridge_call = bridge_call.replace(f"{{{p['name']}}}", pname)

    lines.append(f"// {go_name} {doc}")
    if go_ret:
        lines.append(f"func (g *Game) {go_name}({sig}) {go_ret} {{")
        lines.append(f"\treturn {bridge_call}")
    else:
        lines.append(f"func (g *Game) {go_name}({sig}) {{")
        lines.append(f"\t{bridge_call}")
    lines.append("}")
    lines.append("")


def _gen_stub_method(
    go_name: str, method: dict, params: list[dict], ret: str, lines: list[str]
) -> None:
    doc = method.get("doc", f"{go_name} (not yet implemented).")
    go_ret = _go_return_type(ret)
    sig = _go_param_sig(params)

    # Filter out callback params from signature
    filtered_params = [p for p in params if p.get("type") != "callback(f32)"]
    sig = _go_param_sig(filtered_params)

    lines.append(f"// {go_name} {doc}")
    if go_ret:
        lines.append(f"func (g *Game) {go_name}({sig}) {go_ret} {{")
        if go_ret.startswith("*"):
            lines.append("\treturn nil")
        elif go_ret == "bool":
            lines.append("\treturn false")
        elif go_ret == "string":
            lines.append('\treturn ""')
        elif go_ret in ("int32", "int64", "uint32", "uint64", "float32", "float64",
                        "uint8", "uint16", "int8", "int16", "uintptr", "uint", "int"):
            lines.append("\treturn 0")
        elif go_ret == "EntityID":
            lines.append("\treturn 0")
        elif go_ret == "[]byte":
            lines.append("\treturn nil")
        else:
            lines.append(f"\treturn {go_ret}{{}}")
    else:
        lines.append(f"func (g *Game) {go_name}({sig}) {{")
    lines.append("}")
    lines.append("")
