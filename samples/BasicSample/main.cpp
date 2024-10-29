#include "Engine.h"
#include "SDL2_OpenGLRenderer.h"

using namespace GoudEngine;

int main()
{
    Engine engine;
    SDL2_OpenGLRenderer renderer; // Use the concrete renderer here

    engine.Initialize();
    renderer.Initialize();

    engine.Run();

    renderer.Shutdown();
    engine.Shutdown();

    return 0;
}