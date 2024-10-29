#ifndef GOUDENGINE_SDL2_OPENGLRENDERER_H
#define GOUDENGINE_SDL2_OPENGLRENDERER_H

#include "Renderer.h"
#include <SDL2/SDL.h>
#include <SDL2/SDL_opengl.h>

namespace GoudEngine {

class SDL2_OpenGLRenderer : public Renderer {
public:
    SDL2_OpenGLRenderer();
    ~SDL2_OpenGLRenderer() override;

    bool Initialize() override;
    void Clear() override;
    void Present() override;
    void Shutdown() override;

private:
    SDL_Window* window = nullptr;
    SDL_GLContext glContext = nullptr;
};

} // namespace GoudEngine

#endif // GOUDENGINE_SDL2_OPENGLRENDERER_H