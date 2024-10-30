#include "Engine.h"

using namespace GoudEngine;

int main()
{
    Engine engine("Basic Sample", 400, 600);

    if (!engine.Initialize())
    {
        return -1;
    }

    engine.Run();

    engine.Shutdown();

    return 0;
}