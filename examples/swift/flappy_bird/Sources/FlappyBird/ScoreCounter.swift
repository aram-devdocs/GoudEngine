// ScoreCounter.swift

import GoudEngine

final class ScoreCounter {
    private(set) var score: Int = 0
    private var digitTextures: [UInt64] = Array(repeating: 0, count: 10)

    private var xOffset: Float = 0
    private var yOffset: Float = 50

    private let digitWidth: Float = 24
    private let digitHeight: Float = 36
    private let digitSpacing: Float = 30

    func initialize(game: GoudGame) {
        for i in 0..<10 {
            digitTextures[i] = game.loadTexture(path: "assets/sprites/\(i).png")
        }
        xOffset = Float(GameConstants.screenWidth) / 2 - 30
    }

    func incrementScore() {
        score += 1
    }

    func resetScore() {
        score = 0
    }

    func draw(game: GoudGame) {
        let scoreString = String(score)
        for (i, char) in scoreString.enumerated() {
            let digit = Int(String(char))!
            game.drawSprite(
                texture: digitTextures[digit],
                x: xOffset + Float(i) * digitSpacing + digitWidth / 2,
                y: yOffset + digitHeight / 2,
                width: digitWidth,
                height: digitHeight
            )
        }
    }
}
