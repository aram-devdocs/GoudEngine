// main.swift -- iOS Flappy Bird
// Standalone iOS example demonstrating touch, keyboard, and mouse input.
// Runs in portrait orientation (288x624 including base).

import GoudEngine

let config = EngineConfig()
config
    .setSize(
        width: GameConstants.screenWidth,
        height: GameConstants.screenHeight + UInt32(GameConstants.baseHeight)
    )
    .setTitle(title: "Flappy Bird - iOS")
    .setPhysicsBackend2D(backend: .SIMPLE)

let game = config.build()
let gameManager = GameManager(game: game)

gameManager.initialize()
gameManager.start()

while !game.shouldClose() {
    game.beginFrame(r: 0.4, g: 0.7, b: 0.9, a: 1.0)
    gameManager.update(deltaTime: game.deltaTime)
    game.endFrame()
}
