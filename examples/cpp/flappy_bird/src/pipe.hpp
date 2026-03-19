#ifndef FLAPPY_PIPE_HPP
#define FLAPPY_PIPE_HPP

#include <goud/goud.hpp>
#include <vector>
#include "constants.hpp"
#include "bird.hpp"

namespace flappy {

/// A pair of pipes (top and bottom) forming an obstacle.
struct Pipe {
    float x;
    float gapY;
    bool  scored = false;
};

/// Create a new pipe pair at the right edge of the screen with a random gap.
Pipe createPipe();

/// Move all pipes leftward and remove those that have scrolled off-screen.
/// Returns the number of pipes removed (for scoring).
int updatePipes(std::vector<Pipe>& pipes, float dt);

/// AABB collision test between a pipe pair and bird bounds.
bool checkCollision(const Pipe& pipe, Bird::Bounds bounds);

/// Draw a single pipe pair (top pipe flipped, bottom pipe normal).
void drawPipe(const goud::Context& ctx, const Pipe& pipe,
              goud_texture pipeTex);

}  // namespace flappy

#endif
