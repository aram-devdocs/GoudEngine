#ifndef GOUDENGINE_ENGINE_H
#define GOUDENGINE_ENGINE_H

namespace GoudEngine {

class Engine {
public:
    Engine();
    ~Engine();

    void Initialize();
    void Run();
    void Shutdown();
};

} // namespace GoudEngine

#endif // GOUDENGINE_ENGINE_H
