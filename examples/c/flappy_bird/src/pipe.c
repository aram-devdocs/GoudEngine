#include "pipe.h"
#include <stdlib.h>
#include <math.h>

static float top_pipe_y(const Pipe *p) {
    return p->gap_y - PIPE_GAP - PIPE_IMAGE_HEIGHT;
}

static float bottom_pipe_y(const Pipe *p) {
    return p->gap_y + PIPE_GAP;
}

static bool is_off_screen(const Pipe *p) {
    return p->x + PIPE_COLLISION_WIDTH < 0.0f;
}

static bool aabb_overlap(float x1, float y1, float w1, float h1,
                          float x2, float y2, float w2, float h2) {
    return x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
}

Pipe pipe_create(void) {
    /* Gap Y range matches the Rust implementation: PIPE_GAP .. SCREEN_HEIGHT - PIPE_GAP */
    int range_lo = (int)PIPE_GAP;
    int range_hi = (int)(SCREEN_HEIGHT - PIPE_GAP) - 1;
    int gap_y = range_lo + (rand() % (range_hi - range_lo + 1));

    Pipe p;
    p.x      = SCREEN_WIDTH;
    p.gap_y  = (float)gap_y;
    p.scored = false;
    return p;
}

int pipes_update(Pipe *pipes, int *count, float dt) {
    int i;
    int removed = 0;
    int write = 0;

    /* Move all pipes */
    for (i = 0; i < *count; ++i) {
        pipes[i].x -= PIPE_SPEED * dt * TARGET_FPS;
    }

    /* Compact: remove off-screen pipes */
    for (i = 0; i < *count; ++i) {
        if (is_off_screen(&pipes[i])) {
            ++removed;
        } else {
            pipes[write++] = pipes[i];
        }
    }
    *count = write;
    return removed;
}

bool pipe_check_collision(const Pipe *pipe, float bx, float by, float bw, float bh) {
    /* Top pipe */
    if (aabb_overlap(bx, by, bw, bh,
                     pipe->x, top_pipe_y(pipe), PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT)) {
        return true;
    }
    /* Bottom pipe */
    return aabb_overlap(bx, by, bw, bh,
                        pipe->x, bottom_pipe_y(pipe), PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT);
}

void pipe_draw(goud_context ctx, const Pipe *pipe, goud_texture pipe_tex) {
    goud_color white = { 1.0f, 1.0f, 1.0f, 1.0f };

    /* Top pipe (rotated 180 degrees) */
    goud_renderer_draw_sprite_color(
        ctx,
        pipe_tex,
        pipe->x + PIPE_IMAGE_WIDTH / 2.0f,
        top_pipe_y(pipe) + PIPE_IMAGE_HEIGHT / 2.0f,
        PIPE_IMAGE_WIDTH,
        PIPE_IMAGE_HEIGHT,
        (float)M_PI,
        white
    );

    /* Bottom pipe (no rotation) */
    goud_renderer_draw_sprite_color(
        ctx,
        pipe_tex,
        pipe->x + PIPE_IMAGE_WIDTH / 2.0f,
        bottom_pipe_y(pipe) + PIPE_IMAGE_HEIGHT / 2.0f,
        PIPE_IMAGE_WIDTH,
        PIPE_IMAGE_HEIGHT,
        0.0f,
        white
    );
}
