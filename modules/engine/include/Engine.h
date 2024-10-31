#ifndef GOUDENGINE_ENGINE_H
#define GOUDENGINE_ENGINE_H

#include "Renderer.h"
#include <memory>
#include <string>
#include <functional>
#include <vector>
#include <iostream>

namespace GoudEngine {

    class Engine {
    public:
        Engine(const std::string &title, int width, int height, std::unique_ptr<Renderer> renderer);
        virtual ~Engine();

        bool Initialize();
        void Run();
        void Shutdown();

        // Add Polygon with ID to the engine to be drawn
        void AddPolygon(int id, const std::vector<std::pair<float, float>> &vertices);

        // Update Polygon vertices by ID
        void UpdatePolygon(int id, const std::vector<std::pair<float, float>> &newVertices);

        // Set lifecycle callbacks
        void SetOnInit(const std::function<bool()> &callback);
        void SetOnUpdate(const std::function<void()> &callback);
        void SetOnShutdown(const std::function<void()> &callback);

    private:
        std::unique_ptr<Renderer> renderer;
        std::string title;
        int width;
        int height;

        std::function<bool()> onInit;
        std::function<void()> onUpdate;
        std::function<void()> onShutdown;
        std::vector<std::pair<int, std::vector<std::pair<float, float>>>> polygons;
    };

} // namespace GoudEngine

#endif // GOUDENGINE_ENGINE_H