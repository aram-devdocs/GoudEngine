#include "Engine.h"
#include "SDL2_OpenGLRenderer.h"
#include <iostream>

namespace GoudEngine
{

    Engine::Engine() : renderer(std::make_unique<SDL2_OpenGLRenderer>()) {}

    Engine::~Engine()
    {
        Shutdown();
    }

    bool Engine::Initialize()
    {
        if (!renderer->Initialize())
        {
            std::cerr << "Renderer initialization failed." << std::endl;
            return false;
        }
        std::cout << "Engine initialized." << std::endl;
        return true;
    }

    void Engine::Run()
    {
        bool isRunning = true;
        SDL_Event event;

        while (isRunning)
        {
            while (SDL_PollEvent(&event))
            {
                if (event.type == SDL_QUIT)
                {
                    isRunning = false;
                }
            }

            renderer->Clear();
            // TODO: Add rendering and update logic here
            renderer->Present();
        }
    }

    void Engine::Shutdown()
    {
        if (renderer)
        {
            renderer->Shutdown();
        }
        std::cout << "Engine shutting down." << std::endl;
    }

} // namespace GoudEngine