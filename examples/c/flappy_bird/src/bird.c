#include "bird.h"
#include <math.h>

/* Clamp a float to [lo, hi]. */
static float clampf(float v, float lo, float hi) {
    if (v < lo) return lo;
    if (v > hi) return hi;
    return v;
}

void bird_init(Bird *b) {
    b->x             = BIRD_START_X;
    b->y             = BIRD_START_Y;
    b->velocity      = 0.0f;
    b->rotation      = 0.0f;
    b->jump_cooldown = 0.0f;
    b->anim_time     = 0.0f;
    b->anim_frame    = 0;
}

void bird_update(Bird *b, float dt, bool jump_pressed) {
    if (jump_pressed) {
        bird_jump(b);
    }

    /* Gravity */
    b->velocity += GRAVITY * dt * TARGET_FPS;
    b->jump_cooldown -= dt;
    if (b->jump_cooldown < 0.0f) {
        b->jump_cooldown = 0.0f;
    }

    /* Position */
    b->y += b->velocity * dt;

    /* Rotation (smooth towards velocity-driven target) */
    float target_rotation = clampf(b->velocity * 3.0f, -45.0f, 45.0f);
    b->rotation += (target_rotation - b->rotation) * ROTATION_SMOOTHING;

    /* Animation */
    b->anim_time += dt;
    if (b->anim_time >= ANIMATION_FRAME_DURATION) {
        b->anim_frame = (b->anim_frame + 1) % 3;
        b->anim_time = 0.0f;
    }
}

void bird_jump(Bird *b) {
    if (bird_can_jump(b)) {
        b->velocity = JUMP_STRENGTH * TARGET_FPS;
        b->jump_cooldown = JUMP_COOLDOWN;
    }
}

bool bird_can_jump(const Bird *b) {
    return b->jump_cooldown <= 0.0f;
}

void bird_draw(const Bird *b, goud_context ctx, goud_texture tex) {
    float radians = b->rotation * ((float)M_PI / 180.0f);
    goud_color white = { 1.0f, 1.0f, 1.0f, 1.0f };
    goud_renderer_draw_sprite_color(
        ctx,
        tex,
        b->x + BIRD_WIDTH / 2.0f,
        b->y + BIRD_HEIGHT / 2.0f,
        BIRD_WIDTH,
        BIRD_HEIGHT,
        radians,
        white
    );
}

void bird_reset(Bird *b) {
    bird_init(b);
}
