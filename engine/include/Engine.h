#ifndef GOUDENGINE_ENGINE_H
#define GOUDENGINE_ENGINE_H

#include "Renderer.h"
#include <memory>

namespace GoudEngine
{

    class Engine
    {
    public:
        Engine();
        ~Engine();

        bool Initialize();
        void Run();
        void Shutdown();

    private:
        std::unique_ptr<Renderer> renderer;
    };

} // namespace GoudEngine

#endif // GOUDENGINE_ENGINE_H