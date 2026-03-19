#ifndef FLAPPY_SCORE_HPP
#define FLAPPY_SCORE_HPP

#include <goud/goud.hpp>
#include "constants.hpp"

namespace flappy {

/// Tracks the player's score and renders it as digit sprites.
class ScoreCounter {
public:
    ScoreCounter() = default;

    void increment() { score_++; }
    void reset()     { score_ = 0; }
    int  value() const { return score_; }

    /// Draw the score as centered digit sprites near the top of the screen.
    void draw(const goud::Context& ctx, goud_texture digitTextures[10]) const;

private:
    int score_ = 0;
};

}  // namespace flappy

#endif
