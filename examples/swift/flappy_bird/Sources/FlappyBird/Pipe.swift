// Pipe.swift

import Foundation
import GoudEngine

final class Pipe {
    private let game: GoudGame
    private(set) var x: Float
    let gapY: Float

    static let pipeWidth: Float = 52
    static let pipeHeight: Float = 320

    var topPipeY: Float { gapY - GameConstants.pipeGap - Pipe.pipeHeight }
    var bottomPipeY: Float { gapY + GameConstants.pipeGap }

    init(game: GoudGame) {
        self.game = game
        x = Float(GameConstants.screenWidth)
        gapY = Float(Int.random(
            in: Int(GameConstants.pipeGap)..<(Int(GameConstants.screenHeight) - Int(GameConstants.pipeGap))
        ))
    }

    func update(deltaTime: Float) {
        x -= GameConstants.pipeSpeed * deltaTime * Float(GameConstants.targetFPS)
    }

    func draw(pipeTextureId: UInt64) {
        // Top pipe (rotated 180 degrees)
        game.drawSprite(
            texture: pipeTextureId,
            x: x + Pipe.pipeWidth / 2,
            y: topPipeY + Pipe.pipeHeight / 2,
            width: Pipe.pipeWidth,
            height: Pipe.pipeHeight,
            rotation: Float.pi
        )

        // Bottom pipe (no rotation)
        game.drawSprite(
            texture: pipeTextureId,
            x: x + Pipe.pipeWidth / 2,
            y: bottomPipeY + Pipe.pipeHeight / 2,
            width: Pipe.pipeWidth,
            height: Pipe.pipeHeight,
            rotation: 0
        )
    }

    func isOffScreen() -> Bool {
        return x + GameConstants.pipeWidth < 0
    }

    func collidesWithBird(birdX: Float, birdY: Float, birdWidth: Float, birdHeight: Float) -> Bool {
        // Top pipe collision
        if checkAABB(
            x1: birdX, y1: birdY, w1: birdWidth, h1: birdHeight,
            x2: x, y2: topPipeY, w2: GameConstants.pipeWidth, h2: Pipe.pipeHeight
        ) {
            return true
        }

        // Bottom pipe collision
        if checkAABB(
            x1: birdX, y1: birdY, w1: birdWidth, h1: birdHeight,
            x2: x, y2: bottomPipeY, w2: GameConstants.pipeWidth, h2: Pipe.pipeHeight
        ) {
            return true
        }

        return false
    }

    private func checkAABB(
        x1: Float, y1: Float, w1: Float, h1: Float,
        x2: Float, y2: Float, w2: Float, h2: Float
    ) -> Bool {
        return x1 < x2 + w2 &&
               x1 + w1 > x2 &&
               y1 < y2 + h2 &&
               y1 + h1 > y2
    }
}
