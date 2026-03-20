// main.swift -- GoudEngine Feature Lab (Swift)
//
// Headless smoke test that exercises SDK surface without requiring a window.
// Prints pass/fail for each check and exits 0 on success, 1 on any failure.

import GoudEngine

struct CheckResult {
    let name: String
    let passed: Bool
}

var results: [CheckResult] = []

func record(_ name: String, _ passed: Bool) {
    results.append(CheckResult(name: name, passed: passed))
}

// -- Test: EngineConfig creation and setters ----------------------------------

let config = EngineConfig()
config
    .setTitle(title: "Swift Feature Lab")
    .setSize(width: 1280, height: 720)
    .setVsync(enabled: false)
    .setTargetFps(fps: 120)

record("EngineConfig creation and chained setters", true)

// -- Test: EngineConfig destroy without build ---------------------------------

let disposableConfig = EngineConfig()
disposableConfig.setTitle(title: "Disposable")
disposableConfig.destroy()
record("EngineConfig can be destroyed without building", true)

// -- Test: Error retrieval from FFI -------------------------------------------

let lastError = GoudError.fromLastError()
record("GoudError.fromLastError returns nil when no error is set", lastError == nil)

// -- Test: Value type construction --------------------------------------------

let vec = Vec2(x: 10.0, y: 20.0)
let color = Color(r: 0.5, g: 0.6, b: 0.7, a: 1.0)
let rect = Rect(x: 0.0, y: 0.0, width: 100.0, height: 50.0)

record("Vec2 construction preserves values", vec.x == 10.0 && vec.y == 20.0)
record("Color construction preserves values", color.r == 0.5 && color.a == 1.0)
record("Rect construction preserves values", rect.width == 100.0 && rect.height == 50.0)

// -- Test: Vec2 math helpers --------------------------------------------------

let a = Vec2(x: 3.0, y: 4.0)
let b = Vec2(x: 1.0, y: 2.0)
let sum = a.add(b)
let diff = a.sub(b)
let scaled = a.scale(2.0)
let length = a.length()

record("Vec2 add/sub/scale/length", sum.x == 4.0 && diff.y == 2.0 && scaled.x == 6.0 && length == 5.0)

// -- Test: Rect contains and intersects ---------------------------------------

let inside = Vec2(x: 50.0, y: 25.0)
let outside = Vec2(x: 200.0, y: 200.0)
let overlapping = Rect(x: 50.0, y: 25.0, width: 100.0, height: 50.0)
let disjoint = Rect(x: 200.0, y: 200.0, width: 10.0, height: 10.0)

record("Rect contains/intersects", rect.contains(inside) && !rect.contains(outside) && rect.intersects(overlapping) && !rect.intersects(disjoint))

// -- Test: Color helpers ------------------------------------------------------

let white = Color.white()
let withAlpha = white.withAlpha(0.5)

record("Color.white and withAlpha", white.r == 1.0 && white.a == 1.0 && withAlpha.a == 0.5)

// -- Summary ------------------------------------------------------------------

let passCount = results.filter { $0.passed }.count
let failCount = results.count - passCount

print("Swift Feature Lab complete: \(passCount) pass, \(failCount) fail")
for result in results {
    let status = result.passed ? "PASS" : "FAIL"
    print("\(status): \(result.name)")
}

if failCount > 0 {
    exit(1)
}
