#include "Game.h"
#include <iostream>

using namespace GoudEngine;

Game game("Basic Sample", 400, 600);

bool CustomInit()
{

    // Custom initialization logic
    std::cout << "Custom game initialization." << std::endl;
    // Add a polygon to the game
    std::vector<std::pair<float, float>> vertices = {
        {100.0f, 100.0f},
        {200.0f, 100.0f},
        {150.0f, 200.0f}};

    game.AddPolygon(vertices);

    return true;
}

void CustomUpdate()
{
    // Custom update logic
    std::cout << "Custom game updating." << std::endl;
}

void CustomShutdown()
{
    // Custom shutdown logic
    std::cout << "Custom game shutting down." << std::endl;
}

int main()
{

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