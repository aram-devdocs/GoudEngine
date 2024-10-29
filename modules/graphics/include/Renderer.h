#ifndef GOUDENGINE_RENDERER_H
#define GOUDENGINE_RENDERER_H

namespace GoudEngine {

class Renderer {
public:
    virtual ~Renderer() = default;

    virtual bool Initialize() = 0;
    virtual void Clear() = 0;
    virtual void Present() = 0;
    virtual void Shutdown() = 0;
};

} // namespace GoudEngine

#endif // GOUDENGINE_RENDERER_H