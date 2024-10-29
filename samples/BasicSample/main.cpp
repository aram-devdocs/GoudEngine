#include "Engine.h"
#include "Renderer.h"

using namespace GoudEngine;

int main() {
    Engine engine;
    Renderer renderer;

    engine.Initialize();
    renderer.Initialize();

    engine.Run();

    renderer.Shutdown();
    engine.Shutdown();

    return 0;
}
