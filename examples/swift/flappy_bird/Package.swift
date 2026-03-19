// swift-tools-version: 5.9

import Foundation
import PackageDescription

let libSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_LIB_DIR"]
    ?? "../../../target/release"

let package = Package(
    name: "FlappyBird",
    platforms: [
        .macOS(.v13),
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
