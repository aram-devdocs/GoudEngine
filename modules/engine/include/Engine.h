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

    private:
        std::unique_ptr<Renderer> renderer;
        std::string title;
        int width;
        int height;
    };

} // namespace GoudEngine

#endif // GOUDENGINE_ENGINE_H