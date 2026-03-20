#ifndef FLAPPY_PIPE_H
#define FLAPPY_PIPE_H

#include <stdbool.h>
#include <goud/goud.h>
#include "constants.h"

#define MAX_PIPES 16

/* A pair of pipes (top and bottom) forming an obstacle. */
typedef struct {
    float x;
    float gap_y;
    bool  scored;
} Pipe;

/* Create a new pipe pair at the right edge of the screen with a random gap. */
Pipe pipe_create(void);

/* Move all pipes leftward and remove those that have scrolled off-screen.
 * Returns the number of pipes removed (for scoring). */
int pipes_update(Pipe *pipes, int *count, float dt);

/* AABB collision test between a pipe pair and bird bounds. */
bool pipe_check_collision(const Pipe *pipe, float bx, float by, float bw, float bh);

/* Draw a single pipe pair (top pipe flipped, bottom pipe normal). */
void pipe_draw(goud_context ctx, const Pipe *pipe, goud_texture pipe_tex);

#endif
