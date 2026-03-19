// Bird.swift

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

        if game.isKeyPressed(key: .SPACE) || game.isMouseButtonPressed(button: .LEFT) {
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
