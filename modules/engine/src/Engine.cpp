#include "Engine.h"
#include "SDL2_OpenGLRenderer.h"
#include <iostream>

namespace GoudEngine
{

    Engine::Engine(const std::string &title, int width, int height)
        : renderer(std::make_unique<SDL2_OpenGLRenderer>(title, width, height)) {}

    Engine::~Engine()
    {
        Shutdown();
    }

    bool Engine::Initialize()
    {
        if (!renderer->Initialize() || (onInit && !onInit()))
        {
            std::cerr << "Initialization failed." << std::endl;
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
            if (onUpdate) onUpdate();
            renderer->Present();
        }
    }

    void Engine::Shutdown()
    {
        if (onShutdown) onShutdown();
        if (renderer)
        {
            renderer->Shutdown();
        }
        std::cout << "Engine shutting down." << std::endl;
    }

} // namespace GoudEngine