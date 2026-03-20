// swift-tools-version: 5.9

import Foundation
import PackageDescription

let libSearchPath: String = ProcessInfo.processInfo.environment["GOUD_ENGINE_LIB_DIR"]
    ?? "../../../target/release"

let package = Package(
    name: "Sandbox",
    platforms: [
        .macOS(.v13),
    ],
    dependencies: [
        .package(path: "../../../sdks/swift"),
    ],
    targets: [
        .executableTarget(
            name: "Sandbox",
            dependencies: [
                .product(name: "GoudEngine", package: "swift"),
            ],
            path: "Sources/Sandbox",
            linkerSettings: [
                .unsafeFlags(["-L", libSearchPath]),
            ]
        ),
    ]
)
