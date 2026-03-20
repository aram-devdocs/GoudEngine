// main.swift -- GoudEngine Sandbox (Swift)
//
// Demonstrates basic engine features: window creation, texture loading,
// sprite drawing, quad drawing, WASD movement, and Escape to quit.
//
// Controls:
//   WASD / Arrow Keys -- Move the sprite
//   Escape            -- Quit

import GoudEngine

let windowWidth: UInt32 = 1280
let windowHeight: UInt32 = 720
let moveSpeed: Float = 220.0

// -- Engine setup -------------------------------------------------------------

let config = EngineConfig()
config
    .setSize(width: windowWidth, height: windowHeight)
    .setTitle(title: "GoudEngine Sandbox - Swift")

let game = config.build()

// -- Asset loading ------------------------------------------------------------

let background = game.loadTexture(path: "examples/shared/sandbox/sprites/background-day.png")
let sprite = game.loadTexture(path: "examples/shared/sandbox/sprites/yellowbird-midflap.png")

// -- State --------------------------------------------------------------------

var playerX: Float = 250.0
var playerY: Float = 300.0
var elapsed: Float = 0.0

// -- Game loop ----------------------------------------------------------------

while !game.shouldClose() {
    game.beginFrame(r: 0.07, g: 0.10, b: 0.14, a: 1.0)

    let dt = game.deltaTime > 0 ? game.deltaTime : Float(1.0 / 60.0)
    elapsed += dt

    // Input: Escape to quit
    if game.isKeyPressed(key: .ESCAPE) {
        game.requestClose()
    }

    // Input: WASD / Arrow movement
    if game.isKeyPressed(key: .A) || game.isKeyPressed(key: .LEFT) {
        playerX -= moveSpeed * dt
    }
    if game.isKeyPressed(key: .D) || game.isKeyPressed(key: .RIGHT) {
        playerX += moveSpeed * dt
    }
    if game.isKeyPressed(key: .W) || game.isKeyPressed(key: .UP) {
        playerY -= moveSpeed * dt
    }
    if game.isKeyPressed(key: .S) || game.isKeyPressed(key: .DOWN) {
        playerY += moveSpeed * dt
    }

    // Draw background
    game.drawSprite(
        texture: background,
        x: Float(windowWidth) / 2.0,
        y: Float(windowHeight) / 2.0,
        width: Float(windowWidth),
        height: Float(windowHeight)
    )

    // Draw movable sprite with slow rotation
    let rotation = elapsed * 0.25
    game.drawSprite(
        texture: sprite,
        x: playerX,
        y: playerY,
        width: 64.0,
        height: 64.0,
        rotation: rotation
    )

    // Draw colored quads
    game.drawQuad(
        x: 920.0, y: 260.0,
        width: 180.0, height: 40.0,
        color: Color(r: 0.20, g: 0.55, b: 0.95, a: 0.80)
    )
    game.drawQuad(
        x: 640.0, y: 654.0,
        width: 1280.0, height: 132.0,
        color: Color(r: 0.03, g: 0.10, b: 0.12, a: 0.40)
    )

    game.endFrame()
}
