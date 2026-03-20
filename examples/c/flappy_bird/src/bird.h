#ifndef FLAPPY_BIRD_H
#define FLAPPY_BIRD_H

#include <stdbool.h>
#include <goud/goud.h>
#include "constants.h"

/* The player-controlled bird: movement physics plus animation state. */
typedef struct {
    float x;
    float y;
    float velocity;
    float rotation;
    float jump_cooldown;
    float anim_time;
    int   anim_frame;
} Bird;

/* Initialize bird at starting position. */
void bird_init(Bird *b);

/* Run one frame of physics: gravity, position, rotation, animation. */
void bird_update(Bird *b, float dt, bool jump_pressed);

/* Apply an upward impulse (if cooldown has elapsed). */
void bird_jump(Bird *b);

/* Whether the jump cooldown has elapsed. */
bool bird_can_jump(const Bird *b);

/* Draw the bird sprite at its current position and rotation. */
void bird_draw(const Bird *b, goud_context ctx, goud_texture tex);

/* Reset to starting position, velocity, and animation. */
void bird_reset(Bird *b);

#endif
