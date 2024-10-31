#include "SDL2_OpenGL_Game.h"
#include <vector>
#include <iostream>

int main(int argc, char* argv[]) {
    // Create an SDL2_OpenGL_Game instance
    GoudEngine::SDL2_OpenGL_Game game("GoudEngine - OpenGL Game", 640, 480);

    // Set up the lifecycle callbacks
    game.SetOnInit([]() -> bool {
        std::cout << "Game initialization complete." << std::endl;
        return true;
    });

    game.SetOnUpdate([&]() {
        // Example of updating polygons over time (could be used for animations)
        static float scale = 1.0f;
        static bool growing = true;

        std::vector<std::pair<float, float>> vertices = {
            { -0.5f * scale, -0.5f * scale },
            {  0.5f * scale, -0.5f * scale },
            {  0.0f,         0.5f * scale }
        };
        game.UpdatePolygon(1, vertices);

        // Update scale for simple animation
        if (growing) scale += 0.01f;
        else scale -= 0.01f;
        
        if (scale >= 1.5f) growing = false;
        else if (scale <= 0.5f) growing = true;
    });

    game.SetOnShutdown([]() {
        std::cout << "Game shutdown complete." << std::endl;
    });

    // Add a polygon to the game for testing (a triangle)
    std::vector<std::pair<float, float>> triangle = {
        { -0.5f, -0.5f },
        {  0.5f, -0.5f },
        {  0.0f,  0.5f }
    };
    game.AddPolygon(1, triangle);

    // Start the game
    game.Run();

    return 0;
}