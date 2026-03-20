#ifndef FLAPPY_SCORE_H
#define FLAPPY_SCORE_H

#include <goud/goud.h>
#include "constants.h"

/* Tracks the player's score. */
typedef struct {
    int value;
} Score;

/* Draw the score as centered digit sprites near the top of the screen. */
void score_draw(const Score *s, goud_context ctx, goud_texture digit_textures[10]);

#endif
