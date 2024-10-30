#ifndef GOUDENGINE_ENGINE_H
#define GOUDENGINE_ENGINE_H

#include "Renderer.h"
#include <memory>
#include <string>

namespace GoudEngine
{

    class Engine
    {
    public:
        Engine(const std::string &title, int width, int height);
        ~Engine();

        bool Initialize();
        void Run();
        void Shutdown();

        // Add Polygon to the engine to be drawn
        void AddPolygon(const std::vector<std::pair<float, float>>& vertices) {
            if (renderer) {
                renderer->DrawPolygon(vertices);
            }
        }

    private:
        std::unique_ptr<Renderer> renderer;
        std::string title;
        int width;
        int height;
    };

} // namespace GoudEngine

#endif // GOUDENGINE_ENGINE_H