#include "Game.h"
#include <iostream>

using namespace GoudEngine;

bool CustomInit() {
    // Custom initialization logic
    std::cout << "Custom game initialization." << std::endl;
    return true;
}

void CustomUpdate() {
    // Custom update logic
    std::cout << "Custom game updating." << std::endl;
}

void CustomShutdown() {
    // Custom shutdown logic
    std::cout << "Custom game shutting down." << std::endl;
}

int main()
{
    Game game("Basic Sample", 400, 600);

    game.SetOnInit(CustomInit);
    game.SetOnUpdate(CustomUpdate);
    game.SetOnShutdown(CustomShutdown);

    if (!game.Initialize())
    {
        return -1;
    }

    game.Run();

    game.Shutdown();

    return 0;
}