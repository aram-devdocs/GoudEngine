#include "Engine.h"

using namespace GoudEngine;

int main()
{
    Engine engine;

    engine.Initialize();

    engine.Run();

    engine.Shutdown();

    return 0;
}