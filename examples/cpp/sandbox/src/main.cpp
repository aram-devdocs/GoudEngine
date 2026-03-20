/// GoudEngine Sandbox -- C++ Example
///
/// Demonstrates basic engine features: window creation, texture loading,
/// sprite drawing, quad drawing, WASD movement, and Escape to quit.
///
/// Controls:
///   WASD / Arrow Keys -- Move the sprite
///   Escape            -- Quit

#include <goud/goud.hpp>

#include <cmath>
#include <cstdint>

static constexpr std::uint32_t WINDOW_WIDTH  = 1280;
static constexpr std::uint32_t WINDOW_HEIGHT = 720;
static constexpr float MOVE_SPEED = 220.0f;

int main() {
    // -- Engine setup ---------------------------------------------------------
    auto config = goud::EngineConfig::create();
    config.setTitle("GoudEngine Sandbox - C++");
    config.setSize(WINDOW_WIDTH, WINDOW_HEIGHT);

    auto engine = goud::Engine::create(std::move(config));
    auto& ctx = engine.context();

    ctx.enableBlending();

    // -- Asset loading --------------------------------------------------------
    goud_texture background{};
    ctx.loadTexture("examples/shared/sandbox/sprites/background-day.png", background);

    goud_texture sprite{};
    ctx.loadTexture("examples/shared/sandbox/sprites/yellowbird-midflap.png", sprite);

    // -- State ----------------------------------------------------------------
    float playerX = 250.0f;
    float playerY = 300.0f;
    float elapsed = 0.0f;

    goud_color clearColor{ 0.07f, 0.10f, 0.14f, 1.0f };
    goud_color white{ 1.0f, 1.0f, 1.0f, 1.0f };
    goud_color accentBlue{ 0.20f, 0.55f, 0.95f, 0.80f };
    goud_color groundTint{ 0.03f, 0.10f, 0.12f, 0.40f };

    // -- Game loop ------------------------------------------------------------
    while (!engine.shouldClose()) {
        float dt = engine.pollEvents();
        if (dt <= 0.0f) {
            dt = 1.0f / 60.0f;
        }
        elapsed += dt;

        // Input: Escape to quit
        if (ctx.keyJustPressed(static_cast<goud_key>(KEY_ESCAPE))) {
            break;
        }

        // Input: WASD / Arrow movement
        if (ctx.keyDown(static_cast<goud_key>(KEY_A))
            || ctx.keyDown(static_cast<goud_key>(KEY_LEFT))) {
            playerX -= MOVE_SPEED * dt;
        }
        if (ctx.keyDown(static_cast<goud_key>(KEY_D))
            || ctx.keyDown(static_cast<goud_key>(KEY_RIGHT))) {
            playerX += MOVE_SPEED * dt;
        }
        if (ctx.keyDown(static_cast<goud_key>(KEY_W))
            || ctx.keyDown(static_cast<goud_key>(KEY_UP))) {
            playerY -= MOVE_SPEED * dt;
        }
        if (ctx.keyDown(static_cast<goud_key>(KEY_S))
            || ctx.keyDown(static_cast<goud_key>(KEY_DOWN))) {
            playerY += MOVE_SPEED * dt;
        }

        ctx.beginFrame();
        ctx.clear(clearColor);

        // Draw background
        ctx.drawSprite(
            background,
            static_cast<float>(WINDOW_WIDTH) / 2.0f,
            static_cast<float>(WINDOW_HEIGHT) / 2.0f,
            static_cast<float>(WINDOW_WIDTH),
            static_cast<float>(WINDOW_HEIGHT),
            0.0f,
            white
        );

        // Draw movable sprite with slow rotation
        float rotation = elapsed * 0.25f;
        ctx.drawSprite(sprite, playerX, playerY, 64.0f, 64.0f, rotation, white);

        // Draw colored quads
        ctx.drawQuad(920.0f, 260.0f, 180.0f, 40.0f, accentBlue);
        ctx.drawQuad(640.0f, 654.0f, 1280.0f, 132.0f, groundTint);

        ctx.endFrame();
        engine.swapBuffers();
    }

    return 0;
}
