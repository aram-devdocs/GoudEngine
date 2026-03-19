#ifndef GOUD_C_SDK_H
#define GOUD_C_SDK_H

#include "../goud_engine.h"

#include <string.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef GoudContextId goud_context;
typedef EngineConfigHandle goud_engine_config;
typedef uint64_t goud_entity;
typedef GoudTextureHandle goud_texture;
typedef GoudFontHandle goud_font;
typedef int64_t goud_audio_player;
typedef GoudKeyCode goud_key;
typedef GoudMouseButton goud_mouse_button;
typedef FfiColor goud_color;
typedef FfiVec2 goud_vec2;
typedef GoudRenderStats goud_render_stats;

typedef struct goud_error_info {
    GoudErrorCode code;
    int32_t recovery_class;
    char message[256];
    char subsystem[64];
    char operation[64];
} goud_error_info;

static inline int goud_status_last_error_or(int fallback_code) {
    GoudErrorCode code = goud_last_error_code();
    return code == SUCCESS ? fallback_code : (int)code;
}

static inline int goud_status_from_bool(bool ok) {
    return ok ? SUCCESS : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

static inline int goud_status_from_result(GoudResult result) {
    return result.success ? SUCCESS : (int)result.code;
}

static inline int goud_status_from_context(goud_context context) {
    return context._0 == GOUD_INVALID_CONTEXT_ID._0
        ? goud_status_last_error_or(ERR_INVALID_CONTEXT)
        : SUCCESS;
}

static inline int goud_status_from_handle(uint64_t handle, uint64_t invalid_handle) {
    return handle == invalid_handle
        ? goud_status_last_error_or(ERR_INVALID_HANDLE)
        : SUCCESS;
}

static inline goud_context goud_context_invalid(void) {
    return GOUD_INVALID_CONTEXT_ID;
}

static inline bool goud_context_valid(goud_context context) {
    return context._0 != GOUD_INVALID_CONTEXT_ID._0;
}

static inline bool goud_engine_config_valid(goud_engine_config config) {
    return config != NULL;
}

static inline void goud_error_info_clear(goud_error_info *out_error) {
    if (out_error != NULL) {
        memset(out_error, 0, sizeof(*out_error));
    }
}

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

/* === Context and Engine Config === */

static inline int goud_context_init(goud_context *out_context) {
    if (out_context == NULL) {
        return ERR_INVALID_STATE;
    }

    *out_context = goud_context_create();
    return goud_status_from_context(*out_context);
}

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

static inline int goud_engine_config_init(goud_engine_config *out_config) {
    if (out_config == NULL) {
        return ERR_INVALID_STATE;
    }

    *out_config = goud_engine_config_create();
    return *out_config == NULL ? goud_status_last_error_or(ERR_INVALID_HANDLE) : SUCCESS;
}

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

static inline int goud_engine_config_set_title_utf8(goud_engine_config config, const char *title) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_title(config, title));
}

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

static inline int goud_engine_config_set_vsync_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_vsync(config, enabled));
}

static inline int goud_engine_config_set_fullscreen_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_fullscreen(config, enabled));
}

static inline int goud_engine_config_set_target_fps_value(goud_engine_config config, uint32_t fps) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_target_fps(config, fps));
}

static inline int goud_engine_config_set_fps_overlay_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_fps_overlay(config, enabled));
}

static inline int goud_engine_config_set_physics_debug_enabled(goud_engine_config config, bool enabled) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_physics_debug(config, enabled));
}

static inline int goud_engine_config_set_physics_backend_2d_value(goud_engine_config config, uint32_t backend) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_physics_backend_2d(config, backend));
}

static inline int goud_engine_config_set_render_backend_value(goud_engine_config config, uint32_t backend) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_render_backend(config, backend));
}

static inline int goud_engine_config_set_window_backend_value(goud_engine_config config, uint32_t backend) {
    if (config == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_window_backend(config, backend));
}

static inline int goud_engine_config_set_debugger_config(
    goud_engine_config config,
    const GoudDebuggerConfig *debugger
) {
    if (config == NULL || debugger == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_engine_config_set_debugger(config, debugger));
}

/*
 * `goud_engine_create` consumes the config handle whether creation succeeds or fails.
 * After this call the wrapper clears the caller-owned config pointer.
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

/* === ECS === */

static inline int goud_entity_spawn(goud_context context, goud_entity *out_entity) {
    goud_entity entity;

    if (out_entity == NULL) {
        return ERR_INVALID_STATE;
    }

    entity = goud_entity_spawn_empty(context);
    *out_entity = entity;
    return goud_status_from_handle(entity, GOUD_INVALID_ENTITY_ID);
}

static inline int goud_entity_remove(goud_context context, goud_entity entity) {
    return goud_status_from_result(goud_entity_despawn(context, entity));
}

static inline bool goud_entity_alive(goud_context context, goud_entity entity) {
    return goud_entity_is_alive(context, entity);
}

/* === Assets === */

static inline int goud_texture_load_path(goud_context context, const char *path, goud_texture *out_texture) {
    goud_texture texture;

    if (out_texture == NULL) {
        return ERR_INVALID_STATE;
    }

    texture = goud_texture_load(context, path);
    *out_texture = texture;
    return goud_status_from_handle(texture, UINT64_MAX);
}

static inline int goud_texture_dispose(goud_context context, goud_texture texture) {
    return goud_status_from_bool(goud_texture_destroy(context, texture));
}

static inline int goud_font_load_path(goud_context context, const char *path, goud_font *out_font) {
    goud_font font;

    if (out_font == NULL) {
        return ERR_INVALID_STATE;
    }

    font = goud_font_load(context, path);
    *out_font = font;
    return goud_status_from_handle(font, UINT64_MAX);
}

static inline int goud_font_dispose(goud_context context, goud_font font) {
    return goud_status_from_bool(goud_font_destroy(context, font));
}

/* === Renderer === */

static inline int goud_renderer_begin_frame(goud_context context) {
    return goud_status_from_bool(goud_renderer_begin(context));
}

static inline int goud_renderer_end_frame(goud_context context) {
    return goud_status_from_bool(goud_renderer_end(context));
}

static inline void goud_renderer_clear_color(goud_context context, goud_color color) {
    goud_window_clear(context, color.r, color.g, color.b, color.a);
}

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

static inline int goud_renderer_stats(goud_context context, goud_render_stats *out_stats) {
    if (out_stats == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_renderer_get_stats(context, out_stats));
}

/* === Input === */

static inline bool goud_input_key_down(goud_context context, goud_key key) {
    return goud_input_key_pressed(context, key);
}

static inline bool goud_input_key_pressed_once(goud_context context, goud_key key) {
    return goud_input_key_just_pressed(context, key);
}

static inline bool goud_input_mouse_down(goud_context context, goud_mouse_button button) {
    return goud_input_mouse_button_pressed(context, button);
}

static inline int goud_input_mouse_position(goud_context context, goud_vec2 *out_position) {
    if (out_position == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_input_get_mouse_position(context, &out_position->x, &out_position->y));
}

static inline int goud_input_scroll_delta(goud_context context, goud_vec2 *out_delta) {
    if (out_delta == NULL) {
        return ERR_INVALID_STATE;
    }
    return goud_status_from_bool(goud_input_get_scroll_delta(context, &out_delta->x, &out_delta->y));
}

/* === Window / Game Loop === */

static inline bool goud_window_should_close_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return true;
    }
    return goud_window_should_close(context);
}

static inline float goud_window_poll_events_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return 0.0f;
    }
    return goud_window_poll_events(context);
}

static inline void goud_window_swap_buffers_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return;
    }
    goud_window_swap_buffers(context);
}

static inline void goud_renderer_enable_blending_checked(goud_context context) {
    if (!goud_context_valid(context)) {
        return;
    }
    goud_renderer_enable_blending(context);
}

static inline float goud_window_delta_time(goud_context context) {
    if (!goud_context_valid(context)) {
        return 0.0f;
    }
    return goud_window_get_delta_time(context);
}

/* === Audio === */

static inline int goud_audio_activate_checked(goud_context context) {
    int status = goud_audio_activate(context);
    return status >= 0 ? status : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

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

static inline int goud_audio_stop_checked(goud_context context, goud_audio_player player_id) {
    if (player_id < 0) {
        return ERR_INVALID_STATE;
    }
    int status = goud_audio_stop(context, (uint64_t)player_id);
    return status >= 0 ? status : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

static inline int goud_audio_set_global_volume_checked(goud_context context, float volume) {
    int status = goud_audio_set_global_volume(context, volume);
    return status >= 0 ? status : goud_status_last_error_or(ERR_INTERNAL_ERROR);
}

#ifdef __cplusplus
}  /* extern "C" */
#endif

#endif
