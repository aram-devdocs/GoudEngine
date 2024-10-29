#include "Renderer.h"
#include <iostream>

namespace GoudEngine {

Renderer::Renderer() {}

Renderer::~Renderer() {}

void Renderer::Initialize() {
    std::cout << "Renderer initialized." << std::endl;
}

void Renderer::Render() {
    std::cout << "Rendering frame." << std::endl;
}

void Renderer::Shutdown() {
    std::cout << "Renderer shutting down." << std::endl;
}

} // namespace GoudEngine
