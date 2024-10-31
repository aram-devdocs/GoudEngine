#include "SDL2_OpenGLRenderer.h"
#include <iostream>
#include <vector>
#include <utility>

namespace GoudEngine
{

    SDL2_OpenGLRenderer::SDL2_OpenGLRenderer(const std::string &title, int width, int height)
        : window(nullptr), glContext(nullptr), windowTitle(title), windowWidth(width), windowHeight(height) {}

    SDL2_OpenGLRenderer::~SDL2_OpenGLRenderer()
    {
        Shutdown();
    }

    bool SDL2_OpenGLRenderer::Initialize()
    {
        // Initialize SDL with video support
        if (SDL_Init(SDL_INIT_VIDEO) != 0)
        {
            std::cerr << "SDL Initialization failed: " << SDL_GetError() << std::endl;
            return false;
        }

        // Create an SDL window
        window = SDL_CreateWindow(windowTitle.c_str(), SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED,
                                  windowWidth, windowHeight, SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN);
        if (!window)
        {
            std::cerr << "SDL Window creation failed: " << SDL_GetError() << std::endl;
            return false;
        }

        // Set up the OpenGL context
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MAJOR_VERSION, 3);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_MINOR_VERSION, 3);
        SDL_GL_SetAttribute(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_CORE);
        glContext = SDL_GL_CreateContext(window);
        if (!glContext)
        {
            std::cerr << "OpenGL context creation failed: " << SDL_GetError() << std::endl;
            return false;
        }

        // Initialize OpenGL settings
        glViewport(0, 0, windowWidth, windowHeight);
        glEnable(GL_DEPTH_TEST);

        // Set up the projection matrix
        glMatrixMode(GL_PROJECTION);
        glLoadIdentity();
        glOrtho(0.0, windowWidth, windowHeight, 0.0, -1.0, 1.0);

        // Set up the modelview matrix
        glMatrixMode(GL_MODELVIEW);
        glLoadIdentity();

        // Enable blending for transparency
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);

        return true;
    }

    void SDL2_OpenGLRenderer::Clear()
    {
        glClearColor(0.1f, 0.1f, 0.1f, 1.0f); // Dark grey background
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

    void SDL2_OpenGLRenderer::DrawPolygon(const std::vector<std::pair<float, float>>& vertices)
    {
        std::cout << "Drawing polygon with " << vertices.size() << " vertices." << std::endl;

        // Set the color to red for the polygon
        glColor3f(1.0f, 0.0f, 0.0f);
        glBegin(GL_POLYGON);
        for (const auto& vertex : vertices)
        {
            std::cout << "Vertex: (" << vertex.first << ", " << vertex.second << ")" << std::endl;
            glVertex2f(vertex.first, vertex.second);
        }
        glEnd();

        // Set the color to black for the outline
        glColor3f(0.0f, 0.0f, 0.0f);
        glBegin(GL_LINE_LOOP);
        for (const auto& vertex : vertices)
        {
            glVertex2f(vertex.first, vertex.second);
        }
        glEnd();
    }

} // namespace GoudEngine