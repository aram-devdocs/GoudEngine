"""Tool class generation for game/context wrappers."""

from pathlib import Path
from sdk_common import HEADER_COMMENT, to_pascal, to_snake, write_generated
from .context import NS, OUT, schema, mapping
from .component_body import _gen_component_body
from .method_body import _gen_method_body
from .param_strings import _safe_param_strs
from .helpers import cs_type, _to_cs_field, _cs_default_value

# Methods whose FFI signatures use patterns the C# wrapper codegen
# doesn't handle yet (e.g. mixed buffer+out params, ptr↔ulong casts).
# Skipped from wrapper generation; expose via a higher-level API later.
_CSHARP_SKIP_METHODS = {
    "rollbackCreate",    # statePtr is *mut u8 (IntPtr) but schema says u64
    "rpcCall",           # has call_id_out (*mut u64) out param not in schema
    "rpcReceiveResponse",  # out_buffer template doesn't match FFI signature
    "rpcDrainOne",       # same out_buffer pattern issue as rpcReceiveResponse
}


def _gen_tool_class(tool_name: str, tm: dict, out_path, is_windowed: bool = False):
    tool = schema["tools"][tool_name]
    class_name = tool_name
    extra = []
    needs_network_receive_buffer_cache = any(
        tm.get("methods", {}).get(method["name"], {}).get("ffi") == "goud_network_receive"
        and tm.get("methods", {}).get(method["name"], {}).get("out_buffer")
        for method in tool.get("methods", [])
    )
    if is_windowed:
        for prop in tool.get("properties", []):
            pm_check = tm.get("properties", {}).get(prop["name"], {})
            if pm_check.get("source") == "cached":
                pt_priv = cs_type(prop["type"])
                field = _to_cs_field(pm_check.get("field", f"_{to_snake(prop['name'])}"))
                extra.append(f"        private {pt_priv} {field};")
    if needs_network_receive_buffer_cache:
        extra.append("        private int? _networkReceiveBufferSize;")
    ctor_params = tool.get("constructor", {}).get("params", [])
    ctor_ffi = tm.get("constructor", {}).get("ffi", "goud_context_create")

    if ctor_params:
        cs_ps = []
        for p in ctor_params:
            ct = cs_type(p["type"])
            d = p.get("default")
            if d is None:
                cs_ps.append(f"{ct} {p['name']}")
            elif isinstance(d, str) and not str(d).isdigit():
                cs_ps.append(f'{ct} {p["name"]} = "{d}"')
            else:
                cs_ps.append(f"{ct} {p['name']} = {d}")
        ctor_sig = ", ".join(cs_ps)
        ctor_call = f"NativeMethods.{ctor_ffi}({', '.join(p['name'] for p in ctor_params)})"
    else:
        ctor_sig = ""
        ctor_call = f"NativeMethods.{ctor_ffi}()"

    err_msg = "Failed to create GLFW window" if is_windowed else "Failed to create headless context"
    # Build constructor body with cached field initialization
    ctor_body_lines = [
        f"            _ctx = {ctor_call};",
        f'            if (!_ctx.IsValid) throw new Exception("{err_msg}");',
    ]
    if is_windowed:
        ctor_param_names = {p["name"] for p in ctor_params}
        for prop in tool.get("properties", []):
            pm_init = tm.get("properties", {}).get(prop["name"], {})
            if pm_init.get("source") == "cached":
                field = _to_cs_field(pm_init.get("field", f"_{to_snake(prop['name'])}"))
                if prop["name"] in ctor_param_names:
                    ctor_body_lines.append(f"            {field} = {prop['name']};")
                else:
                    default_val = _cs_default_value(cs_type(prop["type"]))
                    ctor_body_lines.append(f"            {field} = {default_val};")

    lines = [
        f"// {HEADER_COMMENT}",
        "using System;", "using System.Linq;", "using System.Runtime.InteropServices;", "using System.Text.Json;", "",
        f"namespace {NS}", "{",
    ]
    # Emit SrcRectMode, SpriteCmd, and batch-related types before the GoudGame class
    if class_name == "GoudGame":
        lines += [
            "    /// <summary>Source rectangle coordinate mode for DrawSpriteRect.</summary>",
            "    public enum SrcRectMode : uint",
            "    {",
            "        /// <summary>Source rectangle values are normalized UV coordinates (0.0-1.0).</summary>",
            "        Normalized = 0,",
            "        /// <summary>Source rectangle values are in pixel coordinates.</summary>",
            "        Pixels = 1,",
            "    }",
            "",
            "    /// <summary>A sprite command for batch rendering via DrawSpriteBatch.</summary>",
            "    public struct SpriteCmd",
            "    {",
            "        public ulong Texture;",
            "        public float X, Y, Width, Height, Rotation;",
            "        public float SrcX, SrcY, SrcW, SrcH;",
            "        public Color? Color;",
            "        public int ZLayer;",
            "    }",
            "",
        ]
    lines += [
        f"    /// <summary>{tool.get('doc', class_name)}</summary>",
        f"    public class {class_name} : IDisposable", "    {",
        "        private GoudContextId _ctx;",
        "        private bool _disposed;",
        *extra, "",
        f"        public {class_name}({ctor_sig})", "        {",
        *ctor_body_lines,
        "        }", "",
    ]

    if class_name == "GoudContext":
        lines += [
            "        public GoudContext(ContextConfig config)",
            "        {",
            "            var _debuggerFfi = new GoudDebuggerConfig",
            "            {",
            "                Enabled = config.Debugger.Enabled,",
            "                PublishLocalAttach = config.Debugger.PublishLocalAttach,",
            "                RouteLabel = config.Debugger.RouteLabel,",
            "            };",
            "            var _configFfi = new GoudContextConfig",
            "            {",
            "                Debugger = _debuggerFfi,",
            "            };",
            "            _ctx = NativeMethods.goud_context_create_with_config(ref _configFfi);",
            '            if (!_ctx.IsValid) throw new Exception("Failed to create headless context");',
            "        }",
            "",
        ]

    # Internal constructor for EngineConfig.Build() to construct from pre-created context
    if is_windowed:
        lines += [
            f'        internal {class_name}(GoudContextId ctx, string title = "GoudEngine")', "        {",
            "            _ctx = ctx;",
            '            if (!_ctx.IsValid) throw new Exception("Invalid context ID");',
        ]
        # Initialize cached fields from windowed properties
        for prop in tool.get("properties", []):
            pm_init = tm.get("properties", {}).get(prop["name"], {})
            if pm_init.get("source") == "cached":
                field = _to_cs_field(pm_init.get("field", f"_{to_snake(prop['name'])}"))
                if field == "_title":
                    lines.append(f"            {field} = title;")
                else:
                    default_val = _cs_default_value(cs_type(prop["type"]))
                    lines.append(f"            {field} = {default_val};")
        lines += ["        }", ""]

    if needs_network_receive_buffer_cache:
        lines += [
            "        private int GetNetworkReceiveBufferSize()",
            "        {",
            "            if (_networkReceiveBufferSize.HasValue)",
            "            {",
            "                return _networkReceiveBufferSize.Value;",
            "            }",
            "",
            "            NetworkCapabilities _caps = default;",
            "            NativeMethods.goud_provider_network_capabilities(_ctx, ref _caps);",
            "            _networkReceiveBufferSize = _caps.MaxMessageSize switch",
            "            {",
            "                0 => 65536,",
            "                > int.MaxValue => int.MaxValue,",
            "                _ => (int)_caps.MaxMessageSize,",
            "            };",
            "            return _networkReceiveBufferSize.Value;",
            "        }",
            "",
        ]

    # Properties (windowed only)
    for prop in tool.get("properties", []):
        pn = to_pascal(prop["name"])
        pt = cs_type(prop["type"])
        pm = tm.get("properties", {}).get(prop["name"], {})
        src = pm.get("source", "")
        if src == "cached":
            field = _to_cs_field(pm.get("field", f"_{to_snake(prop['name'])}"))
            lines.append(f"        public {pt} {pn} => {field};")
        elif src == "computed":
            # Computed properties reference their dependent cached fields
            lines.append(f"        public {pt} {pn} => _deltaTime > 0 ? 1f / _deltaTime : 0f;")
        elif "out_index" in pm:
            idx = pm["out_index"]
            lines += [f"        public {pt} {pn}", "        {", "            get", "            {",
                      "                uint w = 0, h = 0;",
                      f"                NativeMethods.{pm['ffi']}(_ctx, ref w, ref h);",
                      f"                return {'w' if idx == 0 else 'h'};",
                      "            }", "        }"]
        lines.append("")

    # Methods
    for method in tool.get("methods", []):
        if method["name"] in _CSHARP_SKIP_METHODS:
            continue
        mn = to_pascal(method["name"])
        mm = tm.get("methods", {}).get(method["name"], {})
        params = method.get("params", [])
        ret = method.get("returns", "void")
        cs_ret = cs_type(ret.rstrip("[]").rstrip("?"))
        if ret.endswith("[]"):
            actual_ret = f"{cs_ret}[]"
        elif ret.endswith("?") or method.get("nullable", False):
            actual_ret = f"{cs_ret}?"
        elif method.get("async"):
            actual_ret = cs_ret
        else:
            actual_ret = cs_ret

        sig = ", ".join(_safe_param_strs(params))
        if method.get("doc"):
            lines.append(f"        /// <summary>{method['doc']}</summary>")
        lines += [f"        public {actual_ret} {mn}({sig})", "        {"]
        _gen_method_body(mn, mm, params, ret, lines, is_windowed)
        lines += ["        }", ""]

    dispose_body = "Destroy()" if is_windowed else "_ctx = GoudContextId.Invalid"
    lines += [f"        public void Dispose() => {dispose_body};", "    }", "}", ""]
    write_generated(out_path, "\n".join(lines))


def gen_game():
    _gen_tool_class("GoudGame", mapping["tools"]["GoudGame"], OUT / "GoudGame.g.cs", is_windowed=True)


def gen_context():
    _gen_tool_class("GoudContext", mapping["tools"]["GoudContext"], OUT / "GoudContext.g.cs", is_windowed=False)
