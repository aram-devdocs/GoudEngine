// Bird.swift -- iOS Flappy Bird
// Demonstrates all three input methods:
//   1. Touch  -- primary mobile input (tap anywhere to flap)
//   2. Mouse  -- iOS Simulator trackpad click
//   3. Keyboard -- iOS Simulator hardware keyboard (Space bar)

import GoudEngine

final class Bird {
    private let game: GoudGame
    private let movement: Movement
    private let animator: BirdAnimator

    private(set) var x: Float
    private(set) var y: Float

    static let width: Float = 34
    static let height: Float = 24

    init(game: GoudGame) {
        self.game = game
        movement = Movement(gravity: GameConstants.gravity, jumpStrength: GameConstants.jumpStrength)
        x = Float(GameConstants.screenWidth) / 4
        y = Float(GameConstants.screenHeight) / 2
        animator = BirdAnimator(game: game, x: x, y: y)
    }

    func initialize() {
        animator.initialize()
    }

    func reset() {
        x = Float(GameConstants.screenWidth) / 4
        y = Float(GameConstants.screenHeight) / 2
        movement.velocity = 0
        animator.reset()
    }

    func update(deltaTime: Float) -> Bool {
        var didFlap = false

        // Input method 1: Touch -- primary input on iOS devices
        let wantsTouch = game.isTouchJustPressed(touchId: 0)
        // Input method 2: Keyboard -- Space bar via Simulator hardware keyboard
        let wantsKey = game.isKeyPressed(key: .SPACE)
        // Input method 3: Mouse -- Simulator trackpad / pointer click
        let wantsMouse = game.isMouseButtonPressed(button: .LEFT)

        if wantsTouch || wantsKey || wantsMouse {
            didFlap = movement.tryJump(deltaTime: deltaTime)
        }

        movement.applyGravity(deltaTime: deltaTime)
        var yPosition = y
        movement.updatePosition(positionY: &yPosition, deltaTime: deltaTime)
        y = yPosition

        animator.update(deltaTime: deltaTime, x: x, y: y, rotation: movement.rotation)

        return didFlap
    }

    func draw() {
        animator.draw()
    }
}
