"""Component body generation helpers for tool wrappers."""

from sdk_common import to_pascal, to_snake
from .context import mapping, _FFI_TO_SDK_FIELDS
from .helpers import _type_hash

def _gen_component_body(mm: dict, ret: str, L: list):
    """Generate method body for component FFI strategies in GoudGame/GoudContext."""
    strategy = mm["ffi_strategy"]
    comp_type = mm.get("component_type", "")
    ffi_name = mapping["ffi_types"].get(comp_type, {}).get("ffi_name", "")
    # Get the actual parameter name from struct_params (e.g., "transform", "sprite")
    struct_params = mm.get("struct_params", [])
    param_name = struct_params[0] if struct_params else to_snake(comp_type)

    if strategy == "component_add":
        L.append("            unsafe")
        L.append("            {")
        L.append(f"                var ffi = {param_name}._inner;")
        L.append("                byte* ptr = (byte*)&ffi;")
        L.append("                NativeMethods.goud_component_add(")
        L.append("                    _ctx, entity.ToBits(),")
        L.append(f"                    {_type_hash(comp_type)},")
        L.append(f"                    (IntPtr)ptr, (nuint)sizeof({ffi_name}));")
        L.append("            }")
    elif strategy == "component_set":
        L.append("            unsafe")
        L.append("            {")
        L.append(f"                var ffi = {param_name}._inner;")
        L.append("                byte* ptr = (byte*)&ffi;")
        L.append("                NativeMethods.goud_component_add(")
        L.append("                    _ctx, entity.ToBits(),")
        L.append(f"                    {_type_hash(comp_type)},")
        L.append(f"                    (IntPtr)ptr, (nuint)sizeof({ffi_name}));")
        L.append("            }")
    elif strategy == "component_get":
        fields = _FFI_TO_SDK_FIELDS.get(ffi_name, [])
        L.append("            unsafe")
        L.append("            {")
        L.append("                IntPtr ptr = NativeMethods.goud_component_get(")
        L.append("                    _ctx, entity.ToBits(),")
        L.append(f"                    {_type_hash(comp_type)});")
        L.append("                if (ptr == IntPtr.Zero) return null;")
        L.append(f"                var ffi = *({ffi_name}*)ptr;")
        if fields:
            field_refs = ", ".join(f"ffi.{f}" for f in fields)
            L.append(f"                return new {comp_type}({field_refs});")
        else:
            # Use internal constructor (e.g., Sprite)
            L.append(f"                return new {comp_type}(ffi);")
        L.append("            }")
    elif strategy == "component_has":
        L.append("            return NativeMethods.goud_component_has(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash(comp_type)});")
    elif strategy == "component_remove":
        L.append("            return NativeMethods.goud_component_remove(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash(comp_type)}).Success;")
    elif strategy == "name_add":
        L.append('            unsafe')
        L.append('            {')
        L.append('                var bytes = System.Text.Encoding.UTF8.GetBytes(name);')
        L.append('                fixed (byte* ptr = bytes)')
        L.append('                {')
        L.append('                    NativeMethods.goud_component_add(')
        L.append('                        _ctx, entity.ToBits(),')
        L.append(f'                        {_type_hash("Name")},')
        L.append('                        (IntPtr)ptr, (nuint)bytes.Length);')
        L.append('                }')
        L.append('            }')
    elif strategy == "name_get":
        L.append('            unsafe')
        L.append('            {')
        L.append('                IntPtr ptr = NativeMethods.goud_component_get(')
        L.append('                    _ctx, entity.ToBits(),')
        L.append(f'                    {_type_hash("Name")});')
        L.append('                if (ptr == IntPtr.Zero) return null;')
        L.append('                return System.Runtime.InteropServices.Marshal.PtrToStringUTF8(ptr);')
        L.append('            }')
    elif strategy == "name_has":
        L.append("            return NativeMethods.goud_component_has(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash('Name')});")
    elif strategy == "name_remove":
        L.append("            return NativeMethods.goud_component_remove(")
        L.append("                _ctx, entity.ToBits(),")
        L.append(f"                {_type_hash('Name')}).Success;")
    else:
        L.append(f'            throw new System.NotImplementedException("Unknown strategy: {strategy}");')


_I = "            "  # 12-space indent shorthand for body lines
_WINDOWED_BODIES: dict = {
    "BeginFrame": [
        f"{_I}_deltaTime = NativeMethods.goud_window_poll_events(_ctx);",
        f"{_I}NativeMethods.goud_window_clear(_ctx, r, g, b, a);",
        f"{_I}NativeMethods.goud_renderer_begin(_ctx);",
        f"{_I}NativeMethods.goud_renderer_enable_blending(_ctx);",
    ],
    "EndFrame": [f"{_I}NativeMethods.goud_renderer_end(_ctx);", f"{_I}NativeMethods.goud_window_swap_buffers(_ctx);"],
    "Run":      [f"{_I}while (!ShouldClose())", f"{_I}{{", f"{_I}    BeginFrame();",
                 f"{_I}    update(_deltaTime);", f"{_I}    EndFrame();", f"{_I}}}"],
    "RunWithFixedUpdate": [
        f"{_I}while (!ShouldClose())",
        f"{_I}{{",
        f"{_I}    BeginFrame();",
        f"{_I}    if (NativeMethods.goud_fixed_timestep_begin(_ctx))",
        f"{_I}    {{",
        f"{_I}        while (NativeMethods.goud_fixed_timestep_step(_ctx))",
        f"{_I}            fixedUpdate(NativeMethods.goud_fixed_timestep_dt(_ctx));",
        f"{_I}    }}",
        f"{_I}    update(_deltaTime);",
        f"{_I}    EndFrame();",
        f"{_I}}}",
    ],
    "SetFixedTimestep":  [f"{_I}NativeMethods.goud_fixed_timestep_set(_ctx, stepSize);"],
    "SetMaxFixedSteps":  [f"{_I}NativeMethods.goud_fixed_timestep_set_max_steps(_ctx, maxSteps);"],
    "DrawSprite":     [f"{_I}var c = color ?? Color.White();",
                       f"{_I}NativeMethods.goud_renderer_draw_sprite(_ctx, texture, x, y, width, height, rotation, c.R, c.G, c.B, c.A);"],
    "DrawSpriteRect": [f"{_I}var c = color ?? Color.White();",
                       f"{_I}return NativeMethods.goud_renderer_draw_sprite_rect(_ctx, texture, x, y, width, height, rotation, srcX, srcY, srcW, srcH, (uint)srcMode, c.R, c.G, c.B, c.A);"],
    "DrawSpriteBatch": [
        f"{_I}if (cmds == null || cmds.Length == 0) return 0;",
        f"{_I}const int StackAllocThreshold = 512;",
        f"{_I}FfiSpriteCmd[] heapBuf = cmds.Length > StackAllocThreshold ? new FfiSpriteCmd[cmds.Length] : null;",
        f"{_I}Span<FfiSpriteCmd> ffi = heapBuf != null ? heapBuf.AsSpan() : stackalloc FfiSpriteCmd[cmds.Length];",
        f"{_I}for (int i = 0; i < cmds.Length; i++)",
        f"{_I}{{",
        f"{_I}    ref readonly var s = ref cmds[i];",
        f"{_I}    var c = s.Color ?? Color.White();",
        f"{_I}    ffi[i] = new FfiSpriteCmd {{ Texture = s.Texture, X = s.X, Y = s.Y, Width = s.Width, Height = s.Height, Rotation = s.Rotation, SrcX = s.SrcX, SrcY = s.SrcY, SrcW = s.SrcW, SrcH = s.SrcH, R = c.R, G = c.G, B = c.B, A = c.A, ZLayer = s.ZLayer, _Padding = 0 }};",
        f"{_I}}}",
        f"{_I}fixed (FfiSpriteCmd* ptr = ffi)",
        f"{_I}{{",
        f"{_I}    return NativeMethods.goud_renderer_draw_sprite_batch(_ctx, ptr, (uint)cmds.Length);",
        f"{_I}}}",
    ],
    "DrawTextBatch": [
        f"{_I}if (cmds == null || cmds.Length == 0) return 0;",
        f"{_I}var handles = new System.Collections.Generic.List<GCHandle>(cmds.Length);",
        f"{_I}try",
        f"{_I}{{",
        f"{_I}    var ffi = new FfiTextCmd[cmds.Length];",
        f"{_I}    for (int i = 0; i < cmds.Length; i++)",
        f"{_I}    {{",
        f"{_I}        ref readonly var t = ref cmds[i];",
        f"{_I}        var c = t.Color ?? Color.White();",
        f"{_I}        byte[] bytes = System.Text.Encoding.UTF8.GetBytes(t.Text + '\\0');",
        f"{_I}        var h = GCHandle.Alloc(bytes, GCHandleType.Pinned);",
        f"{_I}        handles.Add(h);",
        f"{_I}        ffi[i] = new FfiTextCmd {{ FontHandle = t.FontHandle, Text = h.AddrOfPinnedObject(), X = t.X, Y = t.Y, FontSize = t.FontSize, Alignment = (byte)t.Alignment, Direction = (byte)t.Direction, _Pad0 = 0, MaxWidth = t.MaxWidth, LineSpacing = t.LineSpacing > 0 ? t.LineSpacing : 1.0f, R = c.R, G = c.G, B = c.B, A = c.A }};",
        f"{_I}    }}",
        f"{_I}    fixed (FfiTextCmd* ptr = ffi)",
        f"{_I}    {{",
        f"{_I}        return NativeMethods.goud_renderer_draw_text_batch(_ctx, ptr, (uint)cmds.Length);",
        f"{_I}    }}",
        f"{_I}}}",
        f"{_I}finally",
        f"{_I}{{",
        f"{_I}    foreach (var h in handles) h.Free();",
        f"{_I}}}",
    ],
    "DrawQuad":       [f"{_I}var c = color ?? Color.White();",
                       f"{_I}NativeMethods.goud_renderer_draw_quad(_ctx, x, y, width, height, c.R, c.G, c.B, c.A);"],
    "LoadTexture":    [f"{_I}return NativeMethods.goud_texture_load(_ctx, path);"],
    "DestroyTexture": [f"{_I}NativeMethods.goud_texture_destroy(_ctx, handle);"],
    "LoadFont":       [f"{_I}return NativeMethods.goud_font_load(_ctx, path);"],
    "DestroyFont":    [f"{_I}return NativeMethods.goud_font_destroy(_ctx, handle);"],
    "DrawText":       [f"{_I}var c = color ?? Color.White();",
                       f"{_I}return NativeMethods.goud_renderer_draw_text(_ctx, fontHandle, text, x, y, fontSize, (byte)alignment, maxWidth, lineSpacing, (byte)direction, c.R, c.G, c.B, c.A);"],
    "Close":          [f"{_I}NativeMethods.goud_window_set_should_close(_ctx, true);"],
    "ShouldClose":    [f"{_I}return NativeMethods.goud_window_should_close(_ctx);"],
    "UpdateFrame":    [f"{_I}_deltaTime = (float)dt;",
                       f"{_I}_frameCount++;",
                       f"{_I}_totalTime += dt;"],
}

