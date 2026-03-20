// swift-tools-version: 5.9

import Foundation
import PackageDescription

let libSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_LIB_DIR"]
    ?? "../../../target/release"

let package = Package(
    name: "FeatureLab",
    platforms: [
        .macOS(.v13),
    ],
    dependencies: [
        .package(path: "../../../sdks/swift"),
    ],
    targets: [
        .executableTarget(
            name: "FeatureLab",
            dependencies: [
                .product(name: "GoudEngine", package: "swift"),
            ],
            path: "Sources/FeatureLab",
            linkerSettings: [
                .unsafeFlags(["-L", libSearchPath]),
            ]
        ),
    ]
)
