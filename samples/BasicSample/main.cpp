#include "Engine.h"

using namespace GoudEngine;

int main()
{
    Engine engine("Basic Sample", 400, 600);

    engine.Initialize();

    engine.Run();

    engine.Shutdown();

    return 0;
}