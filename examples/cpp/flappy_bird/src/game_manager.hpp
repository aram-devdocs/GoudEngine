#ifndef FLAPPY_GAME_MANAGER_HPP
#define FLAPPY_GAME_MANAGER_HPP

#include <goud/goud.hpp>
#include <vector>

#include "bird.hpp"
#include "pipe.hpp"
#include "score.hpp"
#include "constants.hpp"

namespace flappy {

/// Game states.
enum class GameState {
    WAITING,
    PLAYING,
    GAME_OVER,
};

/// Orchestrates bird, pipes, score, textures, and game state.
class GameManager {
public:
    GameManager() = default;

    /// Load all textures and prepare the initial game state.
    void init(goud::Context& ctx);

    /// Process one frame of input. Returns false when the game should quit.
    bool handleInput(goud::Context& ctx);

    /// Advance game logic by dt seconds.
    void update(goud::Context& ctx, float dt);

    /// Render all layers (background, pipes, base, bird, score).
    void draw(goud::Context& ctx);

    /// Reset all game state for a new round.
    void reset();

private:
    Bird          bird_{ BIRD_START_X, BIRD_START_Y };
    std::vector<Pipe> pipes_;
    ScoreCounter  score_;
    GameState     state_ = GameState::WAITING;
    float         pipeSpawnTimer_ = 0.0f;

    // Textures
    goud_texture backgroundTex_ = 0;
    goud_texture baseTex_       = 0;
    goud_texture pipeTex_       = 0;
    goud_texture birdFrames_[3] = {};
    goud_texture digitTex_[10]  = {};
};

}  // namespace flappy

#endif
