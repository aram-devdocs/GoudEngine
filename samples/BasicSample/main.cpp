#include "Engine.h"
#include "SDL2_OpenGLRenderer.h"
#include <memory>
#include <vector>

int main(int argc, char* argv[]) {
    // Create an instance of SDL2_OpenGLRenderer with window title and size
    auto renderer = std::make_unique<GoudEngine::SDL2_OpenGLRenderer>("GoudEngine - OpenGL Test", 640, 480);
    
    // Create an Engine instance with the renderer
    GoudEngine::Engine engine("GoudEngine Test", 640, 480, std::move(renderer));

    // Set up the lifecycle callbacks
    engine.SetOnInit([]() -> bool {
        std::cout << "Engine initialization complete." << std::endl;
        return true;
    });

    engine.SetOnUpdate([&]() {
        // Example of updating polygons over time (could be used for animations)
        static float scale = 1.0f;
        static bool growing = true;

        std::vector<std::pair<float, float>> vertices = {
            { -0.5f * scale, -0.5f * scale },
            {  0.5f * scale, -0.5f * scale },
            {  0.0f,         0.5f * scale }
        };
        engine.UpdatePolygon(1, vertices);

        // Update scale for simple animation
        if (growing) scale += 0.01f;
        else scale -= 0.01f;
        
        if (scale >= 1.5f) growing = false;
        else if (scale <= 0.5f) growing = true;
    });

    engine.SetOnShutdown([]() {
        std::cout << "Engine shutdown complete." << std::endl;
    });

    // Add a polygon to the engine for testing (a triangle)
    std::vector<std::pair<float, float>> triangle = {
        { -0.5f, -0.5f },
        {  0.5f, -0.5f },
        {  0.0f,  0.5f }
    };
    engine.AddPolygon(1, triangle);

    // Start the engine
    engine.Run();

    return 0;
}