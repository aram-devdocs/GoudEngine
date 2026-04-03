// swift-tools-version: 5.9

import Foundation
import PackageDescription

let libSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_IOS_LIB_DIR"]
    ?? "../../../platform/ios/build/simulator"

let package = Package(
    name: "FlappyBirdiOS",
    platforms: [
        .iOS(.v16),
    ],
    dependencies: [
        .package(path: "../../../sdks/swift"),
    ],
    targets: [
        .executableTarget(
            name: "FlappyBird",
            dependencies: [
                .product(name: "GoudEngine", package: "swift"),
            ],
            path: "Sources/FlappyBird",
            linkerSettings: [
                .unsafeFlags(["-L", libSearchPath]),
            ]
        ),
    ]
)
