#ifndef GOUDENGINE_SDL2_OPENGLRENDERER_H
#define GOUDENGINE_SDL2_OPENGLRENDERER_H

#include "Renderer.h"
#include "PolygonService.h"
#include <SDL2/SDL.h>
#include <SDL2/SDL_opengl.h>
#include <string>

namespace GoudEngine {

class SDL2_OpenGLRenderer : public Renderer, public PolygonService {
public:
    SDL2_OpenGLRenderer(const std::string &title, int width, int height);
    ~SDL2_OpenGLRenderer() override;

    bool Initialize() override;
    void Clear() override;
    void Present() override;
    void Shutdown() override;
    void DrawPolygon(const std::vector<std::pair<float, float>>& vertices) override;

private:
    SDL_Window* window = nullptr;
    SDL_GLContext glContext = nullptr;
    std::string windowTitle;
    int windowWidth;
    int windowHeight;
};

} // namespace GoudEngine

#endif // GOUDENGINE_SDL2_OPENGLRENDERER_H