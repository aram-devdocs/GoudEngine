import XCTest
@testable import GoudEngine

final class WrapperTests: XCTestCase {

    // MARK: - GoudGame API surface

    func testGoudGameClassExists() {
        // Verify the type exists and is a reference type (class)
        let metatype: GoudGame.Type = GoudGame.self
        XCTAssertNotNil(metatype)
    }

    func testGoudGameMethodSignatures() {
        // Verify public method selectors exist on GoudGame via type-level checks.
        // We cannot call them without a GL context, but we can confirm the API surface
        // compiles by referencing the method types.
        let _: (GoudGame) -> () -> Bool = GoudGame.shouldClose
        let _: (GoudGame) -> () -> Void = GoudGame.requestClose
        let _: (GoudGame) -> (UInt32, UInt32) -> Bool = GoudGame.setWindowSize
        let _: (GoudGame) -> (String) -> UInt64 = GoudGame.loadTexture
        let _: (GoudGame) -> (UInt64) -> Void = GoudGame.destroyTexture
        let _: (GoudGame) -> (String) -> UInt64 = GoudGame.loadFont
        let _: (GoudGame) -> (UInt64) -> Bool = GoudGame.destroyFont
        let _: (GoudGame) -> () -> Entity = GoudGame.spawnEmpty
        let _: (GoudGame) -> () -> UInt32 = GoudGame.entityCount
        let _: (GoudGame) -> (Key) -> Bool = GoudGame.isKeyPressed
        let _: (GoudGame) -> (Key) -> Bool = GoudGame.isKeyJustPressed
        let _: (GoudGame) -> (Key) -> Bool = GoudGame.isKeyJustReleased
        let _: (GoudGame) -> (MouseButton) -> Bool = GoudGame.isMouseButtonPressed
        let _: (GoudGame) -> (MouseButton) -> Bool = GoudGame.isMouseButtonJustPressed
        let _: (GoudGame) -> (MouseButton) -> Bool = GoudGame.isMouseButtonJustReleased
    }

    func testGoudGameDeltaTimeProperty() {
        // deltaTime is a read-only Float property -- verify its type via a closure reference
        let _: (GoudGame) -> Float = { $0.deltaTime }
    }

    func testGoudGameDestroyMethodExists() {
        let _: (GoudGame) -> () -> Void = GoudGame.destroy
    }

    func testGoudGame3DMethodSignatures() {
        let _: (GoudGame) -> (UInt32, Float, Float, Float) -> UInt32 = GoudGame.createCube
        let _: (GoudGame) -> (UInt32, Float, Float) -> UInt32 = GoudGame.createPlane
        let _: (GoudGame) -> () -> Bool = GoudGame.render3D
    }

    func testGoudGameAudioMethodSignatures() {
        let _: (GoudGame) -> (Data) -> Int64 = GoudGame.audioPlay
        let _: (GoudGame) -> (UInt64) -> Int32 = GoudGame.audioStop
        let _: (GoudGame) -> (UInt64) -> Int32 = GoudGame.audioPause
        let _: (GoudGame) -> (UInt64) -> Int32 = GoudGame.audioResume
        let _: (GoudGame) -> () -> Int32 = GoudGame.audioStopAll
    }

    func testGoudGameCollisionMethodSignatures() {
        let _: (GoudGame) -> (Float, Float, Float, Float, Float, Float) -> Bool = GoudGame.pointInRect
        let _: (GoudGame) -> (Float, Float, Float, Float, Float) -> Bool = GoudGame.pointInCircle
    }

    func testGoudGameNetworkMethodSignatures() {
        let _: (GoudGame) -> (Int64) -> Int32 = GoudGame.networkDisconnect
        let _: (GoudGame) -> (Int64) -> Int32 = GoudGame.networkPoll
        let _: (GoudGame) -> (Int64) -> Int32 = GoudGame.networkPeerCount
    }

    // MARK: - EngineConfig API surface

    func testEngineConfigClassExists() {
        let metatype: EngineConfig.Type = EngineConfig.self
        XCTAssertNotNil(metatype)
    }

    func testEngineConfigBuilderReturnTypes() {
        // Verify builder methods return EngineConfig (discardable result pattern).
        // We check the method type signatures without calling them.
        let _: (EngineConfig) -> (String) -> EngineConfig = EngineConfig.setTitle
        let _: (EngineConfig) -> (UInt32, UInt32) -> EngineConfig = EngineConfig.setSize
        let _: (EngineConfig) -> (Bool) -> EngineConfig = EngineConfig.setVsync
        let _: (EngineConfig) -> (Bool) -> EngineConfig = EngineConfig.setFullscreen
        let _: (EngineConfig) -> (UInt32) -> EngineConfig = EngineConfig.setTargetFps
        let _: (EngineConfig) -> (Bool) -> EngineConfig = EngineConfig.setFpsOverlay
        let _: (EngineConfig) -> (Bool) -> EngineConfig = EngineConfig.setPhysicsDebug
    }

    func testEngineConfigBuildMethodExists() {
        let _: (EngineConfig) -> () -> GoudGame = EngineConfig.build
    }

    func testEngineConfigDestroyMethodExists() {
        let _: (EngineConfig) -> () -> Void = EngineConfig.destroy
    }

    // MARK: - DebuggerConfig

    func testDebuggerConfigDefaultInit() {
        let cfg = DebuggerConfig()
        XCTAssertEqual(cfg.enabled, false)
        XCTAssertEqual(cfg.publishLocalAttach, false)
        XCTAssertEqual(cfg.routeLabel, "")
    }

    func testDebuggerConfigCustomInit() {
        let cfg = DebuggerConfig(enabled: true, publishLocalAttach: true, routeLabel: "test-route")
        XCTAssertEqual(cfg.enabled, true)
        XCTAssertEqual(cfg.publishLocalAttach, true)
        XCTAssertEqual(cfg.routeLabel, "test-route")
    }

    func testDebuggerConfigEquality() {
        let a = DebuggerConfig(enabled: true, publishLocalAttach: false, routeLabel: "a")
        let b = DebuggerConfig(enabled: true, publishLocalAttach: false, routeLabel: "a")
        let c = DebuggerConfig(enabled: false, publishLocalAttach: false, routeLabel: "a")
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - ContextConfig

    func testContextConfigDefaultInit() {
        let cfg = ContextConfig()
        XCTAssertEqual(cfg.debugger, DebuggerConfig())
    }

    func testContextConfigCustomInit() {
        let dbg = DebuggerConfig(enabled: true, publishLocalAttach: true, routeLabel: "ctx")
        let cfg = ContextConfig(debugger: dbg)
        XCTAssertEqual(cfg.debugger.enabled, true)
        XCTAssertEqual(cfg.debugger.routeLabel, "ctx")
    }

    func testContextConfigEquality() {
        let a = ContextConfig(debugger: DebuggerConfig(enabled: true))
        let b = ContextConfig(debugger: DebuggerConfig(enabled: true))
        let c = ContextConfig()
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - RenderStats

    func testRenderStatsDefaultInit() {
        let stats = RenderStats()
        XCTAssertEqual(stats.drawCalls, 0)
        XCTAssertEqual(stats.triangles, 0)
        XCTAssertEqual(stats.textureBinds, 0)
        XCTAssertEqual(stats.shaderBinds, 0)
    }

    func testRenderStatsCustomInit() {
        let stats = RenderStats(drawCalls: 10, triangles: 500, textureBinds: 3, shaderBinds: 2)
        XCTAssertEqual(stats.drawCalls, 10)
        XCTAssertEqual(stats.triangles, 500)
        XCTAssertEqual(stats.textureBinds, 3)
        XCTAssertEqual(stats.shaderBinds, 2)
    }

    func testRenderStatsEquality() {
        let a = RenderStats(drawCalls: 1, triangles: 2, textureBinds: 3, shaderBinds: 4)
        let b = RenderStats(drawCalls: 1, triangles: 2, textureBinds: 3, shaderBinds: 4)
        let c = RenderStats()
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - FpsStats

    func testFpsStatsDefaultInit() {
        let stats = FpsStats()
        XCTAssertEqual(stats.currentFps, 0)
        XCTAssertEqual(stats.minFps, 0)
        XCTAssertEqual(stats.maxFps, 0)
        XCTAssertEqual(stats.avgFps, 0)
        XCTAssertEqual(stats.frameTimeMs, 0)
    }

    func testFpsStatsCustomInit() {
        let stats = FpsStats(currentFps: 60, minFps: 55, maxFps: 65, avgFps: 60, frameTimeMs: 16.67)
        XCTAssertEqual(stats.currentFps, 60, accuracy: 0.01)
        XCTAssertEqual(stats.minFps, 55, accuracy: 0.01)
        XCTAssertEqual(stats.maxFps, 65, accuracy: 0.01)
        XCTAssertEqual(stats.avgFps, 60, accuracy: 0.01)
        XCTAssertEqual(stats.frameTimeMs, 16.67, accuracy: 0.01)
    }

    func testFpsStatsEquality() {
        let a = FpsStats(currentFps: 60, minFps: 55, maxFps: 65, avgFps: 60, frameTimeMs: 16.67)
        let b = FpsStats(currentFps: 60, minFps: 55, maxFps: 65, avgFps: 60, frameTimeMs: 16.67)
        let c = FpsStats()
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - Contact

    func testContactDefaultInit() {
        let c = Contact()
        XCTAssertEqual(c.pointX, 0)
        XCTAssertEqual(c.pointY, 0)
        XCTAssertEqual(c.normalX, 0)
        XCTAssertEqual(c.normalY, 0)
        XCTAssertEqual(c.penetration, 0)
    }

    func testContactCustomInit() {
        let c = Contact(pointX: 1.5, pointY: 2.5, normalX: 0, normalY: 1, penetration: 0.3)
        XCTAssertEqual(c.pointX, 1.5, accuracy: 0.001)
        XCTAssertEqual(c.pointY, 2.5, accuracy: 0.001)
        XCTAssertEqual(c.normalX, 0, accuracy: 0.001)
        XCTAssertEqual(c.normalY, 1, accuracy: 0.001)
        XCTAssertEqual(c.penetration, 0.3, accuracy: 0.001)
    }

    func testContactEquality() {
        let a = Contact(pointX: 1, pointY: 2, normalX: 0, normalY: 1, penetration: 0.5)
        let b = Contact(pointX: 1, pointY: 2, normalX: 0, normalY: 1, penetration: 0.5)
        let c = Contact()
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - MemoryCategoryStats

    func testMemoryCategoryStatsDefaultInit() {
        let stats = MemoryCategoryStats()
        XCTAssertEqual(stats.currentBytes, 0)
        XCTAssertEqual(stats.peakBytes, 0)
    }

    func testMemoryCategoryStatsCustomInit() {
        let stats = MemoryCategoryStats(currentBytes: 1024, peakBytes: 4096)
        XCTAssertEqual(stats.currentBytes, 1024)
        XCTAssertEqual(stats.peakBytes, 4096)
    }

    func testMemoryCategoryStatsEquality() {
        let a = MemoryCategoryStats(currentBytes: 100, peakBytes: 200)
        let b = MemoryCategoryStats(currentBytes: 100, peakBytes: 200)
        let c = MemoryCategoryStats()
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - NetworkStats

    func testNetworkStatsDefaultInit() {
        let stats = NetworkStats()
        XCTAssertEqual(stats.bytesSent, 0)
        XCTAssertEqual(stats.bytesReceived, 0)
        XCTAssertEqual(stats.packetsSent, 0)
        XCTAssertEqual(stats.packetsReceived, 0)
        XCTAssertEqual(stats.packetsLost, 0)
        XCTAssertEqual(stats.rttMs, 0)
        XCTAssertEqual(stats.sendBandwidthBytesPerSec, 0)
        XCTAssertEqual(stats.receiveBandwidthBytesPerSec, 0)
        XCTAssertEqual(stats.packetLossPercent, 0)
        XCTAssertEqual(stats.jitterMs, 0)
    }

    func testNetworkStatsCustomInit() {
        let stats = NetworkStats(
            bytesSent: 1000,
            bytesReceived: 2000,
            packetsSent: 50,
            packetsReceived: 48,
            packetsLost: 2,
            rttMs: 25.5,
            sendBandwidthBytesPerSec: 5000,
            receiveBandwidthBytesPerSec: 10000,
            packetLossPercent: 4.0,
            jitterMs: 3.2
        )
        XCTAssertEqual(stats.bytesSent, 1000)
        XCTAssertEqual(stats.bytesReceived, 2000)
        XCTAssertEqual(stats.packetsSent, 50)
        XCTAssertEqual(stats.packetsReceived, 48)
        XCTAssertEqual(stats.packetsLost, 2)
        XCTAssertEqual(stats.rttMs, 25.5, accuracy: 0.01)
        XCTAssertEqual(stats.packetLossPercent, 4.0, accuracy: 0.01)
        XCTAssertEqual(stats.jitterMs, 3.2, accuracy: 0.01)
    }

    func testNetworkStatsEquality() {
        let a = NetworkStats(bytesSent: 100)
        let b = NetworkStats(bytesSent: 100)
        let c = NetworkStats(bytesSent: 200)
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - NetworkSimulationConfig

    func testNetworkSimulationConfigDefaultInit() {
        let cfg = NetworkSimulationConfig()
        XCTAssertEqual(cfg.oneWayLatencyMs, 0)
        XCTAssertEqual(cfg.jitterMs, 0)
        XCTAssertEqual(cfg.packetLossPercent, 0)
    }

    func testNetworkSimulationConfigCustomInit() {
        let cfg = NetworkSimulationConfig(oneWayLatencyMs: 50, jitterMs: 10, packetLossPercent: 5.0)
        XCTAssertEqual(cfg.oneWayLatencyMs, 50)
        XCTAssertEqual(cfg.jitterMs, 10)
        XCTAssertEqual(cfg.packetLossPercent, 5.0, accuracy: 0.01)
    }

    func testNetworkSimulationConfigEquality() {
        let a = NetworkSimulationConfig(oneWayLatencyMs: 10, jitterMs: 5, packetLossPercent: 1.0)
        let b = NetworkSimulationConfig(oneWayLatencyMs: 10, jitterMs: 5, packetLossPercent: 1.0)
        let c = NetworkSimulationConfig()
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    // MARK: - RigidBodyHandle

    func testRigidBodyHandleInit() {
        let h = RigidBodyHandle(bits: 42)
        XCTAssertEqual(h.bits, 42)
    }

    func testRigidBodyHandleEquality() {
        let a = RigidBodyHandle(bits: 100)
        let b = RigidBodyHandle(bits: 100)
        let c = RigidBodyHandle(bits: 200)
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    func testRigidBodyHandleHashable() {
        let a = RigidBodyHandle(bits: 7)
        let b = RigidBodyHandle(bits: 7)
        let set: Set<RigidBodyHandle> = [a, b]
        XCTAssertEqual(set.count, 1)
    }

    // MARK: - ColliderHandle

    func testColliderHandleInit() {
        let h = ColliderHandle(bits: 55)
        XCTAssertEqual(h.bits, 55)
    }

    func testColliderHandleEquality() {
        let a = ColliderHandle(bits: 300)
        let b = ColliderHandle(bits: 300)
        let c = ColliderHandle(bits: 400)
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    func testColliderHandleHashable() {
        let a = ColliderHandle(bits: 12)
        let b = ColliderHandle(bits: 12)
        let set: Set<ColliderHandle> = [a, b]
        XCTAssertEqual(set.count, 1)
    }

    // MARK: - TweenHandle

    func testTweenHandleInit() {
        let h = TweenHandle(bits: 99)
        XCTAssertEqual(h.bits, 99)
    }

    func testTweenHandleEquality() {
        let a = TweenHandle(bits: 500)
        let b = TweenHandle(bits: 500)
        let c = TweenHandle(bits: 600)
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    func testTweenHandleHashable() {
        let a = TweenHandle(bits: 3)
        let b = TweenHandle(bits: 3)
        let set: Set<TweenHandle> = [a, b]
        XCTAssertEqual(set.count, 1)
    }

    // MARK: - NetworkHandle

    func testNetworkHandleInit() {
        let h = NetworkHandle(bits: 1001)
        XCTAssertEqual(h.bits, 1001)
    }

    func testNetworkHandleEquality() {
        let a = NetworkHandle(bits: 777)
        let b = NetworkHandle(bits: 777)
        let c = NetworkHandle(bits: 888)
        XCTAssertEqual(a, b)
        XCTAssertNotEqual(a, c)
    }

    func testNetworkHandleHashable() {
        let a = NetworkHandle(bits: 50)
        let b = NetworkHandle(bits: 50)
        let set: Set<NetworkHandle> = [a, b]
        XCTAssertEqual(set.count, 1)
    }

    // MARK: - Additional value types

    func testAnimationEventDataDefaultInit() {
        let e = AnimationEventData()
        XCTAssertEqual(e.entity, 0)
        XCTAssertEqual(e.name, "")
        XCTAssertEqual(e.frameIndex, 0)
        XCTAssertEqual(e.payloadType, 0)
        XCTAssertEqual(e.payloadInt, 0)
        XCTAssertEqual(e.payloadFloat, 0)
        XCTAssertEqual(e.payloadString, "")
    }

    func testAnimationEventDataCustomInit() {
        let e = AnimationEventData(entity: 5, name: "jump", frameIndex: 3, payloadType: 1, payloadInt: 42, payloadFloat: 1.5, payloadString: "extra")
        XCTAssertEqual(e.entity, 5)
        XCTAssertEqual(e.name, "jump")
        XCTAssertEqual(e.frameIndex, 3)
        XCTAssertEqual(e.payloadType, 1)
        XCTAssertEqual(e.payloadInt, 42)
        XCTAssertEqual(e.payloadFloat, 1.5, accuracy: 0.001)
        XCTAssertEqual(e.payloadString, "extra")
    }

    func testPhysicsRaycastHit2DDefaultInit() {
        let h = PhysicsRaycastHit2D()
        XCTAssertEqual(h.bodyHandle, 0)
        XCTAssertEqual(h.colliderHandle, 0)
        XCTAssertEqual(h.pointX, 0)
        XCTAssertEqual(h.pointY, 0)
        XCTAssertEqual(h.normalX, 0)
        XCTAssertEqual(h.normalY, 0)
        XCTAssertEqual(h.distance, 0)
    }

    func testPhysicsCollisionEvent2DDefaultInit() {
        let e = PhysicsCollisionEvent2D()
        XCTAssertEqual(e.bodyA, 0)
        XCTAssertEqual(e.bodyB, 0)
        XCTAssertEqual(e.kind, 0)
    }

    func testMemorySummaryDefaultInit() {
        let s = MemorySummary()
        XCTAssertEqual(s.rendering, MemoryCategoryStats())
        XCTAssertEqual(s.assets, MemoryCategoryStats())
        XCTAssertEqual(s.ecs, MemoryCategoryStats())
        XCTAssertEqual(s.ui, MemoryCategoryStats())
        XCTAssertEqual(s.audio, MemoryCategoryStats())
        XCTAssertEqual(s.network, MemoryCategoryStats())
        XCTAssertEqual(s.debugger, MemoryCategoryStats())
        XCTAssertEqual(s.other, MemoryCategoryStats())
        XCTAssertEqual(s.totalCurrentBytes, 0)
        XCTAssertEqual(s.totalPeakBytes, 0)
    }

    func testDebuggerCaptureDefaultInit() {
        let c = DebuggerCapture()
        XCTAssertEqual(c.imagePng, Data())
        XCTAssertEqual(c.metadataJson, "")
        XCTAssertEqual(c.snapshotJson, "")
        XCTAssertEqual(c.metricsTraceJson, "")
    }

    func testDebuggerReplayArtifactDefaultInit() {
        let r = DebuggerReplayArtifact()
        XCTAssertEqual(r.manifestJson, "")
        XCTAssertEqual(r.data, Data())
    }

    func testVec3DefaultInit() {
        let v = Vec3()
        XCTAssertEqual(v.x, 0)
        XCTAssertEqual(v.y, 0)
        XCTAssertEqual(v.z, 0)
    }

    func testVec3StaticConstructors() {
        let zero = Vec3.zero()
        XCTAssertEqual(zero, Vec3(x: 0, y: 0, z: 0))

        let one = Vec3.one()
        XCTAssertEqual(one, Vec3(x: 1, y: 1, z: 1))

        let up = Vec3.up()
        XCTAssertEqual(up, Vec3(x: 0, y: 1, z: 0))
    }

    func testRenderCapabilitiesDefaultInit() {
        let caps = RenderCapabilities()
        XCTAssertEqual(caps.maxTextureUnits, 0)
        XCTAssertEqual(caps.maxTextureSize, 0)
        XCTAssertEqual(caps.supportsInstancing, false)
        XCTAssertEqual(caps.supportsCompute, false)
        XCTAssertEqual(caps.supportsMsaa, false)
    }

    func testPhysicsCapabilitiesDefaultInit() {
        let caps = PhysicsCapabilities()
        XCTAssertEqual(caps.supportsContinuousCollision, false)
        XCTAssertEqual(caps.supportsJoints, false)
        XCTAssertEqual(caps.maxBodies, 0)
    }

    func testAudioCapabilitiesDefaultInit() {
        let caps = AudioCapabilities()
        XCTAssertEqual(caps.supportsSpatial, false)
        XCTAssertEqual(caps.maxChannels, 0)
    }

    func testInputCapabilitiesDefaultInit() {
        let caps = InputCapabilities()
        XCTAssertEqual(caps.supportsGamepad, false)
        XCTAssertEqual(caps.supportsTouch, false)
        XCTAssertEqual(caps.maxGamepads, 0)
    }

    func testNetworkCapabilitiesDefaultInit() {
        let caps = NetworkCapabilities()
        XCTAssertEqual(caps.supportsHosting, false)
        XCTAssertEqual(caps.maxConnections, 0)
        XCTAssertEqual(caps.maxChannels, 0)
        XCTAssertEqual(caps.maxMessageSize, 0)
    }

    func testNetworkConnectResultDefaultInit() {
        let r = NetworkConnectResult()
        XCTAssertEqual(r.handle, 0)
        XCTAssertEqual(r.peerId, 0)
    }

    func testNetworkPacketDefaultInit() {
        let p = NetworkPacket()
        XCTAssertEqual(p.peerId, 0)
        XCTAssertEqual(p.data, Data())
    }

    // MARK: - Handle zero/max boundary values

    func testHandleZeroBits() {
        XCTAssertEqual(RigidBodyHandle(bits: 0).bits, 0)
        XCTAssertEqual(ColliderHandle(bits: 0).bits, 0)
        XCTAssertEqual(TweenHandle(bits: 0).bits, 0)
        XCTAssertEqual(NetworkHandle(bits: 0).bits, 0)
    }

    func testHandleMaxBits() {
        let max = UInt64.max
        XCTAssertEqual(RigidBodyHandle(bits: max).bits, max)
        XCTAssertEqual(ColliderHandle(bits: max).bits, max)
        XCTAssertEqual(TweenHandle(bits: max).bits, max)
        XCTAssertEqual(NetworkHandle(bits: max).bits, max)
    }

    func testHandleDifferentTypesNotInterchangeable() {
        // Different handle types with the same bits are distinct Swift types.
        // This is a compile-time guarantee -- we just verify they are separate types.
        let rigidType = RigidBodyHandle.self
        let colliderType = ColliderHandle.self
        let tweenType = TweenHandle.self
        let networkType = NetworkHandle.self
        XCTAssertFalse(rigidType == colliderType)
        XCTAssertFalse(tweenType == networkType)
    }
}
