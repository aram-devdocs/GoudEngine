#include "pipe.hpp"

#include <algorithm>
#include <cmath>
#include <random>

namespace flappy {

// Thread-local random engine seeded once.
static std::mt19937& rng() {
    static std::mt19937 gen(std::random_device{}());
    return gen;
}

Pipe createPipe() {
    // Gap Y range matches the Rust implementation: PIPE_GAP .. SCREEN_HEIGHT - PIPE_GAP
    std::uniform_int_distribution<int> dist(
        static_cast<int>(PIPE_GAP),
        static_cast<int>(SCREEN_HEIGHT - PIPE_GAP) - 1
    );

    Pipe p;
    p.x     = SCREEN_WIDTH;
    p.gapY  = static_cast<float>(dist(rng()));
    p.scored = false;
    return p;
}

static float topPipeY(const Pipe& p) {
    return p.gapY - PIPE_GAP - PIPE_IMAGE_HEIGHT;
}

static float bottomPipeY(const Pipe& p) {
    return p.gapY + PIPE_GAP;
}

static bool isOffScreen(const Pipe& p) {
    return p.x + PIPE_COLLISION_WIDTH < 0.0f;
}

static bool aabbOverlap(float x1, float y1, float w1, float h1,
                         float x2, float y2, float w2, float h2) {
    return x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
}

int updatePipes(std::vector<Pipe>& pipes, float dt) {
    for (auto& pipe : pipes) {
        pipe.x -= PIPE_SPEED * dt * TARGET_FPS;
    }

    // Count and remove off-screen pipes.
    auto it = std::remove_if(pipes.begin(), pipes.end(),
        [](const Pipe& p) { return isOffScreen(p); });
    int removed = static_cast<int>(std::distance(it, pipes.end()));
    pipes.erase(it, pipes.end());
    return removed;
}

bool checkCollision(const Pipe& pipe, Bird::Bounds b) {
    // Top pipe
    if (aabbOverlap(b.x, b.y, b.w, b.h,
                    pipe.x, topPipeY(pipe), PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT)) {
        return true;
    }
    // Bottom pipe
    return aabbOverlap(b.x, b.y, b.w, b.h,
                       pipe.x, bottomPipeY(pipe), PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT);
}

void drawPipe(const goud::Context& ctx, const Pipe& pipe, goud_texture pipeTex) {
    goud_color white{ 1.0f, 1.0f, 1.0f, 1.0f };
    constexpr float PI = 3.14159265358979323846f;

    // Top pipe (rotated 180 degrees)
    ctx.drawSprite(
        pipeTex,
        pipe.x + PIPE_IMAGE_WIDTH / 2.0f,
        topPipeY(pipe) + PIPE_IMAGE_HEIGHT / 2.0f,
        PIPE_IMAGE_WIDTH,
        PIPE_IMAGE_HEIGHT,
        PI,
        white
    );

    // Bottom pipe (no rotation)
    ctx.drawSprite(
        pipeTex,
        pipe.x + PIPE_IMAGE_WIDTH / 2.0f,
        bottomPipeY(pipe) + PIPE_IMAGE_HEIGHT / 2.0f,
        PIPE_IMAGE_WIDTH,
        PIPE_IMAGE_HEIGHT,
        0.0f,
        white
    );
}

}  // namespace flappy
