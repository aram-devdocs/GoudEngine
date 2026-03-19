// Movement.swift

import Foundation

final class Movement {
    var velocity: Float = 0
    private(set) var rotation: Float = 0

    private let gravity: Float
    private let jumpStrength: Float
    private let rotationSmoothing: Float = 0.03
    private var jumpCooldownTimer: Float = 0

    init(gravity: Float, jumpStrength: Float) {
        self.gravity = gravity
        self.jumpStrength = jumpStrength
    }

    func applyGravity(deltaTime: Float) {
        velocity += gravity * deltaTime * Float(GameConstants.targetFPS)
        jumpCooldownTimer -= max(0, deltaTime)
    }

    func tryJump(deltaTime: Float) -> Bool {
        if jumpCooldownTimer <= 0 {
            jump()
            jumpCooldownTimer = GameConstants.jumpCooldown
            return true
        }
        return false
    }

    private func jump() {
        velocity = 0
        velocity = jumpStrength * Float(GameConstants.targetFPS)
    }

    func updatePosition(positionY: inout Float, deltaTime: Float) {
        positionY += velocity * deltaTime

        let targetRotation = min(max(velocity * 3, -45), 45)
        rotation += (targetRotation - rotation) * rotationSmoothing
    }
}
