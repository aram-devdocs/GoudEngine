#ifndef GOUD_C_SDK_H
#define GOUD_C_SDK_H

/** @file goud.h
 *  @brief C SDK convenience layer for GoudEngine.
 *
 *  Provides type aliases, status-code helpers, and inline wrapper functions
 *  over the raw FFI exports declared in goud_engine.h.  All functions return
 *  an integer status code (0 = success) unless documented otherwise.
 */

#include "../goud_engine.h"

#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================= */
/** @defgroup types Type Aliases
 *  Convenience typedefs over raw FFI handles and value types.
 *  @{ */
/* ========================================================================= */

/** @brief Opaque engine context identifier. */
typedef GoudContextId goud_context;

/** @brief Opaque engine configuration handle (pointer). */
typedef EngineConfigHandle goud_engine_config;

/** @brief Entity identifier (generational handle stored as uint64). */
typedef uint64_t goud_entity;

/** @brief Texture asset handle. */
typedef GoudTextureHandle goud_texture;

/** @brief Font asset handle. */
typedef GoudFontHandle goud_font;

/** @brief Audio player identifier. Negative values indicate invalid state. */
typedef int64_t goud_audio_player;

/** @brief Keyboard key code. */
typedef GoudKeyCode goud_key;

/** @brief Mouse button identifier. */
typedef GoudMouseButton goud_mouse_button;

/** @brief RGBA colour (each channel 0.0 -- 1.0). */
typedef FfiColor goud_color;

/** @brief 2-component float vector (x, y). */
typedef FfiVec2 goud_vec2;

/** @brief Per-frame rendering statistics. */
typedef GoudRenderStats goud_render_stats;

/** @} */ /* end types */

/* ========================================================================= */
/** @defgroup error_handling Error Handling
 *  Structured error retrieval and status-code helpers.
 *  @{ */
/* ========================================================================= */

/** @brief Snapshot of the last error raised by the engine.
 *
 *  Populated by goud_get_last_error().  Field sizes are fixed so the struct
 *  can live on the stack without allocation.
 */
typedef struct goud_error_info {
    GoudErrorCode code;        /**< @brief Numeric error code. */
    int32_t recovery_class;    /**< @brief Recovery hint (0 = unrecoverable). */
    char message[256];         /**< @brief Human-readable error description. */
    char subsystem[64];        /**< @brief Engine subsystem that raised the error. */
    char operation[64];        /**< @brief Operation that failed. */
} goud_error_info;

/** @brief Return the last FFI error code, or @p fallback_code when no error is set.
 *  @param fallback_code  Value returned when the last error code is SUCCESS.
 *  @return The last error code, or @p fallback_code.
 */
static inline int goud_status_last_error_or(int fallback_code) {
    GoudErrorCode code = goud_last_error_code();
    return code == SUCCESS ? fallback_code : (int)code;
}

/** @brief Convert a boolean success flag to a status code.
 *  @param ok  true on success.
 *  @return SUCCESS (0) when @p ok is true; otherwise the last error code.
 */
static inline int goud_status_from_bool(bool ok) {
    return ok ? SUCCESS : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

/** @brief Convert a GoudResult to a status code.
 *  @param result  FFI result value.
 *  @return SUCCESS (0) on success; otherwise the embedded error code.
 */
static inline int goud_status_from_result(GoudResult result) {
    return result.success ? SUCCESS : (int)result.code;
}

/** @brief Convert a context handle to a status code.
 *  @param context  Context returned by a creation call.
 *  @return SUCCESS when valid; ERR_INVALID_CONTEXT otherwise.
 */
static inline int goud_status_from_context(goud_context context) {
    return context._0 == GOUD_INVALID_CONTEXT_ID._0
        ? goud_status_last_error_or(ERR_INVALID_CONTEXT)
        : SUCCESS;
}

/** @brief Convert a generic uint64 handle to a status code.
 *  @param handle          Handle value to check.
 *  @param invalid_handle  Sentinel value representing an invalid handle.
 *  @return SUCCESS when @p handle differs from @p invalid_handle.
 */
static inline int goud_status_from_handle(uint64_t handle, uint64_t invalid_handle) {
    return handle == invalid_handle
        ? goud_status_last_error_or(ERR_INVALID_HANDLE)
        : SUCCESS;
}

/** @brief Zero-fill a goud_error_info struct.
 *  @param out_error  Pointer to the struct to clear (may be NULL).
 */
static inline void goud_error_info_clear(goud_error_info *out_error) {
    if (out_error != NULL) {
        memset(out_error, 0, sizeof(*out_error));
    }
}

/** @brief Populate @p out_error with the last engine error.
 *  @param[out] out_error  Destination struct.
 *  @return The error code stored in @p out_error.
 *  @retval ERR_INVALID_STATE  @p out_error is NULL.
 */
static inline int goud_get_last_error(goud_error_info *out_error) {
    if (out_error == NULL) {
        return ERR_INVALID_STATE;
    }

    goud_error_info_clear(out_error);
    out_error->code = goud_last_error_code();
    out_error->recovery_class = goud_error_recovery_class(out_error->code);
    (void)goud_last_error_message((uint8_t *)out_error->message, sizeof(out_error->message));
    (void)goud_last_error_subsystem((uint8_t *)out_error->subsystem, sizeof(out_error->subsystem));
    (void)goud_last_error_operation((uint8_t *)out_error->operation, sizeof(out_error->operation));
    return (int)out_error->code;
}

/** @} */ /* end error_handling */

/* ========================================================================= */
/** @defgroup context_config Context and Engine Configuration
 *  Create, configure, and destroy engine contexts and configurations.
 *  @{ */
/* ========================================================================= */

/** @brief Return the sentinel value representing an invalid context.
 *  @return Invalid context handle.
 */
static inline goud_context goud_context_invalid(void) {
    return GOUD_INVALID_CONTEXT_ID;
}

/** @brief Test whether a context handle is valid.
 *  @param context  Handle to test.
 *  @return true when @p context is not the invalid sentinel.
 */
static inline bool goud_context_valid(goud_context context) {
    return context._0 != GOUD_INVALID_CONTEXT_ID._0;
}

/** @brief Test whether an engine configuration handle is valid.
 *  @param config  Handle to test.
 *  @return true when @p config is non-NULL.
 */
static inline bool goud_engine_config_valid(goud_engine_config config) {
    return config != NULL;
}

/** @brief Create a new engine context.
 *  @param[out] out_context  Receives the new context handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p out_context is NULL.
 */
static inline int goud_context_init(goud_context *out_context) {
    if (out_context == NULL) {
        return ERR_INVALID_STATE;
    }

    *out_context = goud_context_create();
    return goud_status_from_context(*out_context);
}

/** @brief Destroy an engine context and reset the handle to invalid.
 *  @param[in,out] context  Pointer to the context handle.  Set to invalid on success.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p context is NULL.
 */
static inline int goud_context_dispose(goud_context *context) {
    if (context == NULL) {
        return ERR_INVALID_STATE;
    }
    if (!goud_context_valid(*context)) {
        return SUCCESS;
    }

    int status = goud_status_from_bool(goud_context_destroy(*context));
    if (status == SUCCESS) {
        *context = goud_context_invalid();
    }
    return status;
}

/** @brief Allocate a new engine configuration.
 *  @param[out] out_config  Receives the new config handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p out_config is NULL.
 */
static inline int goud_engine_config_init(goud_engine_config *out_config) {
    if (out_config == NULL) {
        return ERR_INVALID_STATE;
    }

    *out_config = goud_engine_config_create();
    return *out_config == NULL ? goud_status_last_error_or(ERR_INVALID_HANDLE) : SUCCESS;
}

/** @brief Free an engine configuration and NULL the handle.
 *  @param[in,out] config  Pointer to the config handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_dispose(goud_engine_config *config) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    if (*config == NULL) {
        return SUCCESS;
    }

    goud_engine_config_destroy(*config);
    *config = NULL;
    return SUCCESS;
}

/** @brief Set the window title.
 *  @param config  Valid config handle.
 *  @param title   Null-terminated UTF-8 string.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_title_utf8(goud_engine_config config, const char *title) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_title(config, title));
}

/** @brief Set the window dimensions.
 *  @param config  Valid config handle.
 *  @param width   Window width in pixels.
 *  @param height  Window height in pixels.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_window_size(
    goud_engine_config config,
    uint32_t width,
    uint32_t height
) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_size(config, width, height));
}

/** @brief Enable or disable vertical sync.
 *  @param config   Valid config handle.
 *  @param enabled  true to enable vsync.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_vsync_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_vsync(config, enabled));
}

/** @brief Enable or disable fullscreen mode.
 *  @param config   Valid config handle.
 *  @param enabled  true to enable fullscreen.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_fullscreen_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_fullscreen(config, enabled));
}

/** @brief Set the target frames per second.
 *  @param config  Valid config handle.
 *  @param fps     Target FPS (0 = unlimited).
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_target_fps_value(goud_engine_config config, uint32_t fps) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_target_fps(config, fps));
}

/** @brief Enable or disable the FPS debug overlay.
 *  @param config   Valid config handle.
 *  @param enabled  true to show the overlay.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_fps_overlay_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_fps_overlay(config, enabled));
}

/** @brief Enable or disable physics debug rendering.
 *  @param config   Valid config handle.
 *  @param enabled  true to enable physics debug visualization.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_physics_debug_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_physics_debug(config, enabled));
}

/** @brief Select the 2D physics backend.
 *  @param config   Valid config handle.
 *  @param backend  Backend identifier (engine-defined enum value).
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_physics_backend_2d_value(goud_engine_config config, uint32_t backend) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_physics_backend_2d(config, backend));
}

/** @brief Select the rendering backend.
 *  @param config   Valid config handle.
 *  @param backend  Backend identifier (engine-defined enum value).
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_render_backend_value(goud_engine_config config, uint32_t backend) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_render_backend(config, backend));
}

/** @brief Select the window backend.
 *  @param config   Valid config handle.
 *  @param backend  Backend identifier (engine-defined enum value).
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config is NULL.
 */
static inline int goud_engine_config_set_window_backend_value(goud_engine_config config, uint32_t backend) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_window_backend(config, backend));
}

/** @brief Attach a debugger configuration to the engine config.
 *  @param config    Valid config handle.
 *  @param debugger  Pointer to debugger configuration.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config or @p debugger is NULL.
 */
static inline int goud_engine_config_set_debugger_config(
    goud_engine_config config,
    const GoudDebuggerConfig *debugger
) {
    if (config == NULL || debugger == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_debugger(config, debugger));
}

/** @brief Create an engine from a configuration and obtain a context.
 *
 *  The config handle is consumed whether creation succeeds or fails.
 *  After this call the caller's config pointer is set to NULL.
 *
 *  @param[in,out] config       Pointer to the config handle (consumed).
 *  @param[out]    out_context  Receives the new context handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p config, @p out_context, or *config is NULL.
 */
static inline int goud_engine_create_checked(goud_engine_config *config, goud_context *out_context) {
    goud_context context;

    if (config == NULL || out_context == NULL || *config == NULL) {
        return ERR_INVALID_STATE;
    }

    context = goud_engine_create(*config);
    *config = NULL;
    *out_context = context;
    return goud_status_from_context(context);
}

/** @} */ /* end context_config */

/* ========================================================================= */
/** @defgroup ecs Entity Component System
 *  Spawn, destroy, and query entities.
 *  @{ */
/* ========================================================================= */

/** @brief Spawn an empty entity.
 *  @param context            Valid engine context.
 *  @param[out] out_entity    Receives the new entity handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p out_entity is NULL.
 */
static inline int goud_entity_spawn(goud_context context, goud_entity *out_entity) {
    goud_entity entity;

    if (out_entity == NULL) {
        return ERR_INVALID_STATE;
    }

    entity = goud_entity_spawn_empty(context);
    *out_entity = entity;
    return goud_status_from_handle(entity, GOUD_INVALID_ENTITY_ID);
}

/** @brief Despawn an entity and clean up its components.
 *  @param context  Valid engine context.
 *  @param entity   Entity to despawn.
 *  @return SUCCESS on success.
 */
static inline int goud_entity_remove(goud_context context, goud_entity entity) {
    return goud_status_from_result(goud_entity_despawn(context, entity));
}

/** @brief Check whether an entity is still alive.
 *  @param context  Valid engine context.
 *  @param entity   Entity to query.
 *  @return true if the entity exists.
 */
static inline bool goud_entity_alive(goud_context context, goud_entity entity) {
    return goud_entity_is_alive(context, entity);
}

/** @} */ /* end ecs */

/* ========================================================================= */
/** @defgroup assets Assets
 *  Load and destroy texture and font assets.
 *  @{ */
/* ========================================================================= */

/** @brief Load a texture from a file path.
 *  @param context              Valid engine context.
 *  @param path                 Null-terminated file path.
 *  @param[out] out_texture     Receives the texture handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE   @p out_texture is NULL.
 */
static inline int goud_texture_load_path(goud_context context, const char *path, goud_texture *out_texture) {
    goud_texture texture;

    if (out_texture == NULL) {
        return ERR_INVALID_STATE;
    }

    texture = goud_texture_load(context, path);
    *out_texture = texture;
    return goud_status_from_handle(texture, UINT64_MAX);
}

/** @brief Destroy a texture.
 *  @param context  Valid engine context.
 *  @param texture  Texture handle to destroy.
 *  @return SUCCESS on success.
 */
static inline int goud_texture_dispose(goud_context context, goud_texture texture) {
    return goud_status_from_bool(goud_texture_destroy(context, texture));
}

/** @brief Load a font from a file path.
 *  @param context          Valid engine context.
 *  @param path             Null-terminated file path.
 *  @param[out] out_font    Receives the font handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p out_font is NULL.
 */
static inline int goud_font_load_path(goud_context context, const char *path, goud_font *out_font) {
    goud_font font;

    if (out_font == NULL) {
        return ERR_INVALID_STATE;
    }

    font = goud_font_load(context, path);
    *out_font = font;
    return goud_status_from_handle(font, UINT64_MAX);
}

/** @brief Destroy a font.
 *  @param context  Valid engine context.
 *  @param font     Font handle to destroy.
 *  @return SUCCESS on success.
 */
static inline int goud_font_dispose(goud_context context, goud_font font) {
    return goud_status_from_bool(goud_font_destroy(context, font));
}

/** @} */ /* end assets */

/* ========================================================================= */
/** @defgroup renderer Renderer
 *  Frame management, clear, and draw calls.
 *  @{ */
/* ========================================================================= */

/** @brief Begin a new render frame.
 *  @param context  Valid engine context.
 *  @return SUCCESS on success.
 */
static inline int goud_renderer_begin_frame(goud_context context) {
    return goud_status_from_bool(goud_renderer_begin(context));
}

/** @brief End the current render frame.
 *  @param context  Valid engine context.
 *  @return SUCCESS on success.
 */
static inline int goud_renderer_end_frame(goud_context context) {
    return goud_status_from_bool(goud_renderer_end(context));
}

/** @brief Clear the framebuffer with a solid colour.
 *  @param context  Valid engine context.
 *  @param color    Clear colour.
 */
static inline void goud_renderer_clear_color(goud_context context, goud_color color) {
    goud_window_clear(context, color.r, color.g, color.b, color.a);
}

/** @brief Draw a textured sprite.
 *  @param context   Valid engine context.
 *  @param texture   Texture handle.
 *  @param x         X position.
 *  @param y         Y position.
 *  @param width     Sprite width.
 *  @param height    Sprite height.
 *  @param rotation  Rotation in radians.
 *  @param color     Tint colour.
 *  @return SUCCESS on success.
 */
static inline int goud_renderer_draw_sprite_color(
    goud_context context,
    goud_texture texture,
    float x,
    float y,
    float width,
    float height,
    float rotation,
    goud_color color
) {
    return goud_status_from_bool(
        goud_renderer_draw_sprite(
            context,
            texture,
            x,
            y,
            width,
            height,
            rotation,
            color.r,
            color.g,
            color.b,
            color.a
        )
    );
}

/** @brief Draw a solid-colour quad.
 *  @param context  Valid engine context.
 *  @param x        X position.
 *  @param y        Y position.
 *  @param width    Quad width.
 *  @param height   Quad height.
 *  @param color    Fill colour.
 *  @return SUCCESS on success.
 */
static inline int goud_renderer_draw_quad_color(
    goud_context context,
    float x,
    float y,
    float width,
    float height,
    goud_color color
) {
    return goud_status_from_bool(
        goud_renderer_draw_quad(context, x, y, width, height, color.r, color.g, color.b, color.a)
    );
}

/** @brief Retrieve per-frame render statistics.
 *  @param context            Valid engine context.
 *  @param[out] out_stats     Receives the stats struct.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p out_stats is NULL.
 */
static inline int goud_renderer_stats(goud_context context, goud_render_stats *out_stats) {
    if (out_stats == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_renderer_get_stats(context, out_stats));
}

/** @} */ /* end renderer */

/* ========================================================================= */
/** @defgroup input Input
 *  Keyboard and mouse state queries.
 *  @{ */
/* ========================================================================= */

/** @brief Test whether a key is currently held down.
 *  @param context  Valid engine context.
 *  @param key      Key code.
 *  @return true if the key is pressed.
 */
static inline bool goud_input_key_down(goud_context context, goud_key key) {
    return goud_input_key_pressed(context, key);
}

/** @brief Test whether a key was pressed this frame (edge trigger).
 *  @param context  Valid engine context.
 *  @param key      Key code.
 *  @return true if the key was just pressed.
 */
static inline bool goud_input_key_pressed_once(goud_context context, goud_key key) {
    return goud_input_key_just_pressed(context, key);
}

/** @brief Test whether a mouse button is currently held down.
 *  @param context  Valid engine context.
 *  @param button   Mouse button.
 *  @return true if the button is pressed.
 */
static inline bool goud_input_mouse_down(goud_context context, goud_mouse_button button) {
    return goud_input_mouse_button_pressed(context, button);
}

/** @brief Get the current mouse cursor position.
 *  @param context              Valid engine context.
 *  @param[out] out_position    Receives the (x, y) position.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE   @p out_position is NULL.
 */
static inline int goud_input_mouse_position(goud_context context, goud_vec2 *out_position) {
    if (out_position == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_input_get_mouse_position(context, &out_position->x, &out_position->y));
}

/** @brief Get the mouse scroll delta for this frame.
 *  @param context          Valid engine context.
 *  @param[out] out_delta   Receives the (x, y) scroll delta.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE  @p out_delta is NULL.
 */
static inline int goud_input_scroll_delta(goud_context context, goud_vec2 *out_delta) {
    if (out_delta == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_input_get_scroll_delta(context, &out_delta->x, &out_delta->y));
}

/** @} */ /* end input */

/* ========================================================================= */
/** @defgroup window Window / Game Loop
 *  Window lifecycle, event polling, and frame timing.
 *  @{ */
/* ========================================================================= */

/** @brief Check whether the window close has been requested.
 *  @param context  Valid engine context.
 *  @return true if the window should close.  Returns true for invalid contexts.
 */
static inline bool goud_window_should_close_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return true;
    }
    return goud_window_should_close(context);
}

/** @brief Poll window events and return the frame delta time.
 *  @param context  Valid engine context.
 *  @return Delta time in seconds.  Returns 0 for invalid contexts.
 */
static inline float goud_window_poll_events_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return 0.0f;
    }
    return goud_window_poll_events(context);
}

/** @brief Swap the front and back framebuffers.
 *  @param context  Valid engine context.  No-op for invalid contexts.
 */
static inline void goud_window_swap_buffers_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return;
    }
    goud_window_swap_buffers(context);
}

/** @brief Enable alpha blending for the renderer.
 *  @param context  Valid engine context.  No-op for invalid contexts.
 */
static inline void goud_renderer_enable_blending_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return;
    }
    goud_renderer_enable_blending(context);
}

/** @brief Get the delta time for the current frame.
 *  @param context  Valid engine context.
 *  @return Delta time in seconds.  Returns 0 for invalid contexts.
 */
static inline float goud_window_delta_time(goud_context context) {
    if (!goud_context_valid(context)) {
        return 0.0f;
    }
    return goud_window_get_delta_time(context);
}

/** @} */ /* end window */

/* ========================================================================= */
/** @defgroup audio Audio
 *  Audio playback and volume control.
 *  @{ */
/* ========================================================================= */

/** @brief Activate the audio subsystem.
 *  @param context  Valid engine context.
 *  @return Non-negative value on success, negative error code on failure.
 */
static inline int goud_audio_activate_checked(goud_context context) {
    int status = goud_audio_activate(context);
    return status >= 0 ? status : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

/** @brief Play audio from an in-memory buffer.
 *  @param context              Valid engine context.
 *  @param asset_data           Pointer to audio data.
 *  @param asset_len            Length of @p asset_data in bytes.
 *  @param[out] out_player      Receives the player handle.
 *  @return SUCCESS on success.
 *  @retval ERR_INVALID_STATE   @p asset_data or @p out_player is NULL.
 */
static inline int goud_audio_play_memory(
    goud_context context,
    const void *asset_data,
    size_t asset_len,
    goud_audio_player *out_player
) {
    goud_audio_player player;

    if (asset_data == NULL || out_player == NULL) {
        return ERR_INVALID_STATE;
    }

    player = goud_audio_play(context, (const uint8_t *)asset_data, asset_len);
    *out_player = player;
    return player >= 0 ? SUCCESS : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

/** @brief Stop an active audio player.
 *  @param context    Valid engine context.
 *  @param player_id  Player handle returned by goud_audio_play_memory().
 *  @return Non-negative value on success, negative error code on failure.
 *  @retval ERR_INVALID_STATE  @p player_id is negative.
 */
static inline int goud_audio_stop_checked(goud_context context, goud_audio_player player_id) {
    if (player_id < 0) {
        return ERR_INVALID_STATE;
    }
    int status = goud_audio_stop(context, (uint64_t)player_id);
    return status >= 0 ? status : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

/** @brief Set the global audio volume.
 *  @param context  Valid engine context.
 *  @param volume   Volume level (0.0 = mute, 1.0 = full).
 *  @return Non-negative value on success, negative error code on failure.
 */
static inline int goud_audio_set_global_volume_checked(goud_context context, float volume) {
    int status = goud_audio_set_global_volume(context, volume);
    return status >= 0 ? status : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

/** @} */ /* end audio */

#ifdef __cplusplus
}  /* extern "C" */
#endif

#endif
