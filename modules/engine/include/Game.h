#ifndef GOUDENGINE_GAME_H
#define GOUDENGINE_GAME_H

#include "Engine.h"
#include "Renderer.h"

namespace GoudEngine
{

    class Game : public Engine
    {
    public:
        Game(const std::string &title, int width, int height, std::unique_ptr<Renderer> renderer)
            : Engine(title, width, height, std::move(renderer)) {}
    };

} // namespace GoudEngine

#endif // GOUDENGINE_GAME_H
