#include "Engine.h"
#include <iostream>

namespace GoudEngine {

Engine::Engine() {}

Engine::~Engine() {}

void Engine::Initialize() {
    std::cout << "Engine initialized." << std::endl;
}

void Engine::Run() {
    std::cout << "Engine running." << std::endl;
}

void Engine::Shutdown() {
    std::cout << "Engine shutting down." << std::endl;
}

} // namespace GoudEngine
