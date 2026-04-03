// GameConstants.swift -- iOS Flappy Bird
// Portrait orientation: 288x512 game area + 112 base

import Foundation

enum GameConstants {
    static let targetFPS: UInt32 = 120
    static let baseHeight: Float = 112

    // Portrait dimensions matching the original Flappy Bird aspect ratio
    static let screenWidth: UInt32 = 288
    static let screenHeight: UInt32 = 512
    static let gravity: Float = 9.8
    static let jumpStrength: Float = -3.5
    static let jumpCooldown: Float = 0.30

    static let pipeSpeed: Float = 1.0
    static let pipeSpawnInterval: Float = 1.5
    static let pipeWidth: Float = 60
    static let pipeGap: Float = 100
}
