#include <goud/goud.hpp>

int main() {
    auto config = goud::EngineConfig::create();
    config.setTitle("My Game");
    config.setSize(800, 600);

    auto engine = goud::Engine::create(std::move(config));
    engine.enableBlending();

    goud_color clear{0.2f, 0.3f, 0.4f, 1.0f};

    while (!engine.shouldClose()) {
        engine.pollEvents();
        engine.context().beginFrame();
        engine.context().clear(clear);
        engine.context().endFrame();
        engine.swapBuffers();
    }
    return 0;
}
