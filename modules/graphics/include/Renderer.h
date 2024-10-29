#ifndef GOUDENGINE_RENDERER_H
#define GOUDENGINE_RENDERER_H

namespace GoudEngine {

class Renderer {
public:
    Renderer();
    ~Renderer();

    void Initialize();
    void Render();
    void Shutdown();
};

} // namespace GoudEngine

#endif // GOUDENGINE_RENDERER_H
