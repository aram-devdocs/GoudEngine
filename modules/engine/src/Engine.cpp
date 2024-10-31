#include "Engine.h"
#include "Renderer.h"
#include <SDL2/SDL.h>

namespace GoudEngine {

    Engine::Engine(const std::string &title, int width, int height, std::unique_ptr<Renderer> renderer)
        : title(title), width(width), height(height), renderer(std::move(renderer)) {}
    
    Engine::~Engine() {
        Shutdown();
    }

    bool Engine::Initialize() {
        if (renderer && !renderer->Initialize()) {
            std::cerr << "Renderer initialization failed." << std::endl;
            return false;
        }
        if (onInit && !onInit()) {
            std::cerr << "Engine onInit callback failed." << std::endl;
            return false;
        }
        return true;
    }

    void Engine::Run() {
        if (!Initialize()) {
            std::cerr << "Engine initialization failed." << std::endl;
            return;
        }

        bool running = true;
        SDL_Event event;

        while (running) {
            // Poll SDL events
            while (SDL_PollEvent(&event)) {
                if (event.type == SDL_QUIT) {
                    running = false;
                }
            }

            // Clear the renderer
            renderer->Clear();

            // Update polygons
            for (const auto &polygon : polygons) {
                renderer->DrawPolygon(polygon.second);
            }

            // Call the onUpdate callback if defined
            if (onUpdate) {
                onUpdate();
            }

            // Present the renderer
            renderer->Present();

            // Simple frame delay for stability
            SDL_Delay(16); // approximately 60 FPS
        }

        Shutdown();
    }

    void Engine::Shutdown() {
        if (renderer) {
            renderer->Shutdown();
            renderer.reset();
        }
        if (onShutdown) {
            onShutdown();
        }
    }

    void Engine::AddPolygon(int id, const std::vector<std::pair<float, float>>& vertices) {
        std::cout << "Adding polygon #" << id << " with vertices." << std::endl;
        polygons.push_back({id, vertices});
    }

    void Engine::UpdatePolygon(int id, const std::vector<std::pair<float, float>>& newVertices) {
        for (auto& polygon : polygons) {
            if (polygon.first == id) {
                polygon.second = newVertices;
                break;
            }
        }
    }

    void Engine::SetOnInit(const std::function<bool()>& callback) {
        onInit = callback;
    }

    void Engine::SetOnUpdate(const std::function<void()>& callback) {
        onUpdate = callback;
    }

    void Engine::SetOnShutdown(const std::function<void()>& callback) {
        onShutdown = callback;
    }

} // namespace GoudEngine