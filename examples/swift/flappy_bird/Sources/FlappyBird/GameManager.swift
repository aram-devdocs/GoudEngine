// GameManager.swift

import Foundation
import GoudEngine

final class GameManager {
    private let game: GoudGame

    private let scoreCounter = ScoreCounter()
    private let bird: Bird
    private var pipes: [Pipe] = []
    private var pipeSpawnTimer: Float = 0

    private var backgroundTextureId: UInt64 = 0
    private var baseTextureId: UInt64 = 0
    private var pipeTextureId: UInt64 = 0

    private let backgroundWidth: Float = 288
    private let backgroundHeight: Float = 512
    private let baseWidth: Float = 336
    private let baseHeight: Float = 112

    init(game: GoudGame) {
        self.game = game
        bird = Bird(game: game)
    }

    func initialize() {
        backgroundTextureId = game.loadTexture(path: "assets/sprites/background-day.png")
        baseTextureId = game.loadTexture(path: "assets/sprites/base.png")
        pipeTextureId = game.loadTexture(path: "assets/sprites/pipe-green.png")

        bird.initialize()
        scoreCounter.initialize(game: game)
    }

    func start() {
        bird.reset()
        pipes.removeAll()
        scoreCounter.resetScore()
        pipeSpawnTimer = 0
    }

    func update(deltaTime: Float) {
        if game.isKeyPressed(key: .ESCAPE) {
            game.requestClose()
            return
        }

        if game.isKeyPressed(key: .R) {
            resetGame()
            return
        }

        let didFlap = bird.update(deltaTime: deltaTime)
        if didFlap {
            let _ = game.audioPlay(data: EmbeddedAudioClips.flapWav)
        }

        // Ground collision
        if bird.y + Bird.height > Float(GameConstants.screenHeight) {
            resetGame()
            return
        }

        // Top of screen
        if bird.y < 0 {
            resetGame()
            return
        }

        // Update pipes and check collisions
        for pipe in pipes {
            pipe.update(deltaTime: deltaTime)

            if pipe.collidesWithBird(
                birdX: bird.x, birdY: bird.y,
                birdWidth: Bird.width, birdHeight: Bird.height
            ) {
                resetGame()
                return
            }
        }

        // Spawn new pipes
        pipeSpawnTimer += deltaTime
        if pipeSpawnTimer > GameConstants.pipeSpawnInterval {
            pipeSpawnTimer = 0
            pipes.append(Pipe(game: game))
        }

        // Remove off-screen pipes and increment score
        pipes.removeAll { pipe in
            if pipe.isOffScreen() {
                scoreCounter.incrementScore()
                return true
            }
            return false
        }

        draw()
    }

    private func draw() {
        // Layer 0: Background
        game.drawSprite(
            texture: backgroundTextureId,
            x: backgroundWidth / 2,
            y: backgroundHeight / 2,
            width: backgroundWidth,
            height: backgroundHeight
        )

        // Layer 1: Score
        scoreCounter.draw(game: game)

        // Layer 2: Pipes
        for pipe in pipes {
            pipe.draw(pipeTextureId: pipeTextureId)
        }

        // Layer 3: Bird
        bird.draw()

        // Layer 4: Base/ground
        game.drawSprite(
            texture: baseTextureId,
            x: baseWidth / 2,
            y: Float(GameConstants.screenHeight) + baseHeight / 2,
            width: baseWidth,
            height: baseHeight
        )
    }

    private func resetGame() {
        let _ = game.audioPlay(data: EmbeddedAudioClips.resetWav)
        scoreCounter.resetScore()
        start()
    }
}
