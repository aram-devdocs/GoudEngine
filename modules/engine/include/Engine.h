#ifndef GOUDENGINE_ENGINE_H
#define GOUDENGINE_ENGINE_H

#include "Renderer.h"
#include <memory>
#include <string>
#include <functional>

namespace GoudEngine
{

    class Engine
    {
    public:
        Engine(const std::string &title, int width, int height);
        virtual ~Engine();

        bool Initialize();
        void Run();
        void Shutdown();

        // Add Polygon to the engine to be drawn
        void AddPolygon(const std::vector<std::pair<float, float>>& vertices) {
            if (renderer) {
                renderer->DrawPolygon(vertices);
            }
        }

        // Set lifecycle callbacks
        void SetOnInit(const std::function<bool()>& callback) { onInit = callback; }
        void SetOnUpdate(const std::function<void()>& callback) { onUpdate = callback; }
        void SetOnShutdown(const std::function<void()>& callback) { onShutdown = callback; }

    private:
        std::unique_ptr<Renderer> renderer;
        std::string title;
        int width;
        int height;

        std::function<bool()> onInit;
        std::function<void()> onUpdate;
        std::function<void()> onShutdown;
    };

} // namespace GoudEngine

#endif // GOUDENGINE_ENGINE_H