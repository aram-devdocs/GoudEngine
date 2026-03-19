/// Flappy Bird -- GoudEngine C++ Example
///
/// A complete Flappy Bird clone demonstrating the GoudEngine from C++.
/// Game constants and behavior match the C#, Python, TypeScript, and Rust
/// versions exactly for SDK parity validation.
///
/// Controls:
///   Space / Left Click -- Flap (jump)
///   R                  -- Restart
///   Escape             -- Quit

#include <goud/goud.hpp>
#include "constants.hpp"
#include "game_manager.hpp"

int main() {
    using namespace flappy;

    // -- Engine setup ---------------------------------------------------------
    auto config = goud::EngineConfig::create();
    config.setTitle("Flappy Bird C++");
    config.setSize(
        static_cast<std::uint32_t>(SCREEN_WIDTH),
        static_cast<std::uint32_t>(SCREEN_HEIGHT + BASE_HEIGHT)
    );
    config.setVsync(false);
    config.setTargetFps(static_cast<std::uint32_t>(TARGET_FPS));

    auto engine = goud::Engine::create(std::move(config));
    auto& ctx = engine.context();

    ctx.enableBlending();

    // -- Game init ------------------------------------------------------------
    flappy::GameManager manager;
    manager.init(ctx);

    // -- Game loop ------------------------------------------------------------
    goud_color skyBlue{ 0.4f, 0.7f, 0.9f, 1.0f };

    while (!engine.shouldClose()) {
        float dt = engine.pollEvents();
        if (dt <= 0.0f) {
            dt = 1.0f / TARGET_FPS;
        }

        if (!manager.handleInput(ctx)) {
            break;  // Escape pressed
        }
        manager.update(ctx, dt);

        ctx.beginFrame();
        ctx.clear(skyBlue);
        manager.draw(ctx);
        ctx.endFrame();
        engine.swapBuffers();
    }

    return 0;
}
