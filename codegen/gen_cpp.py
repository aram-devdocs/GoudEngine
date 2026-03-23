#!/usr/bin/env python3
"""Generate the C++ SDK RAII wrapper from the universal schema.

Reads codegen/goud_sdk.schema.json and produces
sdks/cpp/include/goud/goud.g.hpp -- a header-only C++17 module with
move-only RAII classes that delegate to the C SDK (goud/goud.h).
"""

import sys
from pathlib import Path

# Ensure codegen dir is on the path for sdk_common
sys.path.insert(0, str(Path(__file__).parent))

import sdk_common

REPO_ROOT = sdk_common.ROOT_DIR
OUTPUT_PATH = REPO_ROOT / "sdks" / "cpp" / "include" / "goud" / "goud.g.hpp"

# ---------------------------------------------------------------------------
# Type mapping: schema type -> C++ type
# ---------------------------------------------------------------------------

_SCHEMA_TO_CPP = {
    "f32": "float",
    "f64": "double",
    "u8": "std::uint8_t",
    "u16": "std::uint16_t",
    "u32": "std::uint32_t",
    "u64": "std::uint64_t",
    "i8": "std::int8_t",
    "i16": "std::int16_t",
    "i32": "std::int32_t",
    "i64": "std::int64_t",
    "bool": "bool",
    "usize": "std::size_t",
    "string": "const char *",
    "void": "void",
    "ptr": "void *",
    "Entity": "std::uint64_t",
    "Color": "::goud_color",
    "Vec2": "::goud_vec2",
    "Key": "::goud_key",
    "MouseButton": "::goud_mouse_button",
}


def cpp_type(schema_type: str) -> str:
    """Map a schema type name to a C++ type string."""
    # Strip nullable marker
    base = schema_type.rstrip("?")
    # Array types are not wrapped in the C++ SDK (low-level)
    if base.endswith("[]"):
        return "void *"
    return _SCHEMA_TO_CPP.get(base, base)


# ---------------------------------------------------------------------------
# Enum generation
# ---------------------------------------------------------------------------

def generate_enums(schema: dict) -> list[str]:
    """Generate C++ enum class wrappers from the schema enums section."""
    lines: list[str] = []
    enums = schema.get("enums", {})
    for name, edef in enums.items():
        doc = edef.get("doc", "")
        underlying = edef.get("underlying", "i32")
        cpp_under = _SCHEMA_TO_CPP.get(underlying, "int")
        lines.append(f"/** @brief {doc} */")
        lines.append(f"enum class {name} : {cpp_under} {{")
        values = edef.get("values", {})
        for vname, vval in values.items():
            lines.append(f"    {vname} = {vval},")
        lines.append("};")
        lines.append("")
    return lines


# ---------------------------------------------------------------------------
# Error class
# ---------------------------------------------------------------------------

def generate_error_class(schema: dict) -> list[str]:
    """Generate the Error class that wraps goud_error_info."""
    return [
        "/** @brief Snapshot of the last engine error.",
        " *",
        " *  Retrieve the most recent error with Error::last().  An Error is falsy",
        " *  when no error has occurred (code == SUCCESS).",
        " */",
        "class Error {",
        "public:",
        "    /** @brief Construct an empty (no-error) instance. */",
        "    Error() noexcept = default;",
        "",
        "    /** @brief Capture the last error from the engine thread-local state.",
        "     *  @return An Error populated with code, message, subsystem, and operation.",
        "     */",
        "    static Error last() {",
        "        ::goud_error_info info;",
        "        ::goud_get_last_error(&info);",
        "        return Error(info);",
        "    }",
        "",
        "    /** @brief True when an error is present (code != SUCCESS). */",
        "    explicit operator bool() const noexcept {",
        "        return code_ != SUCCESS;",
        "    }",
        "",
        "    /** @brief Numeric error code. */",
        "    ::GoudErrorCode code() const noexcept {",
        "        return code_;",
        "    }",
        "",
        "    /** @brief Recovery class hint (0 = unrecoverable). */",
        "    int recoveryClass() const noexcept {",
        "        return recovery_class_;",
        "    }",
        "",
        "    /** @brief Human-readable error description. */",
        "    const std::string &message() const noexcept {",
        "        return message_;",
        "    }",
        "",
        "    /** @brief Engine subsystem that raised the error. */",
        "    const std::string &subsystem() const noexcept {",
        "        return subsystem_;",
        "    }",
        "",
        "    /** @brief Operation that failed. */",
        "    const std::string &operation() const noexcept {",
        "        return operation_;",
        "    }",
        "",
        "private:",
        "    explicit Error(const ::goud_error_info &info)",
        "        : code_(info.code),",
        "          recovery_class_(info.recovery_class),",
        "          message_(info.message),",
        "          subsystem_(info.subsystem),",
        "          operation_(info.operation) {}",
        "",
        "    ::GoudErrorCode code_ = SUCCESS;",
        "    int recovery_class_ = 0;",
        "    std::string message_;",
        "    std::string subsystem_;",
        "    std::string operation_;",
        "};",
        "",
    ]


# ---------------------------------------------------------------------------
# EngineConfig class
# ---------------------------------------------------------------------------

_CONFIG_SETTERS = [
    # (method_name, doc, c_func, params)
    ("setTitle", "Set the window title.",
     "::goud_engine_config_set_title_utf8",
     [("const char *", "title")]),
    ("setSize", "Set the window dimensions.",
     "::goud_engine_config_set_window_size",
     [("std::uint32_t", "width"), ("std::uint32_t", "height")]),
    ("setVsync", "Enable or disable vertical sync.",
     "::goud_engine_config_set_vsync_enabled",
     [("bool", "enabled")]),
    ("setFullscreen", "Enable or disable fullscreen mode.",
     "::goud_engine_config_set_fullscreen_enabled",
     [("bool", "enabled")]),
    ("setTargetFps", "Set the target frames per second.",
     "::goud_engine_config_set_target_fps_value",
     [("std::uint32_t", "fps")]),
    ("setFpsOverlay", "Enable or disable the FPS debug overlay.",
     "::goud_engine_config_set_fps_overlay_enabled",
     [("bool", "enabled")]),
    ("setPhysicsDebug", "Enable or disable physics debug rendering.",
     "::goud_engine_config_set_physics_debug_enabled",
     [("bool", "enabled")]),
    ("setPhysicsBackend2d", "Select the 2D physics backend.",
     "::goud_engine_config_set_physics_backend_2d_value",
     [("std::uint32_t", "backend")]),
    ("setRenderBackend", "Select the rendering backend.",
     "::goud_engine_config_set_render_backend_value",
     [("std::uint32_t", "backend")]),
    ("setWindowBackend", "Select the window backend.",
     "::goud_engine_config_set_window_backend_value",
     [("std::uint32_t", "backend")]),
]


def generate_engine_config_class(schema: dict) -> list[str]:
    """Generate the EngineConfig RAII wrapper class."""
    lines: list[str] = [
        "/** @brief RAII wrapper for an engine configuration handle.",
        " *",
        " *  Move-only.  The underlying handle is freed on destruction.  Use create()",
        " *  for stack allocation or createUnique() for heap allocation.",
        " */",
        "class EngineConfig {",
        "public:",
        "    /** @brief Construct an empty (invalid) config. */",
        "    EngineConfig() noexcept = default;",
        "",
        "    /** @brief Destroy the config, releasing the underlying handle. */",
        "    ~EngineConfig() noexcept {",
        "        reset();",
        "    }",
        "",
        "    EngineConfig(const EngineConfig &) = delete;",
        "    EngineConfig &operator=(const EngineConfig &) = delete;",
        "",
        "    /** @brief Move-construct from another config. */",
        "    EngineConfig(EngineConfig &&other) noexcept",
        "        : handle_(other.release()) {}",
        "",
        "    /** @brief Move-assign from another config. */",
        "    EngineConfig &operator=(EngineConfig &&other) noexcept {",
        "        if (this != &other) {",
        "            reset();",
        "            handle_ = other.release();",
        "        }",
        "        return *this;",
        "    }",
        "",
        "    /** @brief Allocate a new engine configuration on the stack.",
        "     *  @param[out] out_status  Optional pointer to receive the status code.",
        "     *  @return A valid EngineConfig on success.",
        "     */",
        "    static EngineConfig create(int *out_status = nullptr) noexcept {",
        "        EngineConfig config;",
        "        int status = config.init();",
        "        if (out_status != nullptr) {",
        "            *out_status = status;",
        "        }",
        "        return config;",
        "    }",
        "",
        "    /** @brief Allocate a new engine configuration on the heap.",
        "     *  @param[out] out_status  Optional pointer to receive the status code.",
        "     *  @return A unique_ptr owning the config.",
        "     */",
        "    static std::unique_ptr<EngineConfig> createUnique(int *out_status = nullptr) {",
        "        auto config = std::unique_ptr<EngineConfig>(new EngineConfig());",
        "        int status = config->init();",
        "        if (out_status != nullptr) {",
        "            *out_status = status;",
        "        }",
        "        return config;",
        "    }",
        "",
        "    /** @brief Initialize the underlying config handle.",
        "     *  @return SUCCESS if the handle was created or was already valid.",
        "     */",
        "    int init() noexcept {",
        "        if (valid()) {",
        "            return SUCCESS;",
        "        }",
        "        return ::goud_engine_config_init(&handle_);",
        "    }",
        "",
        "    /** @brief Check whether the config handle is valid.",
        "     *  @return true when the handle is non-NULL.",
        "     */",
        "    bool valid() const noexcept {",
        "        return ::goud_engine_config_valid(handle_);",
        "    }",
        "",
    ]

    # Generate setter methods from the table
    for method_name, doc, c_func, params in _CONFIG_SETTERS:
        param_list = ", ".join(f"{ptype} {pname}" for ptype, pname in params)
        arg_list = ", ".join(["handle_"] + [pname for _, pname in params])
        lines.append(f"    /** @brief {doc} */")
        lines.append(f"    int {method_name}({param_list}) noexcept {{")
        lines.append(f"        return {c_func}({arg_list});")
        lines.append("    }")
        lines.append("")

    lines.extend([
        "    /** @brief Access the raw FFI config handle.",
        "     *  @return The underlying handle (may be NULL).",
        "     */",
        "    ::goud_engine_config raw() const noexcept {",
        "        return handle_;",
        "    }",
        "",
        "    /** @brief Release ownership of the raw handle.",
        "     *  @return The underlying handle.  The config is left in an invalid state.",
        "     */",
        "    ::goud_engine_config release() noexcept {",
        "        ::goud_engine_config handle = handle_;",
        "        handle_ = nullptr;",
        "        return handle;",
        "    }",
        "",
        "    /** @brief Destroy the underlying handle and reset to invalid. */",
        "    void reset() noexcept {",
        "        if (handle_ != nullptr) {",
        "            (void)::goud_engine_config_dispose(&handle_);",
        "        }",
        "    }",
        "",
        "private:",
        "    ::goud_engine_config handle_ = nullptr;",
        "};",
        "",
    ])
    return lines


# ---------------------------------------------------------------------------
# Context class -- methods generated from schema tools.GoudGame.methods
# ---------------------------------------------------------------------------

# Mapping from schema method name to C SDK function + how to wrap.
# Each entry: (cpp_method, cpp_return, cpp_params, body_lines)
# We build this dynamically from schema, but some methods need
# hand-coded mappings because the C SDK wrapper names differ from
# the schema names.

_CONTEXT_METHOD_MAP = {
    # Schema method name -> (cpp_name, return_type, params, body)
    # params is list of (type, name), body is list of code lines
    "shouldClose": (
        "shouldClose", "bool", [],
        ["return ::goud_window_should_close_checked(handle_);"]
    ),
    "beginFrame": (
        "beginFrame", "int", [],
        ["return ::goud_renderer_begin_frame(handle_);"]
    ),
    "endFrame": (
        "endFrame", "int", [],
        ["return ::goud_renderer_end_frame(handle_);"]
    ),
    "loadTexture": (
        "loadTexture", "int",
        [("const char *", "path"), ("::goud_texture &", "out_texture")],
        ["return ::goud_texture_load_path(handle_, path, &out_texture);"]
    ),
    "destroyTexture": (
        "destroyTexture", "int",
        [("::goud_texture", "texture")],
        ["return ::goud_texture_dispose(handle_, texture);"]
    ),
    "loadFont": (
        "loadFont", "int",
        [("const char *", "path"), ("::goud_font &", "out_font")],
        ["return ::goud_font_load_path(handle_, path, &out_font);"]
    ),
    "destroyFont": (
        "destroyFont", "int",
        [("::goud_font", "font")],
        ["return ::goud_font_dispose(handle_, font);"]
    ),
    "drawSprite": (
        "drawSprite", "int",
        [
            ("::goud_texture", "texture"),
            ("float", "x"), ("float", "y"),
            ("float", "width"), ("float", "height"),
            ("float", "rotation"),
            ("::goud_color", "color"),
        ],
        ["return ::goud_renderer_draw_sprite_color(handle_, texture, x, y, width, height, rotation, color);"]
    ),
    "drawQuad": (
        "drawQuad", "int",
        [
            ("float", "x"), ("float", "y"),
            ("float", "width"), ("float", "height"),
            ("::goud_color", "color"),
        ],
        ["return ::goud_renderer_draw_quad_color(handle_, x, y, width, height, color);"]
    ),
    "isKeyPressed": (
        "keyDown", "bool",
        [("::goud_key", "key")],
        ["return ::goud_input_key_down(handle_, key);"]
    ),
    "isKeyJustPressed": (
        "keyJustPressed", "bool",
        [("::goud_key", "key")],
        ["return ::goud_input_key_pressed_once(handle_, key);"]
    ),
    "isMouseButtonPressed": (
        "mouseDown", "bool",
        [("::goud_mouse_button", "button")],
        ["return ::goud_input_mouse_down(handle_, button);"]
    ),
    "getMousePosition": (
        "mousePosition", "int",
        [("::goud_vec2 &", "out_position")],
        ["return ::goud_input_mouse_position(handle_, &out_position);"]
    ),
    "getScrollDelta": (
        "scrollDelta", "int",
        [("::goud_vec2 &", "out_delta")],
        ["return ::goud_input_scroll_delta(handle_, &out_delta);"]
    ),
    "spawnEmpty": (
        "spawnEntity", "int",
        [("std::uint64_t &", "out_entity")],
        ["return ::goud_entity_spawn(handle_, &out_entity);"]
    ),
    "despawn": (
        "destroyEntity", "int",
        [("std::uint64_t", "entity")],
        ["return ::goud_entity_remove(handle_, entity);"]
    ),
    "isAlive": (
        "isEntityAlive", "bool",
        [("std::uint64_t", "entity")],
        ["return ::goud_entity_alive(handle_, entity);"]
    ),
}

# Extra methods not directly in the schema's GoudGame tool but present in
# the hand-written C++ API (window lifecycle helpers).
_EXTRA_CONTEXT_METHODS = [
    ("pollEvents", "float", [],
     "Poll window events and return the frame delta time.",
     ["return ::goud_window_poll_events_checked(handle_);"]),
    ("swapBuffers", "void", [],
     "Swap the front and back framebuffers.",
     ["::goud_window_swap_buffers_checked(handle_);"]),
    ("enableBlending", "void", [],
     "Enable alpha blending for the renderer.",
     ["::goud_renderer_enable_blending_checked(handle_);"]),
    ("deltaTime", "float", [],
     "Get the delta time for the current frame.",
     ["return ::goud_window_delta_time(handle_);"]),
    ("clear", "void", [("::goud_color", "color")],
     "Clear the framebuffer with a solid colour.",
     ["::goud_renderer_clear_color(handle_, color);"]),
    ("activateAudio", "int", [],
     "Activate the audio subsystem.",
     ["return ::goud_audio_activate_checked(handle_);"]),
    ("playAudio", "int",
     [("const void *", "asset_data"), ("std::size_t", "asset_len"),
      ("::goud_audio_player &", "out_player")],
     "Play audio from an in-memory buffer.",
     ["return ::goud_audio_play_memory(handle_, asset_data, asset_len, &out_player);"]),
    ("stopAudio", "int",
     [("::goud_audio_player", "player_id")],
     "Stop an active audio player.",
     ["return ::goud_audio_stop_checked(handle_, player_id);"]),
    ("setGlobalVolume", "int",
     [("float", "volume")],
     "Set the global audio volume.",
     ["return ::goud_audio_set_global_volume_checked(handle_, volume);"]),
    ("renderStats", "int",
     [("::goud_render_stats &", "out_stats")],
     "Retrieve per-frame render statistics.",
     ["return ::goud_renderer_stats(handle_, &out_stats);"]),
]


def generate_context_class(schema: dict) -> list[str]:
    """Generate the Context RAII wrapper class."""
    lines: list[str] = [
        "/** @brief RAII wrapper for an engine context.",
        " *",
        " *  Move-only.  Provides methods for ECS, assets, rendering, input, and",
        " *  window management.  The context is destroyed on destruction.",
        " */",
        "class Context {",
        "public:",
        "    /** @brief Construct an invalid context. */",
        "    Context() noexcept",
        "        : handle_(::goud_context_invalid()) {}",
        "",
        "    /** @brief Construct from a raw FFI context handle.",
        "     *  @param handle  Raw context handle.",
        "     */",
        "    explicit Context(::goud_context handle) noexcept",
        "        : handle_(handle) {}",
        "",
        "    /** @brief Destroy the context. */",
        "    ~Context() noexcept {",
        "        reset();",
        "    }",
        "",
        "    Context(const Context &) = delete;",
        "    Context &operator=(const Context &) = delete;",
        "",
        "    /** @brief Move-construct from another context. */",
        "    Context(Context &&other) noexcept",
        "        : handle_(other.release()) {}",
        "",
        "    /** @brief Move-assign from another context. */",
        "    Context &operator=(Context &&other) noexcept {",
        "        if (this != &other) {",
        "            reset();",
        "            handle_ = other.release();",
        "        }",
        "        return *this;",
        "    }",
        "",
        "    /** @brief Create a standalone context (without an engine).",
        "     *  @param[out] out_status  Optional pointer to receive the status code.",
        "     *  @return A valid Context on success.",
        "     */",
        "    static Context create(int *out_status = nullptr) noexcept {",
        "        Context context;",
        "        int status = ::goud_context_init(&context.handle_);",
        "        if (out_status != nullptr) {",
        "            *out_status = status;",
        "        }",
        "        return context;",
        "    }",
        "",
        "    /** @brief Create a shared-ownership context.",
        "     *  @param[out] out_status  Optional pointer to receive the status code.",
        "     *  @return A shared_ptr owning the context.",
        "     */",
        "    static std::shared_ptr<Context> createShared(int *out_status = nullptr) {",
        "        auto context = std::make_shared<Context>();",
        "        int status = ::goud_context_init(&context->handle_);",
        "        if (out_status != nullptr) {",
        "            *out_status = status;",
        "        }",
        "        return context;",
        "    }",
        "",
        "    /** @brief Check whether the context handle is valid.",
        "     *  @return true when the context is not the invalid sentinel.",
        "     */",
        "    bool valid() const noexcept {",
        "        return ::goud_context_valid(handle_);",
        "    }",
        "",
        "    /** @brief Access the raw FFI context handle.",
        "     *  @return The underlying handle.",
        "     */",
        "    ::goud_context raw() const noexcept {",
        "        return handle_;",
        "    }",
        "",
        "    /** @brief Destroy the underlying context and reset to invalid.",
        "     *  @return SUCCESS on success.",
        "     */",
        "    int reset() noexcept {",
        "        return ::goud_context_dispose(&handle_);",
        "    }",
        "",
        "    /** @brief Release ownership of the raw handle.",
        "     *  @return The underlying handle.  The context is left invalid.",
        "     */",
        "    ::goud_context release() noexcept {",
        "        ::goud_context handle = handle_;",
        "        handle_ = ::goud_context_invalid();",
        "        return handle;",
        "    }",
        "",
    ]

    # Gather methods from schema's GoudGame tool
    tool = schema.get("tools", {}).get("GoudGame", {})
    methods = tool.get("methods", [])

    # Track which methods we have already emitted to avoid duplicates
    emitted: set[str] = set()

    for m in methods:
        schema_name = m.get("name", "")
        if schema_name in emitted:
            continue

        mapping = _CONTEXT_METHOD_MAP.get(schema_name)
        if mapping is None:
            continue

        cpp_name, ret, params, body = mapping
        emitted.add(schema_name)

        doc = m.get("doc", "")
        noexcept = " noexcept" if ret != "std::string" else ""
        const = " const" if ret != "void" or cpp_name in (
            "clear", "swapBuffers", "enableBlending",
        ) else ""
        # All Context methods are const because the handle is an opaque ID
        const = " const"

        param_str = ", ".join(f"{ptype} {pname}" for ptype, pname in params)

        lines.append(f"    /** @brief {doc} */")
        lines.append(f"    {ret} {cpp_name}({param_str}){const}{noexcept} {{")
        for bline in body:
            lines.append(f"        {bline}")
        lines.append("    }")
        lines.append("")

    # Emit extra methods not in the schema tool
    for cpp_name, ret, params, doc, body in _EXTRA_CONTEXT_METHODS:
        if cpp_name in emitted:
            continue
        emitted.add(cpp_name)

        noexcept = " noexcept"
        param_str = ", ".join(f"{ptype} {pname}" for ptype, pname in params)

        lines.append(f"    /** @brief {doc} */")
        lines.append(f"    {ret} {cpp_name}({param_str}) const{noexcept} {{")
        for bline in body:
            lines.append(f"        {bline}")
        lines.append("    }")
        lines.append("")

    lines.extend([
        "private:",
        "    ::goud_context handle_;",
        "};",
        "",
    ])
    return lines


# ---------------------------------------------------------------------------
# Engine class
# ---------------------------------------------------------------------------

def generate_engine_class() -> list[str]:
    """Generate the Engine wrapper that owns a Context."""
    return [
        "/** @brief High-level engine wrapper that owns a Context.",
        " *",
        " *  Created from an EngineConfig via Engine::create().  Delegates window",
        " *  and rendering operations to the owned Context.",
        " */",
        "class Engine {",
        "public:",
        "    /** @brief Construct an empty (invalid) engine. */",
        "    Engine() noexcept = default;",
        "",
        "    /** @brief Construct from an existing context.",
        "     *  @param context  Context to take ownership of (moved).",
        "     */",
        "    explicit Engine(Context &&context) noexcept",
        "        : context_(std::move(context)) {}",
        "",
        "    /** @brief Create an engine from a configuration.",
        "     *",
        "     *  The config is consumed (moved) regardless of success or failure.",
        "     *",
        "     *  @param config            Configuration to consume.",
        "     *  @param[out] out_status   Optional pointer to receive the status code.",
        "     *  @return A valid Engine on success.",
        "     */",
        "    static Engine create(EngineConfig &&config, int *out_status = nullptr) noexcept {",
        "        Engine engine;",
        "        ::goud_context handle = ::goud_context_invalid();",
        "        ::goud_engine_config raw_config = config.release();",
        "        int status = ::goud_engine_create_checked(&raw_config, &handle);",
        "        engine.context_ = Context(handle);",
        "        if (out_status != nullptr) {",
        "            *out_status = status;",
        "        }",
        "        return engine;",
        "    }",
        "",
        "    /** @brief Create a shared-ownership engine.",
        "     *  @param config            Configuration to consume.",
        "     *  @param[out] out_status   Optional pointer to receive the status code.",
        "     *  @return A shared_ptr owning the engine.",
        "     */",
        "    static std::shared_ptr<Engine> createShared(EngineConfig &&config, int *out_status = nullptr) {",
        "        auto engine = std::make_shared<Engine>();",
        "        ::goud_context handle = ::goud_context_invalid();",
        "        ::goud_engine_config raw_config = config.release();",
        "        int status = ::goud_engine_create_checked(&raw_config, &handle);",
        "        engine->context_ = Context(handle);",
        "        if (out_status != nullptr) {",
        "            *out_status = status;",
        "        }",
        "        return engine;",
        "    }",
        "",
        "    /** @brief Check whether the engine context is valid. */",
        "    bool valid() const noexcept {",
        "        return context_.valid();",
        "    }",
        "",
        "    /** @brief Access the owned context (mutable).",
        "     *  @return Reference to the context.",
        "     */",
        "    Context &context() noexcept {",
        "        return context_;",
        "    }",
        "",
        "    /** @brief Access the owned context (const).",
        "     *  @return Const reference to the context.",
        "     */",
        "    const Context &context() const noexcept {",
        "        return context_;",
        "    }",
        "",
        "    /** @brief Access the raw FFI context handle.",
        "     *  @return The underlying handle.",
        "     */",
        "    ::goud_context raw() const noexcept {",
        "        return context_.raw();",
        "    }",
        "",
        "    /** @brief Check whether the window close has been requested.",
        "     *  @return true if the window should close.",
        "     */",
        "    bool shouldClose() const noexcept {",
        "        return context_.shouldClose();",
        "    }",
        "",
        "    /** @brief Poll window events and return the frame delta time.",
        "     *  @return Delta time in seconds.",
        "     */",
        "    float pollEvents() noexcept {",
        "        return context_.pollEvents();",
        "    }",
        "",
        "    /** @brief Swap the front and back framebuffers. */",
        "    void swapBuffers() noexcept {",
        "        context_.swapBuffers();",
        "    }",
        "",
        "    /** @brief Enable alpha blending for the renderer. */",
        "    void enableBlending() noexcept {",
        "        context_.enableBlending();",
        "    }",
        "",
        "    /** @brief Get the delta time for the current frame.",
        "     *  @return Delta time in seconds.",
        "     */",
        "    float deltaTime() const noexcept {",
        "        return context_.deltaTime();",
        "    }",
        "",
        "private:",
        "    Context context_;",
        "};",
        "",
    ]


# ---------------------------------------------------------------------------
# Main generation entry point
# ---------------------------------------------------------------------------

def generate() -> None:
    schema = sdk_common.load_schema()
    version = schema.get("version", "0.0.1")

    lines: list[str] = []
    lines.append(f"// {sdk_common.HEADER_COMMENT}")
    lines.append(f"// Schema version: {version} x-release-please-version")
    lines.append("")
    lines.append("#ifndef GOUD_CPP_SDK_GENERATED_HPP")
    lines.append("#define GOUD_CPP_SDK_GENERATED_HPP")
    lines.append("")
    lines.append("/** @file goud.g.hpp")
    lines.append(" *  @brief Auto-generated C++17 RAII wrapper for GoudEngine.")
    lines.append(" *")
    lines.append(" *  Provides move-only wrappers around the C SDK handles.")
    lines.append(" *  Most methods are noexcept and return integer status codes (0 = success).")
    lines.append(" *  Methods that allocate (Error::last, createUnique, createShared) may throw.")
    lines.append(" */")
    lines.append("")
    lines.append("#include <goud/goud.h>")
    lines.append("")
    lines.append("#include <cstddef>")
    lines.append("#include <cstdint>")
    lines.append("#include <memory>")
    lines.append("#include <string>")
    lines.append("#include <utility>")
    lines.append("")
    lines.append("namespace goud {")
    lines.append("")

    # Enums
    lines.extend(generate_enums(schema))

    # Error class
    lines.extend(generate_error_class(schema))

    # EngineConfig class
    lines.extend(generate_engine_config_class(schema))

    # Context class
    lines.extend(generate_context_class(schema))

    # Engine class
    lines.extend(generate_engine_class())

    lines.append("}  // namespace goud")
    lines.append("")
    lines.append("#endif")
    lines.append("")

    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text("\n".join(lines), encoding="utf-8")
    print(f"Generated {OUTPUT_PATH}")


if __name__ == "__main__":
    generate()
