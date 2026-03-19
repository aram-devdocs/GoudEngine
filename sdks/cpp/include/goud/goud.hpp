#ifndef GOUD_CPP_SDK_HPP
#define GOUD_CPP_SDK_HPP

/** @file goud.hpp
 *  @brief C++17 RAII wrapper for GoudEngine.
 *
 *  Provides move-only, exception-free wrappers around the C SDK handles.
 *  All methods are noexcept and return integer status codes (0 = success).
 */

#include <goud/goud.h>

#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <utility>

namespace goud {

/** @brief Snapshot of the last engine error.
 *
 *  Retrieve the most recent error with Error::last().  An Error is falsy
 *  when no error has occurred (code == SUCCESS).
 */
class Error {
public:
    /** @brief Construct an empty (no-error) instance. */
    Error() noexcept = default;

    /** @brief Capture the last error from the engine thread-local state.
     *  @return An Error populated with code, message, subsystem, and operation.
     */
    static Error last() {
        ::goud_error_info info;
        ::goud_get_last_error(&info);
        return Error(info);
    }

    /** @brief True when an error is present (code != SUCCESS). */
    explicit operator bool() const noexcept {
        return code_ != SUCCESS;
    }

    /** @brief Numeric error code. */
    ::GoudErrorCode code() const noexcept {
        return code_;
    }

    /** @brief Recovery class hint (0 = unrecoverable). */
    int recoveryClass() const noexcept {
        return recovery_class_;
    }

    /** @brief Human-readable error description. */
    const std::string &message() const noexcept {
        return message_;
    }

    /** @brief Engine subsystem that raised the error. */
    const std::string &subsystem() const noexcept {
        return subsystem_;
    }

    /** @brief Operation that failed. */
    const std::string &operation() const noexcept {
        return operation_;
    }

private:
    explicit Error(const ::goud_error_info &info)
        : code_(info.code),
          recovery_class_(info.recovery_class),
          message_(info.message),
          subsystem_(info.subsystem),
          operation_(info.operation) {}

    ::GoudErrorCode code_ = SUCCESS;
    int recovery_class_ = 0;
    std::string message_;
    std::string subsystem_;
    std::string operation_;
};

/** @brief RAII wrapper for an engine configuration handle.
 *
 *  Move-only.  The underlying handle is freed on destruction.  Use create()
 *  for stack allocation or createUnique() for heap allocation.
 */
class EngineConfig {
public:
    /** @brief Construct an empty (invalid) config. */
    EngineConfig() noexcept = default;

    /** @brief Destroy the config, releasing the underlying handle. */
    ~EngineConfig() noexcept {
        reset();
    }

    EngineConfig(const EngineConfig &) = delete;
    EngineConfig &operator=(const EngineConfig &) = delete;

    /** @brief Move-construct from another config. */
    EngineConfig(EngineConfig &&other) noexcept
        : handle_(other.release()) {}

    /** @brief Move-assign from another config. */
    EngineConfig &operator=(EngineConfig &&other) noexcept {
        if (this != &other) {
            reset();
            handle_ = other.release();
        }
        return *this;
    }

    /** @brief Allocate a new engine configuration on the stack.
     *  @param[out] out_status  Optional pointer to receive the status code.
     *  @return A valid EngineConfig on success.
     */
    static EngineConfig create(int *out_status = nullptr) noexcept {
        EngineConfig config;
        int status = config.init();
        if (out_status != nullptr) {
            *out_status = status;
        }
        return config;
    }

    /** @brief Allocate a new engine configuration on the heap.
     *  @param[out] out_status  Optional pointer to receive the status code.
     *  @return A unique_ptr owning the config.
     */
    static std::unique_ptr<EngineConfig> createUnique(int *out_status = nullptr) {
        auto config = std::unique_ptr<EngineConfig>(new EngineConfig());
        int status = config->init();
        if (out_status != nullptr) {
            *out_status = status;
        }
        return config;
    }

    /** @brief Initialize the underlying config handle.
     *  @return SUCCESS if the handle was created or was already valid.
     */
    int init() noexcept {
        if (valid()) {
            return SUCCESS;
        }
        return ::goud_engine_config_init(&handle_);
    }

    /** @brief Check whether the config handle is valid.
     *  @return true when the handle is non-NULL.
     */
    bool valid() const noexcept {
        return ::goud_engine_config_valid(handle_);
    }

    /** @brief Set the window title.
     *  @param title  Null-terminated UTF-8 string.
     *  @return SUCCESS on success.
     */
    int setTitle(const char *title) noexcept {
        return ::goud_engine_config_set_title_utf8(handle_, title);
    }

    /** @brief Set the window dimensions.
     *  @param width   Width in pixels.
     *  @param height  Height in pixels.
     *  @return SUCCESS on success.
     */
    int setSize(std::uint32_t width, std::uint32_t height) noexcept {
        return ::goud_engine_config_set_window_size(handle_, width, height);
    }

    /** @brief Enable or disable vertical sync.
     *  @param enabled  true to enable vsync.
     *  @return SUCCESS on success.
     */
    int setVsync(bool enabled) noexcept {
        return ::goud_engine_config_set_vsync_enabled(handle_, enabled);
    }

    /** @brief Set the target frames per second.
     *  @param fps  Target FPS (0 = unlimited).
     *  @return SUCCESS on success.
     */
    int setTargetFps(std::uint32_t fps) noexcept {
        return ::goud_engine_config_set_target_fps_value(handle_, fps);
    }

    /** @brief Select the rendering backend.
     *  @param backend  Backend identifier.
     *  @return SUCCESS on success.
     */
    int setRenderBackend(std::uint32_t backend) noexcept {
        return ::goud_engine_config_set_render_backend_value(handle_, backend);
    }

    /** @brief Select the window backend.
     *  @param backend  Backend identifier.
     *  @return SUCCESS on success.
     */
    int setWindowBackend(std::uint32_t backend) noexcept {
        return ::goud_engine_config_set_window_backend_value(handle_, backend);
    }

    /** @brief Access the raw FFI config handle.
     *  @return The underlying handle (may be NULL).
     */
    ::goud_engine_config raw() const noexcept {
        return handle_;
    }

    /** @brief Release ownership of the raw handle.
     *  @return The underlying handle.  The config is left in an invalid state.
     */
    ::goud_engine_config release() noexcept {
        ::goud_engine_config handle = handle_;
        handle_ = nullptr;
        return handle;
    }

    /** @brief Destroy the underlying handle and reset to invalid. */
    void reset() noexcept {
        if (handle_ != nullptr) {
            (void)::goud_engine_config_dispose(&handle_);
        }
    }

private:
    ::goud_engine_config handle_ = nullptr;
};

/** @brief RAII wrapper for an engine context.
 *
 *  Move-only.  Provides methods for ECS, assets, rendering, input, and
 *  window management.  The context is destroyed on destruction.
 */
class Context {
public:
    /** @brief Construct an invalid context. */
    Context() noexcept
        : handle_(::goud_context_invalid()) {}

    /** @brief Construct from a raw FFI context handle.
     *  @param handle  Raw context handle.
     */
    explicit Context(::goud_context handle) noexcept
        : handle_(handle) {}

    /** @brief Destroy the context. */
    ~Context() noexcept {
        reset();
    }

    Context(const Context &) = delete;
    Context &operator=(const Context &) = delete;

    /** @brief Move-construct from another context. */
    Context(Context &&other) noexcept
        : handle_(other.release()) {}

    /** @brief Move-assign from another context. */
    Context &operator=(Context &&other) noexcept {
        if (this != &other) {
            reset();
            handle_ = other.release();
        }
        return *this;
    }

    /** @brief Create a standalone context (without an engine).
     *  @param[out] out_status  Optional pointer to receive the status code.
     *  @return A valid Context on success.
     */
    static Context create(int *out_status = nullptr) noexcept {
        Context context;
        int status = ::goud_context_init(&context.handle_);
        if (out_status != nullptr) {
            *out_status = status;
        }
        return context;
    }

    /** @brief Create a shared-ownership context.
     *  @param[out] out_status  Optional pointer to receive the status code.
     *  @return A shared_ptr owning the context.
     */
    static std::shared_ptr<Context> createShared(int *out_status = nullptr) {
        auto context = std::make_shared<Context>();
        int status = ::goud_context_init(&context->handle_);
        if (out_status != nullptr) {
            *out_status = status;
        }
        return context;
    }

    /** @brief Check whether the context handle is valid.
     *  @return true when the context is not the invalid sentinel.
     */
    bool valid() const noexcept {
        return ::goud_context_valid(handle_);
    }

    /** @brief Access the raw FFI context handle.
     *  @return The underlying handle.
     */
    ::goud_context raw() const noexcept {
        return handle_;
    }

    /** @brief Destroy the underlying context and reset to invalid.
     *  @return SUCCESS on success.
     */
    int reset() noexcept {
        return ::goud_context_dispose(&handle_);
    }

    /** @brief Release ownership of the raw handle.
     *  @return The underlying handle.  The context is left invalid.
     */
    ::goud_context release() noexcept {
        ::goud_context handle = handle_;
        handle_ = ::goud_context_invalid();
        return handle;
    }

    /** @brief Spawn an empty entity.
     *  @param[out] out_entity  Receives the new entity handle.
     *  @return SUCCESS on success.
     */
    int spawnEntity(std::uint64_t &out_entity) const noexcept {
        return ::goud_entity_spawn(handle_, &out_entity);
    }

    /** @brief Destroy an entity.
     *  @param entity  Entity handle.
     *  @return SUCCESS on success.
     */
    int destroyEntity(std::uint64_t entity) const noexcept {
        return ::goud_entity_remove(handle_, entity);
    }

    /** @brief Check whether an entity is alive.
     *  @param entity  Entity handle.
     *  @return true if the entity exists.
     */
    bool isEntityAlive(std::uint64_t entity) const noexcept {
        return ::goud_entity_alive(handle_, entity);
    }

    /** @brief Load a texture from a file path.
     *  @param path              Null-terminated file path.
     *  @param[out] out_texture  Receives the texture handle.
     *  @return SUCCESS on success.
     */
    int loadTexture(const char *path, ::goud_texture &out_texture) const noexcept {
        return ::goud_texture_load_path(handle_, path, &out_texture);
    }

    /** @brief Begin a new render frame.
     *  @return SUCCESS on success.
     */
    int beginFrame() const noexcept {
        return ::goud_renderer_begin_frame(handle_);
    }

    /** @brief End the current render frame.
     *  @return SUCCESS on success.
     */
    int endFrame() const noexcept {
        return ::goud_renderer_end_frame(handle_);
    }

    /** @brief Clear the framebuffer with a solid colour.
     *  @param color  Clear colour.
     */
    void clear(::goud_color color) const noexcept {
        ::goud_renderer_clear_color(handle_, color);
    }

    /** @brief Draw a textured sprite.
     *  @param texture   Texture handle.
     *  @param x         X position.
     *  @param y         Y position.
     *  @param width     Sprite width.
     *  @param height    Sprite height.
     *  @param rotation  Rotation in radians.
     *  @param color     Tint colour.
     *  @return SUCCESS on success.
     */
    int drawSprite(
        ::goud_texture texture,
        float x,
        float y,
        float width,
        float height,
        float rotation,
        ::goud_color color
    ) const noexcept {
        return ::goud_renderer_draw_sprite_color(handle_, texture, x, y, width, height, rotation, color);
    }

    /** @brief Test whether a key is currently held down.
     *  @param key  Key code.
     *  @return true if pressed.
     */
    bool keyDown(::goud_key key) const noexcept {
        return ::goud_input_key_down(handle_, key);
    }

    /** @brief Get the current mouse cursor position.
     *  @param[out] out_position  Receives the (x, y) position.
     *  @return SUCCESS on success.
     */
    int mousePosition(::goud_vec2 &out_position) const noexcept {
        return ::goud_input_mouse_position(handle_, &out_position);
    }

    /** @brief Check whether the window close has been requested.
     *  @return true if the window should close.
     */
    bool shouldClose() const noexcept {
        return ::goud_window_should_close_checked(handle_);
    }

    /** @brief Poll window events and return the frame delta time.
     *  @return Delta time in seconds.
     */
    float pollEvents() const noexcept {
        return ::goud_window_poll_events_checked(handle_);
    }

    /** @brief Swap the front and back framebuffers. */
    void swapBuffers() const noexcept {
        ::goud_window_swap_buffers_checked(handle_);
    }

    /** @brief Enable alpha blending for the renderer. */
    void enableBlending() const noexcept {
        ::goud_renderer_enable_blending_checked(handle_);
    }

    /** @brief Get the delta time for the current frame.
     *  @return Delta time in seconds.
     */
    float deltaTime() const noexcept {
        return ::goud_window_delta_time(handle_);
    }

    /** @brief Test whether a mouse button is currently held down.
     *  @param button  Mouse button.
     *  @return true if pressed.
     */
    bool mouseDown(::goud_mouse_button button) const noexcept {
        return ::goud_input_mouse_down(handle_, button);
    }

    /** @brief Test whether a key was pressed this frame (edge trigger).
     *  @param key  Key code.
     *  @return true if just pressed.
     */
    bool keyJustPressed(::goud_key key) const noexcept {
        return ::goud_input_key_pressed_once(handle_, key);
    }

    /** @brief Draw a solid-colour quad.
     *  @param x       X position.
     *  @param y       Y position.
     *  @param width   Quad width.
     *  @param height  Quad height.
     *  @param color   Fill colour.
     *  @return SUCCESS on success.
     */
    int drawQuad(float x, float y, float width, float height, ::goud_color color) const noexcept {
        return ::goud_renderer_draw_quad_color(handle_, x, y, width, height, color);
    }

    /** @brief Load a font from a file path.
     *  @param path          Null-terminated file path.
     *  @param[out] out_font Receives the font handle.
     *  @return SUCCESS on success.
     */
    int loadFont(const char *path, ::goud_font &out_font) const noexcept {
        return ::goud_font_load_path(handle_, path, &out_font);
    }

    /** @brief Activate the audio subsystem.
     *  @return SUCCESS on success.
     */
    int activateAudio() const noexcept {
        return ::goud_audio_activate_checked(handle_);
    }

    /** @brief Play audio from an in-memory buffer.
     *  @param asset_data       Pointer to audio data.
     *  @param asset_len        Length of @p asset_data in bytes.
     *  @param[out] out_player  Receives the player handle.
     *  @return SUCCESS on success.
     */
    int playAudio(const void *asset_data, std::size_t asset_len, ::goud_audio_player &out_player) const noexcept {
        return ::goud_audio_play_memory(handle_, asset_data, asset_len, &out_player);
    }

private:
    ::goud_context handle_;
};

/** @brief High-level engine wrapper that owns a Context.
 *
 *  Created from an EngineConfig via Engine::create().  Delegates window
 *  and rendering operations to the owned Context.
 */
class Engine {
public:
    /** @brief Construct an empty (invalid) engine. */
    Engine() noexcept = default;

    /** @brief Construct from an existing context.
     *  @param context  Context to take ownership of (moved).
     */
    explicit Engine(Context &&context) noexcept
        : context_(std::move(context)) {}

    /** @brief Create an engine from a configuration.
     *
     *  The config is consumed (moved) regardless of success or failure.
     *
     *  @param config            Configuration to consume.
     *  @param[out] out_status   Optional pointer to receive the status code.
     *  @return A valid Engine on success.
     */
    static Engine create(EngineConfig &&config, int *out_status = nullptr) noexcept {
        Engine engine;
        ::goud_context handle = ::goud_context_invalid();
        ::goud_engine_config raw_config = config.release();
        int status = ::goud_engine_create_checked(&raw_config, &handle);
        engine.context_ = Context(handle);
        if (out_status != nullptr) {
            *out_status = status;
        }
        return engine;
    }

    /** @brief Create a shared-ownership engine.
     *  @param config            Configuration to consume.
     *  @param[out] out_status   Optional pointer to receive the status code.
     *  @return A shared_ptr owning the engine.
     */
    static std::shared_ptr<Engine> createShared(EngineConfig &&config, int *out_status = nullptr) {
        auto engine = std::make_shared<Engine>();
        ::goud_context handle = ::goud_context_invalid();
        ::goud_engine_config raw_config = config.release();
        int status = ::goud_engine_create_checked(&raw_config, &handle);
        engine->context_ = Context(handle);
        if (out_status != nullptr) {
            *out_status = status;
        }
        return engine;
    }

    /** @brief Check whether the engine context is valid. */
    bool valid() const noexcept {
        return context_.valid();
    }

    /** @brief Access the owned context (mutable).
     *  @return Reference to the context.
     */
    Context &context() noexcept {
        return context_;
    }

    /** @brief Access the owned context (const).
     *  @return Const reference to the context.
     */
    const Context &context() const noexcept {
        return context_;
    }

    /** @brief Access the raw FFI context handle.
     *  @return The underlying handle.
     */
    ::goud_context raw() const noexcept {
        return context_.raw();
    }

    /** @brief Check whether the window close has been requested.
     *  @return true if the window should close.
     */
    bool shouldClose() const noexcept {
        return context_.shouldClose();
    }

    /** @brief Poll window events and return the frame delta time.
     *  @return Delta time in seconds.
     */
    float pollEvents() noexcept {
        return context_.pollEvents();
    }

    /** @brief Swap the front and back framebuffers. */
    void swapBuffers() noexcept {
        context_.swapBuffers();
    }

    /** @brief Enable alpha blending for the renderer. */
    void enableBlending() noexcept {
        context_.enableBlending();
    }

    /** @brief Get the delta time for the current frame.
     *  @return Delta time in seconds.
     */
    float deltaTime() const noexcept {
        return context_.deltaTime();
    }

private:
    Context context_;
};

}  // namespace goud

#endif
