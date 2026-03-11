"""Physics world, engine config, and UI manager generation."""

from sdk_common import HEADER_COMMENT, write_generated
from .context import NS, OUT, mapping, schema
from .helpers import _ffi_fn_def

def gen_physics_world_2d():
    if "PhysicsWorld2D" not in schema.get("tools", {}) or "PhysicsWorld2D" not in mapping.get("tools", {}):
        return
    tool = schema["tools"]["PhysicsWorld2D"]
    tm = mapping["tools"]["PhysicsWorld2D"]
    ctor_params = tool.get("constructor", {}).get("params", [])
    ctor_sig = ", ".join(_safe_param_strs(ctor_params + [{"name": "backend", "type": "PhysicsBackend2D", "default": 0}]))

    lines = [
        f"// {HEADER_COMMENT}",
        "using System;", "",
        f"namespace {NS}", "{",
        f"    /// <summary>{tool.get('doc', '2D physics simulation world')}</summary>",
        "    public class PhysicsWorld2D : IDisposable", "    {",
        "        private GoudContextId _ctx;",
        "        private bool _disposed;", "",
        f"        public PhysicsWorld2D({ctor_sig})",
        "        {",
        "            _ctx = NativeMethods.goud_context_create();",
        "            if (!_ctx.IsValid) throw new Exception(\"Failed to create headless context\");",
        "            int status = NativeMethods.goud_physics_create_with_backend(_ctx, gravityX, gravityY, (uint)backend);",
        "            if (status != 0)",
        "            {",
        "                NativeMethods.goud_context_destroy(_ctx);",
        "                _ctx = GoudContextId.Invalid;",
        "                throw new Exception($\"Failed to create PhysicsWorld2D (status {status})\");",
        "            }",
        "        }", "",
    ]

    for method in tool.get("methods", []):
        mn = to_pascal(method["name"])
        mm = tm.get("methods", {}).get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")
        cs_ret = cs_type(ret.rstrip("[]").rstrip("?"))
        actual_ret = cs_ret if not ret.endswith("[]") else f"{cs_ret}[]"

        sig = ", ".join(_safe_param_strs(params))
        if method.get("doc"):
            lines.append(f"        /// <summary>{method['doc']}</summary>")

        if method["name"] == "destroy":
            lines += [
                f"        public {actual_ret} {mn}({sig})",
                "        {",
                "            if (_disposed) return 0;",
                "            int status = NativeMethods.goud_physics_destroy(_ctx);",
                "            NativeMethods.goud_context_destroy(_ctx);",
                "            _disposed = true;",
                "            return status;",
                "        }", "",
            ]
            continue

        lines += [f"        public {actual_ret} {mn}({sig})", "        {"]
        _gen_method_body(mn, mm, params, ret, lines, False)
        lines += ["        }", ""]

    lines += [
        "        public void Dispose()",
        "        {",
        "            if (!_disposed) Destroy();",
        "        }",
        "    }",
        "}",
        "",
    ]
    write_generated(OUT / "Core" / "PhysicsWorld2D.g.cs", "\n".join(lines))


def gen_physics_world_3d():
    if "PhysicsWorld3D" not in schema.get("tools", {}) or "PhysicsWorld3D" not in mapping.get("tools", {}):
        return
    tool = schema["tools"]["PhysicsWorld3D"]
    tm = mapping["tools"]["PhysicsWorld3D"]
    ctor_params = tool.get("constructor", {}).get("params", [])
    ctor_sig = ", ".join(_safe_param_strs(ctor_params))

    lines = [
        f"// {HEADER_COMMENT}",
        "using System;", "",
        f"namespace {NS}", "{",
        f"    /// <summary>{tool.get('doc', '3D physics simulation world')}</summary>",
        "    public class PhysicsWorld3D : IDisposable", "    {",
        "        private GoudContextId _ctx;",
        "        private bool _disposed;", "",
        f"        public PhysicsWorld3D({ctor_sig})",
        "        {",
        "            _ctx = NativeMethods.goud_context_create();",
        "            if (!_ctx.IsValid) throw new Exception(\"Failed to create headless context\");",
        "            int status = NativeMethods.goud_physics3d_create(_ctx, gravityX, gravityY, gravityZ);",
        "            if (status != 0)",
        "            {",
        "                NativeMethods.goud_context_destroy(_ctx);",
        "                _ctx = GoudContextId.Invalid;",
        "                throw new Exception($\"Failed to create PhysicsWorld3D (status {status})\");",
        "            }",
        "        }", "",
    ]

    for method in tool.get("methods", []):
        mn = to_pascal(method["name"])
        mm = tm.get("methods", {}).get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")
        cs_ret = cs_type(ret.rstrip("[]").rstrip("?"))
        actual_ret = cs_ret if not ret.endswith("[]") else f"{cs_ret}[]"

        sig = ", ".join(_safe_param_strs(params))
        if method.get("doc"):
            lines.append(f"        /// <summary>{method['doc']}</summary>")

        if method["name"] == "destroy":
            lines += [
                f"        public {actual_ret} {mn}({sig})",
                "        {",
                "            if (_disposed) return 0;",
                "            int status = NativeMethods.goud_physics3d_destroy(_ctx);",
                "            NativeMethods.goud_context_destroy(_ctx);",
                "            _disposed = true;",
                "            return status;",
                "        }", "",
            ]
            continue

        lines += [f"        public {actual_ret} {mn}({sig})", "        {"]
        _gen_method_body(mn, mm, params, ret, lines, False)
        lines += ["        }", ""]

    lines += [
        "        public void Dispose()",
        "        {",
        "            if (!_disposed) Destroy();",
        "        }",
        "    }",
        "}",
        "",
    ]
    write_generated(OUT / "Core" / "PhysicsWorld3D.g.cs", "\n".join(lines))


def gen_engine_config():
    """Generate EngineConfig builder class for C#."""
    tool = schema["tools"]["EngineConfig"]
    tm = mapping["tools"]["EngineConfig"]

    lines = [
        f"// {HEADER_COMMENT}",
        "using System;", "using System.Runtime.InteropServices;", "",
        f"namespace {NS}", "{",
        f"    /// <summary>{tool.get('doc', 'EngineConfig')}</summary>",
        "    public class EngineConfig : IDisposable", "    {",
        "        private IntPtr _handle;",
        '        private string _title = "GoudEngine";', "",
        "        public EngineConfig()",
        "        {",
        f"            _handle = NativeMethods.{tm['constructor']['ffi']}();",
        "        }", "",
    ]

    for method in tool.get("methods", []):
        mn = method["name"]
        mm = tm.get("methods", {}).get(mn, {})
        ffi_fn = mm.get("ffi", "")
        params = method.get("params", [])
        ret = method.get("returns", "void")
        cs_mn = to_pascal(mn)

        if method.get("doc"):
            lines.append(f"        /// <summary>{method['doc']}</summary>")

        if mn == "build":
            lines += [
                "        public GoudGame Build()",
                "        {",
                "            if (_handle == IntPtr.Zero) throw new ObjectDisposedException(\"EngineConfig\");",
                f"            var ctx = NativeMethods.{ffi_fn}(_handle);",
                "            _handle = IntPtr.Zero;",
                "            if (!ctx.IsValid) throw new Exception(\"Failed to create engine from config\");",
                "            return new GoudGame(ctx, _title);",
                "        }", "",
            ]
        elif mn == "destroy":
            lines += [
                "        public void Destroy()",
                "        {",
                f"            if (_handle != IntPtr.Zero) {{ NativeMethods.{ffi_fn}(_handle); _handle = IntPtr.Zero; }}",
                "        }", "",
            ]
        elif mn == "setTitle":
            lines += [
                "        public EngineConfig SetTitle(string title)",
                "        {",
                "            if (_handle == IntPtr.Zero) throw new ObjectDisposedException(\"EngineConfig\");",
                f"            NativeMethods.{ffi_fn}(_handle, title);",
                "            _title = title;",
                "            return this;",
                "        }", "",
            ]
        else:
            cs_params = ", ".join(f"{cs_type(p['type'])} {p['name']}" for p in params)
            ffi_params = []
            ffi_fn_param_index = 1  # _handle is first
            for p in params:
                if p["type"] in schema.get("enums", {}):
                    expected = _ffi_param_type_at(ffi_fn, ffi_fn_param_index)
                    if expected.startswith("Ffi") and expected[3:] in schema.get("enums", {}):
                        ffi_params.append(p["name"])
                    else:
                        underlying = schema["enums"][p["type"]].get("underlying", "i32")
                        ffi_params.append(f"({cs_type(underlying)}){p['name']}")
                else:
                    ffi_params.append(p["name"])
                ffi_fn_param_index += 1
            ffi_args = ", ".join(["_handle"] + ffi_params)
            lines += [
                f"        public EngineConfig {cs_mn}({cs_params})",
                "        {",
                "            if (_handle == IntPtr.Zero) throw new ObjectDisposedException(\"EngineConfig\");",
                f"            NativeMethods.{ffi_fn}({ffi_args});",
                "            return this;",
                "        }", "",
            ]

    lines += [
        "        public void Dispose() => Destroy();",
        "    }", "}", "",
    ]
    write_generated(OUT / "Core" / "EngineConfig.g.cs", "\n".join(lines))


def gen_ui_manager():
    if "UiManager" not in schema.get("tools", {}) or "UiManager" not in mapping.get("tools", {}):
        return
    tool = schema["tools"]["UiManager"]
    tm = mapping["tools"]["UiManager"]
    ctor_ffi = tm["constructor"]["ffi"]
    dtor_ffi = tm["destructor"]

    lines = [
        f"// {HEADER_COMMENT}",
        "using System;",
        "using System.Runtime.InteropServices;",
        "",
        f"namespace {NS}",
        "{",
        f"    /// <summary>{tool.get('doc', 'Standalone UI manager')}</summary>",
        "    public class UiManager : IDisposable",
        "    {",
        "        private IntPtr _handle;",
        "        private bool _disposed;",
        "",
        "        public UiManager()",
        "        {",
        f"            _handle = NativeMethods.{ctor_ffi}();",
        "            if (_handle == IntPtr.Zero) throw new Exception(\"Failed to create UiManager\");",
        "        }",
        "",
        "        public void Destroy()",
        "        {",
        "            if (_disposed) return;",
        f"            NativeMethods.{dtor_ffi}(_handle);",
        "            _handle = IntPtr.Zero;",
        "            _disposed = true;",
        "        }",
        "",
        "        public void Update() => NativeMethods.goud_ui_manager_update(_handle);",
        "        public void Render() => NativeMethods.goud_ui_manager_render(_handle);",
        "        public uint NodeCount() => NativeMethods.goud_ui_manager_node_count(_handle);",
        "        public ulong CreateNode(int componentType) => NativeMethods.goud_ui_create_node(_handle, componentType);",
        "        public int RemoveNode(ulong nodeId) => NativeMethods.goud_ui_remove_node(_handle, nodeId);",
        "        public int SetParent(ulong childId, ulong parentId) => NativeMethods.goud_ui_set_parent(_handle, childId, parentId);",
        "        public ulong GetParent(ulong nodeId) => NativeMethods.goud_ui_get_parent(_handle, nodeId);",
        "        public uint GetChildCount(ulong nodeId) => NativeMethods.goud_ui_get_child_count(_handle, nodeId);",
        "        public ulong GetChildAt(ulong nodeId, uint index) => NativeMethods.goud_ui_get_child_at(_handle, nodeId, index);",
        "        public int SetWidget(ulong nodeId, int widgetKind) => NativeMethods.goud_ui_set_widget(_handle, nodeId, widgetKind);",
        "",
        "        public int SetStyle(ulong nodeId, UiStyle style)",
        "        {",
        "            unsafe",
        "            {",
        "                var fontFamilyBytes = style.HasFontFamily",
        "                    ? System.Text.Encoding.UTF8.GetBytes(style.FontFamily ?? string.Empty)",
        "                    : Array.Empty<byte>();",
        "                var texturePathBytes = style.HasTexturePath",
        "                    ? System.Text.Encoding.UTF8.GetBytes(style.TexturePath ?? string.Empty)",
        "                    : Array.Empty<byte>();",
        "                fixed (byte* fontFamilyPtr = fontFamilyBytes)",
        "                fixed (byte* texturePathPtr = texturePathBytes)",
        "                {",
        "                    var ffi = new FfiUiStyle",
        "                    {",
        "                        HasBackgroundColor = style.HasBackgroundColor,",
        "                        BackgroundColor = new FfiColor { R = style.BackgroundColor.R, G = style.BackgroundColor.G, B = style.BackgroundColor.B, A = style.BackgroundColor.A },",
        "                        HasForegroundColor = style.HasForegroundColor,",
        "                        ForegroundColor = new FfiColor { R = style.ForegroundColor.R, G = style.ForegroundColor.G, B = style.ForegroundColor.B, A = style.ForegroundColor.A },",
        "                        HasBorderColor = style.HasBorderColor,",
        "                        BorderColor = new FfiColor { R = style.BorderColor.R, G = style.BorderColor.G, B = style.BorderColor.B, A = style.BorderColor.A },",
        "                        HasBorderWidth = style.HasBorderWidth,",
        "                        BorderWidth = style.BorderWidth,",
        "                        HasFontFamily = style.HasFontFamily,",
        "                        FontFamilyPtr = fontFamilyBytes.Length == 0 ? IntPtr.Zero : (IntPtr)fontFamilyPtr,",
        "                        FontFamilyLen = (nuint)fontFamilyBytes.Length,",
        "                        HasFontSize = style.HasFontSize,",
        "                        FontSize = style.FontSize,",
        "                        HasTexturePath = style.HasTexturePath,",
        "                        TexturePathPtr = texturePathBytes.Length == 0 ? IntPtr.Zero : (IntPtr)texturePathPtr,",
        "                        TexturePathLen = (nuint)texturePathBytes.Length,",
        "                        HasWidgetSpacing = style.HasWidgetSpacing,",
        "                        WidgetSpacing = style.WidgetSpacing,",
        "                    };",
        "                    return NativeMethods.goud_ui_set_style(_handle, nodeId, ref ffi);",
        "                }",
        "            }",
        "        }",
        "",
        "        public int SetLabelText(ulong nodeId, string text)",
        "        {",
        "            unsafe",
        "            {",
        "                var textBytes = System.Text.Encoding.UTF8.GetBytes(text ?? string.Empty);",
        "                fixed (byte* textPtr = textBytes)",
        "                {",
        "                    return NativeMethods.goud_ui_set_label_text(",
        "                        _handle,",
        "                        nodeId,",
        "                        textBytes.Length == 0 ? IntPtr.Zero : (IntPtr)textPtr,",
        "                        (nuint)textBytes.Length",
        "                    );",
        "                }",
        "            }",
        "        }",
        "",
        "        public int SetButtonEnabled(ulong nodeId, bool enabled) => NativeMethods.goud_ui_set_button_enabled(_handle, nodeId, enabled);",
        "",
        "        public int SetImageTexturePath(ulong nodeId, string path)",
        "        {",
        "            unsafe",
        "            {",
        "                var pathBytes = System.Text.Encoding.UTF8.GetBytes(path ?? string.Empty);",
        "                fixed (byte* pathPtr = pathBytes)",
        "                {",
        "                    return NativeMethods.goud_ui_set_image_texture_path(",
        "                        _handle,",
        "                        nodeId,",
        "                        pathBytes.Length == 0 ? IntPtr.Zero : (IntPtr)pathPtr,",
        "                        (nuint)pathBytes.Length",
        "                    );",
        "                }",
        "            }",
        "        }",
        "",
        "        public int SetSlider(ulong nodeId, float min, float max, float value, bool enabled) =>",
        "            NativeMethods.goud_ui_set_slider(_handle, nodeId, min, max, value, enabled);",
        "",
        "        public uint EventCount() => NativeMethods.goud_ui_event_count(_handle);",
        "",
        "        public UiEvent? EventRead(uint index)",
        "        {",
        "            var ffi = new FfiUiEvent();",
        "            var status = NativeMethods.goud_ui_event_read(_handle, index, ref ffi);",
        "            if (status <= 0) return null;",
        "            return new UiEvent(ffi.EventKind, ffi.NodeId, ffi.PreviousNodeId, ffi.CurrentNodeId);",
        "        }",
        "",
        "        // Convenience widget helpers",
        "        public ulong CreatePanel() => CreateNode(0);",
        "",
        "        public ulong CreateLabel(string text)",
        "        {",
        "            var node = CreateNode(2);",
        "            SetLabelText(node, text);",
        "            return node;",
        "        }",
        "",
        "        public ulong CreateButton(bool enabled = true)",
        "        {",
        "            var node = CreateNode(1);",
        "            SetButtonEnabled(node, enabled);",
        "            return node;",
        "        }",
        "",
        "        public ulong CreateImage(string path)",
        "        {",
        "            var node = CreateNode(3);",
        "            SetImageTexturePath(node, path);",
        "            return node;",
        "        }",
        "",
        "        public ulong CreateSlider(float min, float max, float value, bool enabled = true)",
        "        {",
        "            var node = CreateNode(4);",
        "            SetSlider(node, min, max, value, enabled);",
        "            return node;",
        "        }",
        "",
        "        public void Dispose() => Destroy();",
        "    }",
        "}",
        "",
    ]
    write_generated(OUT / "Core" / "UiManager.g.cs", "\n".join(lines))


