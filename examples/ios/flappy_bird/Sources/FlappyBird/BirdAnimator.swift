// BirdAnimator.swift -- iOS Flappy Bird

import Foundation
import GoudEngine

final class BirdAnimator {
    private let game: GoudGame
    private var frameTextures: [UInt64] = []

    private var currentFrame: Int = 0
    private var animationTime: Float = 0
    private let frameDuration: Float

    private var initialX: Float
    private var initialY: Float
    private var currentX: Float
    private var currentY: Float
    private var currentRotation: Float = 0

    private let birdWidth: Float = 34
    private let birdHeight: Float = 24

    init(game: GoudGame, x: Float, y: Float, frameDuration: Float = 0.1) {
        self.game = game
        self.frameDuration = frameDuration
        self.initialX = x
        self.initialY = y
        self.currentX = x
        self.currentY = y
    }

    func initialize() {
        frameTextures = [
            game.loadTexture(path: "assets/sprites/bluebird-downflap.png"),
            game.loadTexture(path: "assets/sprites/bluebird-midflap.png"),
            game.loadTexture(path: "assets/sprites/bluebird-upflap.png"),
        ]
    }

    func update(deltaTime: Float, x: Float, y: Float, rotation: Float) {
        animationTime += deltaTime
        if animationTime >= frameDuration {
            currentFrame = (currentFrame + 1) % frameTextures.count
            animationTime = 0
        }
        currentX = x
        currentY = y
        currentRotation = rotation
    }

    func draw() {
        guard !frameTextures.isEmpty else { return }
        game.drawSprite(
            texture: frameTextures[currentFrame],
            x: currentX + birdWidth / 2,
            y: currentY + birdHeight / 2,
            width: birdWidth,
            height: birdHeight,
            rotation: currentRotation * Float.pi / 180
        )
    }

    func reset() {
        currentFrame = 0
        animationTime = 0
        currentX = initialX
        currentY = initialY
        currentRotation = 0
    }
}
