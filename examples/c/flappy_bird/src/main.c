/* Flappy Bird -- GoudEngine C Example
 *
 * A complete Flappy Bird clone demonstrating the GoudEngine from C.
 * Game constants and behavior match the C#, Python, TypeScript, C++, and Rust
 * versions exactly for SDK parity validation.
 *
 * Controls:
 *   Space / Left Click -- Flap (jump)
 *   R                  -- Restart
 *   Escape             -- Quit
 */

#include <goud/goud.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "constants.h"
#include "bird.h"
#include "pipe.h"
#include "score.h"

/* Game states */
typedef enum {
    STATE_WAITING,
    STATE_PLAYING,
    STATE_GAME_OVER
} GameState;

/* All game state in one struct */
typedef struct {
    Bird       bird;
    Pipe       pipes[MAX_PIPES];
    int        pipe_count;
    Score      score;
    GameState  state;
    float      pipe_spawn_timer;

    /* Textures */
    goud_texture background_tex;
    goud_texture base_tex;
    goud_texture pipe_tex;
    goud_texture bird_frames[3];
    goud_texture digit_tex[10];
} Game;

/* Relative path from the working directory (examples/c/flappy_bird/) to shared
 * sprite assets. */
static const char *ASSET_BASE = "../../../csharp/flappy_goud/assets/sprites/";

static goud_texture load_tex(goud_context ctx, const char *file) {
    char path[512];
    goud_texture tex = 0;
    snprintf(path, sizeof(path), "%s%s", ASSET_BASE, file);
    goud_texture_load_path(ctx, path, &tex);
    return tex;
}

static void game_init(Game *g, goud_context ctx) {
    int i;
    char file[16];

    g->background_tex = load_tex(ctx, "background-day.png");
    g->base_tex       = load_tex(ctx, "base.png");
    g->pipe_tex       = load_tex(ctx, "pipe-green.png");

    g->bird_frames[0] = load_tex(ctx, "bluebird-downflap.png");
    g->bird_frames[1] = load_tex(ctx, "bluebird-midflap.png");
    g->bird_frames[2] = load_tex(ctx, "bluebird-upflap.png");

    for (i = 0; i < 10; ++i) {
        snprintf(file, sizeof(file), "%d.png", i);
        g->digit_tex[i] = load_tex(ctx, file);
    }

    bird_init(&g->bird);
    g->pipe_count       = 0;
    g->score.value      = 0;
    g->state            = STATE_WAITING;
    g->pipe_spawn_timer = 0.0f;
}

static void game_reset(Game *g) {
    bird_reset(&g->bird);
    g->pipe_count       = 0;
    g->score.value      = 0;
    g->state            = STATE_WAITING;
    g->pipe_spawn_timer = 0.0f;
}

/* Returns false when the game should quit. */
static bool game_handle_input(Game *g, goud_context ctx) {
    /* Escape quits */
    if (goud_input_key_down(ctx, KEY_ESCAPE)) {
        return false;
    }

    /* R restarts */
    if (goud_input_key_pressed_once(ctx, KEY_R)) {
        game_reset(g);
    }

    return true;
}

static void game_update(Game *g, goud_context ctx, float dt) {
    int i;
    int removed;
    bool jump_input;

    jump_input = goud_input_key_down(ctx, KEY_SPACE)
              || goud_input_mouse_down(ctx, MOUSE_BUTTON_LEFT);

    if (g->state == STATE_WAITING) {
        if (jump_input) {
            g->state = STATE_PLAYING;
        }
        return;
    }

    if (g->state == STATE_GAME_OVER) {
        return;
    }

    /* -- PLAYING state ---------------------------------------------------- */

    /* Update bird (physics + animation + jump intent) */
    bird_update(&g->bird, dt, jump_input);

    /* Ground / ceiling collision */
    if (g->bird.y + BIRD_HEIGHT > SCREEN_HEIGHT || g->bird.y < 0.0f) {
        game_reset(g);
        return;
    }

    /* Pipe collision */
    for (i = 0; i < g->pipe_count; ++i) {
        if (pipe_check_collision(&g->pipes[i],
                                  g->bird.x, g->bird.y,
                                  BIRD_WIDTH, BIRD_HEIGHT)) {
            game_reset(g);
            return;
        }
    }

    /* Move pipes and score for removed (off-screen) ones */
    removed = pipes_update(g->pipes, &g->pipe_count, dt);
    for (i = 0; i < removed; ++i) {
        g->score.value++;
    }

    /* Spawn new pipes */
    g->pipe_spawn_timer += dt;
    if (g->pipe_spawn_timer > PIPE_SPAWN_INTERVAL) {
        g->pipe_spawn_timer = 0.0f;
        if (g->pipe_count < MAX_PIPES) {
            g->pipes[g->pipe_count++] = pipe_create();
        }
    }
}

static void game_draw(Game *g, goud_context ctx) {
    int i;
    goud_color white = { 1.0f, 1.0f, 1.0f, 1.0f };
    goud_texture bird_tex;

    /* Layer 0: Background */
    goud_renderer_draw_sprite_color(
        ctx, g->background_tex,
        BACKGROUND_WIDTH / 2.0f,
        BACKGROUND_HEIGHT / 2.0f,
        BACKGROUND_WIDTH,
        BACKGROUND_HEIGHT,
        0.0f, white
    );

    /* Layer 1: Score (behind pipes, in front of background) */
    score_draw(&g->score, ctx, g->digit_tex);

    /* Layer 2: Pipes */
    for (i = 0; i < g->pipe_count; ++i) {
        pipe_draw(ctx, &g->pipes[i], g->pipe_tex);
    }

    /* Layer 3: Bird */
    bird_tex = g->bird_frames[g->bird.anim_frame];
    bird_draw(&g->bird, ctx, bird_tex);

    /* Layer 4: Base / ground (in front of everything) */
    goud_renderer_draw_sprite_color(
        ctx, g->base_tex,
        BASE_SPRITE_WIDTH / 2.0f,
        SCREEN_HEIGHT + BASE_HEIGHT / 2.0f,
        BASE_SPRITE_WIDTH,
        BASE_HEIGHT,
        0.0f, white
    );
}

int main(void) {
    goud_engine_config config = NULL;
    goud_context ctx;
    Game game;
    goud_color sky_blue = { 0.4f, 0.7f, 0.9f, 1.0f };
    float dt;

    /* Seed the random number generator */
    srand((unsigned int)time(NULL));

    /* -- Engine setup ----------------------------------------------------- */
    if (goud_engine_config_init(&config) != SUCCESS) {
        fprintf(stderr, "Failed to create engine config\n");
        return 1;
    }
    goud_engine_config_set_title_utf8(config, "Flappy Bird C");
    goud_engine_config_set_window_size(config,
        (uint32_t)SCREEN_WIDTH,
        (uint32_t)(SCREEN_HEIGHT + BASE_HEIGHT));
    goud_engine_config_set_vsync_enabled(config, false);
    goud_engine_config_set_target_fps_value(config, (uint32_t)TARGET_FPS);

    if (goud_engine_create_checked(&config, &ctx) != SUCCESS) {
        fprintf(stderr, "Failed to create engine\n");
        return 1;
    }

    goud_renderer_enable_blending_checked(ctx);

    /* -- Game init -------------------------------------------------------- */
    game_init(&game, ctx);

    /* -- Game loop -------------------------------------------------------- */
    while (!goud_window_should_close_checked(ctx)) {
        dt = goud_window_poll_events_checked(ctx);
        if (dt <= 0.0f) {
            dt = 1.0f / TARGET_FPS;
        }

        if (!game_handle_input(&game, ctx)) {
            break; /* Escape pressed */
        }
        game_update(&game, ctx, dt);

        goud_renderer_begin_frame(ctx);
        goud_renderer_clear_color(ctx, sky_blue);
        game_draw(&game, ctx);
        goud_renderer_end_frame(ctx);
        goud_window_swap_buffers_checked(ctx);
    }

    return 0;
}
