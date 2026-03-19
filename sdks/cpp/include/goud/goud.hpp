#ifndef GOUD_CPP_SDK_HPP
#define GOUD_CPP_SDK_HPP

#include <goud/goud.h>

#include <cstddef>
#include <cstdint>
#include <memory>
#include <string>
#include <utility>

namespace goud {

class Error {
public:
    Error() noexcept = default;

    static Error last() {
        ::goud_error_info info;
        ::goud_get_last_error(&info);
        return Error(info);
    }

    explicit operator bool() const noexcept {
        return code_ != SUCCESS;
    }

    ::GoudErrorCode code() const noexcept {
        return code_;
    }

    int recoveryClass() const noexcept {
        return recovery_class_;
    }

    const std::string &message() const noexcept {
        return message_;
    }

    const std::string &subsystem() const noexcept {
        return subsystem_;
    }

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

class EngineConfig {
public:
    EngineConfig() noexcept = default;

    ~EngineConfig() noexcept {
        reset();
    }

    EngineConfig(const EngineConfig &) = delete;
    EngineConfig &operator=(const EngineConfig &) = delete;

    EngineConfig(EngineConfig &&other) noexcept
        : handle_(other.release()) {}

    EngineConfig &operator=(EngineConfig &&other) noexcept {
        if (this != &other) {
            reset();
            handle_ = other.release();
        }
        return *this;
    }

    static EngineConfig create(int *out_status = nullptr) noexcept {
        EngineConfig config;
        int status = config.init();
        if (out_status != nullptr) {
            *out_status = status;
        }
        return config;
    }

    static std::unique_ptr<EngineConfig> createUnique(int *out_status = nullptr) {
        auto config = std::unique_ptr<EngineConfig>(new EngineConfig());
        int status = config->init();
        if (out_status != nullptr) {
            *out_status = status;
        }
        return config;
    }

    int init() noexcept {
        if (valid()) {
            return SUCCESS;
        }
        return ::goud_engine_config_init(&handle_);
    }

    bool valid() const noexcept {
        return ::goud_engine_config_valid(handle_);
    }

    int setTitle(const char *title) noexcept {
        return ::goud_engine_config_set_title_utf8(handle_, title);
    }

    int setSize(std::uint32_t width, std::uint32_t height) noexcept {
        return ::goud_engine_config_set_window_size(handle_, width, height);
    }

    int setVsync(bool enabled) noexcept {
        return ::goud_engine_config_set_vsync_enabled(handle_, enabled);
    }

    int setTargetFps(std::uint32_t fps) noexcept {
        return ::goud_engine_config_set_target_fps_value(handle_, fps);
    }

    int setRenderBackend(std::uint32_t backend) noexcept {
        return ::goud_engine_config_set_render_backend_value(handle_, backend);
    }

    int setWindowBackend(std::uint32_t backend) noexcept {
        return ::goud_engine_config_set_window_backend_value(handle_, backend);
    }

    ::goud_engine_config raw() const noexcept {
        return handle_;
    }

    ::goud_engine_config release() noexcept {
        ::goud_engine_config handle = handle_;
        handle_ = nullptr;
        return handle;
    }

    void reset() noexcept {
        if (handle_ != nullptr) {
            (void)::goud_engine_config_dispose(&handle_);
        }
    }

private:
    ::goud_engine_config handle_ = nullptr;
};

class Context {
public:
    Context() noexcept
        : handle_(::goud_context_invalid()) {}

    explicit Context(::goud_context handle) noexcept
        : handle_(handle) {}

    ~Context() noexcept {
        reset();
    }

    Context(const Context &) = delete;
    Context &operator=(const Context &) = delete;

    Context(Context &&other) noexcept
        : handle_(other.release()) {}

    Context &operator=(Context &&other) noexcept {
        if (this != &other) {
            reset();
            handle_ = other.release();
        }
        return *this;
    }

    static Context create(int *out_status = nullptr) noexcept {
        Context context;
        int status = ::goud_context_init(&context.handle_);
        if (out_status != nullptr) {
            *out_status = status;
        }
        return context;
    }

    static std::shared_ptr<Context> createShared(int *out_status = nullptr) {
        auto context = std::make_shared<Context>();
        int status = ::goud_context_init(&context->handle_);
        if (out_status != nullptr) {
            *out_status = status;
        }
        return context;
    }

    bool valid() const noexcept {
        return ::goud_context_valid(handle_);
    }

    ::goud_context raw() const noexcept {
        return handle_;
    }

    int reset() noexcept {
        return ::goud_context_dispose(&handle_);
    }

    ::goud_context release() noexcept {
        ::goud_context handle = handle_;
        handle_ = ::goud_context_invalid();
        return handle;
    }

    int spawnEntity(std::uint64_t &out_entity) const noexcept {
        return ::goud_entity_spawn(handle_, &out_entity);
    }

    int destroyEntity(std::uint64_t entity) const noexcept {
        return ::goud_entity_remove(handle_, entity);
    }

    bool isEntityAlive(std::uint64_t entity) const noexcept {
        return ::goud_entity_alive(handle_, entity);
    }

    int loadTexture(const char *path, ::goud_texture &out_texture) const noexcept {
        return ::goud_texture_load_path(handle_, path, &out_texture);
    }

    int beginFrame() const noexcept {
        return ::goud_renderer_begin_frame(handle_);
    }

    int endFrame() const noexcept {
        return ::goud_renderer_end_frame(handle_);
    }

    void clear(::goud_color color) const noexcept {
        ::goud_renderer_clear_color(handle_, color);
    }

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

    bool keyDown(::goud_key key) const noexcept {
        return ::goud_input_key_down(handle_, key);
    }

    int mousePosition(::goud_vec2 &out_position) const noexcept {
        return ::goud_input_mouse_position(handle_, &out_position);
    }

    bool shouldClose() const noexcept {
        return ::goud_window_should_close_checked(handle_);
    }

    float pollEvents() const noexcept {
        return ::goud_window_poll_events_checked(handle_);
    }

    void swapBuffers() const noexcept {
        ::goud_window_swap_buffers_checked(handle_);
    }

    void enableBlending() const noexcept {
        ::goud_renderer_enable_blending_checked(handle_);
    }

    float deltaTime() const noexcept {
        return ::goud_window_delta_time(handle_);
    }

    bool mouseDown(::goud_mouse_button button) const noexcept {
        return ::goud_input_mouse_down(handle_, button);
    }

    bool keyJustPressed(::goud_key key) const noexcept {
        return ::goud_input_key_pressed_once(handle_, key);
    }

    int drawQuad(float x, float y, float width, float height, ::goud_color color) const noexcept {
        return ::goud_renderer_draw_quad_color(handle_, x, y, width, height, color);
    }

    int loadFont(const char *path, ::goud_font &out_font) const noexcept {
        return ::goud_font_load_path(handle_, path, &out_font);
    }

    int activateAudio() const noexcept {
        return ::goud_audio_activate_checked(handle_);
    }

    int playAudio(const void *asset_data, std::size_t asset_len, ::goud_audio_player &out_player) const noexcept {
        return ::goud_audio_play_memory(handle_, asset_data, asset_len, &out_player);
    }

private:
    ::goud_context handle_;
};

class Engine {
public:
    Engine() noexcept = default;

    explicit Engine(Context &&context) noexcept
        : context_(std::move(context)) {}

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

    bool valid() const noexcept {
        return context_.valid();
    }

    Context &context() noexcept {
        return context_;
    }

    const Context &context() const noexcept {
        return context_;
    }

    ::goud_context raw() const noexcept {
        return context_.raw();
    }

    bool shouldClose() const noexcept {
        return context_.shouldClose();
    }

    float pollEvents() noexcept {
        return context_.pollEvents();
    }

    void swapBuffers() noexcept {
        context_.swapBuffers();
    }

    void enableBlending() noexcept {
        context_.enableBlending();
    }

    float deltaTime() const noexcept {
        return context_.deltaTime();
    }

private:
    Context context_;
};

}  // namespace goud

#endif
