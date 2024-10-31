#include "SDL2_OpenGLRenderer.h"
#include <iostream>
#include <vector>
#include <cmath>

namespace GoudEngine
{

    SDL2_OpenGLRenderer::SDL2_OpenGLRenderer(const std::string &title, int width, int height)
        : windowTitle(title), windowWidth(width), windowHeight(height) {}

    SDL2_OpenGLRenderer::~SDL2_OpenGLRenderer()
    {
        Shutdown();
    }

    bool SDL2_OpenGLRenderer::Initialize()
    {
        if (SDL_Init(SDL_INIT_VIDEO) < 0)
        {
            std::cerr << "SDL video initialization failed: " << SDL_GetError() << std::endl;
            return false;
        }

        SDL_GL_SetAttribute(SDL_GL_RED_SIZE, 5);
        SDL_GL_SetAttribute(SDL_GL_GREEN_SIZE, 5);
        SDL_GL_SetAttribute(SDL_GL_BLUE_SIZE, 5);
        SDL_GL_SetAttribute(SDL_GL_DEPTH_SIZE, 16);
        SDL_GL_SetAttribute(SDL_GL_DOUBLEBUFFER, 1);

        window = SDL_CreateWindow(
            windowTitle.c_str(),
            SDL_WINDOWPOS_CENTERED,
            SDL_WINDOWPOS_CENTERED,
            windowWidth, windowHeight,
            SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);

        if (!window)
        {
            std::cerr << "Window creation failed: " << SDL_GetError() << std::endl;
            return false;
        }

        glContext = SDL_GL_CreateContext(window);
        if (!glContext)
        {
            std::cerr << "OpenGL context creation failed: " << SDL_GetError() << std::endl;
            return false;
        }

        glViewport(0, 0, windowWidth, windowHeight);
        glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        glEnable(GL_DEPTH_TEST);
        return true;
    }

    void SDL2_OpenGLRenderer::Clear()
    {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
    }

    void SDL2_OpenGLRenderer::Present()
    {
        SDL_GL_SwapWindow(window);
    }

    void SDL2_OpenGLRenderer::Shutdown()
    {
        if (glContext)
        {
            SDL_GL_DeleteContext(glContext);
            glContext = nullptr;
        }
        if (window)
        {
            SDL_DestroyWindow(window);
            window = nullptr;
        }
        SDL_Quit();
    }

    void SDL2_OpenGLRenderer::DrawPolygon(const std::vector<std::pair<float, float>> &vertices)
    {
        glBegin(GL_POLYGON);
        for (const auto &vertex : vertices)
        {
            glVertex2f(vertex.first, vertex.second);
        }
        glEnd();
    }

} // namespace GoudEngine
