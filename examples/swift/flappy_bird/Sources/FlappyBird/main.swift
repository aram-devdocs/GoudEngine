// main.swift — Swift Flappy Bird (parity with C# flappy_goud)

import GoudEngine

let config = EngineConfig()
config
    .setSize(
        width: GameConstants.screenWidth,
        height: GameConstants.screenHeight + UInt32(GameConstants.baseHeight)
    )
    .setTitle(title: "Flappy Bird Clone")
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
