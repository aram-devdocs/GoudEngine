// #include "Game.h"
// #include <iostream>

// namespace GoudEngine {

// Game::Game() : engine(std::make_unique<Engine>()) {}

// Game::~Game() {
//     Shutdown();
// }

// void Game::SetWindowTitle(const std::string& title) {
//     windowTitle = title;
// }

// void Game::SetWindowSize(int width, int height) {
//     windowWidth = width;
//     windowHeight = height;
// }

// bool Game::Initialize() {
//     if (!engine->Initialize()) {
//         std::cerr << "Engine initialization failed." << std::endl;
//         return false;
//     }
//     std::cout << "Game initialized with window title: " << windowTitle 
//               << " and size: " << windowWidth << "x" << windowHeight << std::endl;
//     return true;
// }

// void Game::Run() {
//     engine->Run();
// }

// void Game::Shutdown() {
//     engine->Shutdown();
//     std::cout << "Game shutting down." << std::endl;
// }

// } // namespace GoudEngine