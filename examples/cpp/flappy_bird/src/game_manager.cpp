#include "game_manager.hpp"

#include <string>

namespace flappy {

// Relative path from the build directory to the shared sprite assets.
// Expected working directory: examples/cpp/flappy_bird/build/
// (run the binary from the build/ subdirectory created by cmake)
static const std::string ASSET_BASE = "../../../csharp/flappy_goud/assets/sprites/";

static goud_texture loadTex(goud::Context& ctx, const std::string& file) {
    goud_texture tex = 0;
    std::string path = ASSET_BASE + file;
    ctx.loadTexture(path.c_str(), tex);
    return tex;
}

void GameManager::init(goud::Context& ctx) {
    backgroundTex_ = loadTex(ctx, "background-day.png");
    baseTex_       = loadTex(ctx, "base.png");
    pipeTex_       = loadTex(ctx, "pipe-green.png");

    birdFrames_[0] = loadTex(ctx, "bluebird-downflap.png");
    birdFrames_[1] = loadTex(ctx, "bluebird-midflap.png");
    birdFrames_[2] = loadTex(ctx, "bluebird-upflap.png");

    for (int i = 0; i < 10; ++i) {
        digitTex_[i] = loadTex(ctx, std::to_string(i) + ".png");
    }

    reset();
}

void GameManager::reset() {
    bird_.reset();
    pipes_.clear();
    score_.reset();
    pipeSpawnTimer_ = 0.0f;
    state_ = GameState::WAITING;
}

bool GameManager::handleInput(goud::Context& ctx) {
    // Escape quits.
    if (ctx.keyDown(static_cast<goud_key>(KEY_ESCAPE))) {
        return false;
    }

    // R restarts.
    if (ctx.keyJustPressed(static_cast<goud_key>(KEY_R))) {
        reset();
    }

    return true;
}

void GameManager::update(goud::Context& ctx, float dt) {
    // Determine jump input.
    bool jumpInput = ctx.keyDown(static_cast<goud_key>(KEY_SPACE))
                  || ctx.mouseDown(static_cast<goud_mouse_button>(MOUSE_BUTTON_LEFT));

    if (state_ == GameState::WAITING) {
        if (jumpInput) {
            state_ = GameState::PLAYING;
        }
        return;
    }

    if (state_ == GameState::GAME_OVER) {
        return;
    }

    // -- PLAYING state --------------------------------------------------------

    // Update bird (physics + animation + jump intent).
    bird_.update(dt, jumpInput);

    // Ground / ceiling collision.
    if (bird_.y() + BIRD_HEIGHT > SCREEN_HEIGHT || bird_.y() < 0.0f) {
        reset();
        return;
    }

    // Update pipes.
    for (auto& pipe : pipes_) {
        if (checkCollision(pipe, bird_.getBounds())) {
            reset();
            return;
        }
    }

    // Move pipes and score for removed (off-screen) ones.
    int removed = updatePipes(pipes_, dt);
    for (int i = 0; i < removed; ++i) {
        score_.increment();
    }

    // Spawn new pipes.
    pipeSpawnTimer_ += dt;
    if (pipeSpawnTimer_ > PIPE_SPAWN_INTERVAL) {
        pipeSpawnTimer_ = 0.0f;
        pipes_.push_back(createPipe());
    }
}

void GameManager::draw(goud::Context& ctx) {
    goud_color white{ 1.0f, 1.0f, 1.0f, 1.0f };

    // Layer 0: Background
    ctx.drawSprite(
        backgroundTex_,
        BACKGROUND_WIDTH / 2.0f,
        BACKGROUND_HEIGHT / 2.0f,
        BACKGROUND_WIDTH,
        BACKGROUND_HEIGHT,
        0.0f,
        white
    );

    // Layer 1: Score (behind pipes, in front of background)
    score_.draw(ctx, digitTex_);

    // Layer 2: Pipes
    for (const auto& pipe : pipes_) {
        drawPipe(ctx, pipe, pipeTex_);
    }

    // Layer 3: Bird
    goud_texture birdTex = birdFrames_[bird_.animFrame()];
    bird_.draw(ctx, birdTex);

    // Layer 4: Base / ground (in front of everything)
    ctx.drawSprite(
        baseTex_,
        BASE_SPRITE_WIDTH / 2.0f,
        SCREEN_HEIGHT + BASE_HEIGHT / 2.0f,
        BASE_SPRITE_WIDTH,
        BASE_HEIGHT,
        0.0f,
        white
    );
}

}  // namespace flappy
