#ifndef GOUDENGINE_SDL2_OPENGL_GAME_H
#define GOUDENGINE_SDL2_OPENGL_GAME_H

#include "Engine.h"
#include "SDL2_OpenGLRenderer.h"
#include <memory>
#include <string>

namespace GoudEngine {

    class SDL2_OpenGL_Game : public Engine {
    public:
        SDL2_OpenGL_Game(const std::string &title, int width, int height)
            : Engine(title, width, height, std::make_unique<SDL2_OpenGLRenderer>(title, width, height)) {}

        // Additional game-specific initialization or helper methods can go here if needed
    };

} // namespace GoudEngine

#endif // GOUDENGINE_SDL2_OPENGL_GAME_H