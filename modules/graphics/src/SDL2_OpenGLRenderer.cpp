#include "SDL2_OpenGLRenderer.h"
#include <iostream>

namespace GoudEngine {

SDL2_OpenGLRenderer::SDL2_OpenGLRenderer() = default;

SDL2_OpenGLRenderer::~SDL2_OpenGLRenderer() {
    Shutdown();
}

bool SDL2_OpenGLRenderer::Initialize() {
    // Initialize SDL with video support
    if (SDL_Init(SDL_INIT_VIDEO) != 0) {
        std::cerr << "SDL Initialization failed: " << SDL_GetError() << std::endl;
        return false;
    }

    // Create an SDL window
    window = SDL_CreateWindow("GoudEngine", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                              800, 600, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);
    if (!window) {
        std::cerr << "SDL Window creation failed: " << SDL_GetError() << std::endl;
        return false;
    }

    // Set up the OpenGL context
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
    SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
    glContext = SDL_GL_CreateContext(window);
    if (!glContext) {
        std::cerr << "OpenGL context creation failed: " << SDL_GetError() << std::endl;
        return false;
    }

    // Initialize OpenGL settings
    glViewport(0, 0, 800, 600);
    glEnable(GL_DEPTH_TEST);

    return true;
}

void SDL2_OpenGLRenderer::Clear() {
    glClearColor(0.1f, 0.1f, 0.1f, 1.0f); // Dark grey background
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
}

void SDL2_OpenGLRenderer::Present() {
    SDL_GL_SwapWindow(window);
}

void SDL2_OpenGLRenderer::Shutdown() {
    if (glContext) {
        SDL_GL_DeleteContext(glContext);
        glContext = nullptr;
    }
    if (window) {
        SDL_DestroyWindow(window);
        window = nullptr;
    }
    SDL_Quit();
}

} // namespace GoudEngine