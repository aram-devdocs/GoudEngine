#ifndef GOUDENGINE_GAME_H
#define GOUDENGINE_GAME_H

#include "Engine.h"

namespace GoudEngine
{

    class Game : public Engine
    {
    public:
        Game(const std::string &title, int width, int height)
            : Engine(title, width, height) {}
    };

} // namespace GoudEngine

#endif // GOUDENGINE_GAME_H
