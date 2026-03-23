import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import Module from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');
const require = createRequire(import.meta.url);

const {
  GoudGame,
  GoudContext,
  PhysicsWorld2D,
  PhysicsWorld3D,
  EngineConfig,
  UiManager,
} = require(path.join(repoRoot, 'dist', 'generated', 'node', 'index.g.js'));
const { DiagnosticMode } = require(path.join(repoRoot, 'dist', 'generated', 'diagnostic.g.js'));
const { Color } = require(path.join(repoRoot, 'dist', 'generated', 'types', 'math.g.js'));
const {
  NetworkManager,
  parseDebuggerManifest,
  parseDebuggerSnapshot,
} = require(path.join(repoRoot, 'dist', 'index.js'));

function makeProxyNative(overrides = {}, calls = []) {
  return new Proxy(overrides, {
    get(target, prop) {
      if (prop in target) return target[prop];
      return (...args) => {
        calls.push([prop, args]);
        return 0;
      };
    },
  });
}

function makeGameForTests(native) {
  const game = Object.create(GoudGame.prototype);
  game.native = native;
  game.preloadedTextures = new Map();
  game.preloadedFonts = new Map();
  game.texturePathByHandle = new Map();
  game.fontPathByHandle = new Map();
  game.preloadInFlight = false;
  return game;
}

describe('Generated node wrappers runtime coverage (fake native)', () => {
  it('executes GoudGame forwarding methods and conversion paths', async () => {
    const calls = [];
    let shouldCloseCalls = 0;

    const native = makeProxyNative(
      {
        deltaTime: 0.125,
        fps: 120,
        windowWidth: 640,
        windowHeight: 360,
        title: 'fake',
        totalTime: 9,
        frameCount: 10,
        interpolationAlpha: 0.5,
        shouldClose() {
          shouldCloseCalls += 1;
          return shouldCloseCalls > 1;
        },
        beginFrame(...args) {
          calls.push(['beginFrame', args]);
        },
        endFrame(...args) {
          calls.push(['endFrame', args]);
        },
        loadTexture: async (p) => {
          calls.push(['loadTexture', [p]]);
          return p.length;
        },
        loadFont: async (p) => {
          calls.push(['loadFont', [p]]);
          return p.length + 100;
        },
        getMousePosition: () => [3, 4],
        getMouseDelta: () => [5, 6],
        getScrollDelta: () => [7, 8],
        getDebuggerSnapshotJson: () => '{"version":1}',
        getDebuggerManifestJson: () => '{"manifest_version":1}',
        captureDebuggerFrame: () => ({
          imagePng: new Uint8Array([1, 2, 3]),
          metadataJson: '{"route":"game"}',
          snapshotJson: '{"frame":7}',
          metricsTraceJson: '{"version":1}',
        }),
        stopDebuggerRecording: () => ({
          manifestJson: '{"determinism":"limited"}',
          data: new Uint8Array([9, 8, 7]),
        }),
        getDebuggerReplayStatusJson: () => '{"state":"idle"}',
        getDebuggerMetricsTraceJson: () => '{"frames":[]}',
        getMemorySummary: () => ({
          rendering: { current_bytes: 1, peak_bytes: 2 },
          assets: { current_bytes: 3, peak_bytes: 4 },
          ecs: { current_bytes: 5, peak_bytes: 6 },
          ui: { current_bytes: 7, peak_bytes: 8 },
          audio: { current_bytes: 9, peak_bytes: 10 },
          network: { current_bytes: 11, peak_bytes: 12 },
          debugger: { current_bytes: 13, peak_bytes: 14 },
          other: { current_bytes: 15, peak_bytes: 16 },
          total_current_bytes: 64,
          total_peak_bytes: 72,
        }),
        spawnBatch: () => [11, 12, 13],
        getSprite: () => ({ flipX: true, flipY: false, zLayer: 7 }),
        networkSend(handle, peerId, data, channel) {
          calls.push(['networkSend', [handle, peerId, data, channel]]);
          return data.length + channel;
        },
        setNetworkSimulation(handle, config) {
          calls.push(['setNetworkSimulation', [handle, config]]);
          return handle;
        },
      },
      calls,
    );

    const game = makeGameForTests(native);

    assert.equal(game.deltaTime, 0.125);
    assert.equal(game.fps, 120);
    assert.equal(game.windowWidth, 640);
    assert.equal(game.windowHeight, 360);
    assert.equal(game.title, 'fake');
    assert.equal(game.totalTime, 9);
    assert.equal(game.frameCount, 10);

    game.run((dt) => {
      assert.equal(dt, 0.125);
    });

    // Reset shouldClose counter for runWithFixedUpdate test
    shouldCloseCalls = 0;
    game.runWithFixedUpdate(
      (dt) => { /* fixedUpdate */ },
      (dt) => { /* update */ },
    );

    assert.deepEqual(game.getMousePosition(), { x: 3, y: 4 });
    assert.deepEqual(game.getMouseDelta(), { x: 5, y: 6 });
    assert.deepEqual(game.getScrollDelta(), { x: 7, y: 8 });

    const tex = await game.loadTexture('a.png');
    assert.equal(tex, 5);
    const texCached = await game.loadTexture('a.png');
    assert.equal(texCached, 5);

    const font = await game.loadFont('font.ttf');
    assert.equal(font, 108);
    const fontCached = await game.loadFont('font.ttf');
    assert.equal(fontCached, 108);

    game.destroyTexture(tex);
    game.destroyFont(font);

    assert.equal(game.drawText(1, 'hi', 0, 0), 0);
    game.drawSprite(1, 1, 2, 3, 4, 0.5);
    game.drawQuad(1, 2, 3, 4);

    assert.equal(game.isKeyPressed(1), 0);
    assert.equal(game.isKeyJustPressed(1), 0);
    assert.equal(game.isKeyJustReleased(1), 0);
    assert.equal(game.isMouseButtonPressed(1), 0);
    assert.equal(game.isMouseButtonJustPressed(1), 0);
    assert.equal(game.isMouseButtonJustReleased(1), 0);

    assert.equal(game.spawnEmpty(), 0);
    assert.equal(game.despawn(1), 0);
    assert.equal(game.cloneEntity(1), 0);
    assert.equal(game.cloneEntityRecursive(1), 0);
    assert.equal(game.entityCount(), 0);
    assert.equal(game.isAlive(1), 0);

    game.addTransform2d(1, { positionX: 1, positionY: 2, rotation: 0, scaleX: 1, scaleY: 1 });
    assert.equal(game.getTransform2d(1), 0);
    game.setTransform2d(1, { positionX: 3, positionY: 4, rotation: 0, scaleX: 1, scaleY: 1 });
    assert.equal(game.hasTransform2d(1), 0);
    assert.equal(game.removeTransform2d(1), 0);

    game.addName(1, 'e');
    assert.equal(game.getName(1), 0);
    assert.equal(game.hasName(1), 0);
    assert.equal(game.removeName(1), 0);

    game.addSprite(1, {});
    assert.deepEqual(game.getSprite(1), { flipX: true, flipY: false, zLayer: 7 });
    game.setSprite(1, {});
    assert.equal(game.hasSprite(1), 0);
    assert.equal(game.removeSprite(1), 0);

    assert.deepEqual(game.spawnBatch(3), [11, 12, 13]);
    assert.equal(game.despawnBatch([1, 2]), 0);

    assert.equal(game.play(1), 0);
    assert.equal(game.stop(1), 0);
    assert.equal(game.setState(1, 'idle'), 0);
    assert.equal(game.setParameterBool(1, 'a', true), 0);
    assert.equal(game.setParameterFloat(1, 'a', 1.2), 0);

    assert.equal(game.createCube(1, 1, 1, 1), 0);
    assert.equal(game.createPlane(1, 1, 1), 0);
    assert.equal(game.createSphere(1, 2), 0);
    assert.equal(game.createCylinder(1, 2, 3), 0);
    assert.equal(game.setObjectPosition(1, 1, 2, 3), 0);
    assert.equal(game.setObjectRotation(1, 1, 2, 3), 0);
    assert.equal(game.setObjectScale(1, 1, 2, 3), 0);
    assert.equal(game.destroyObject(1), 0);
    assert.equal(game.addLight(0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 30), 0);
    assert.equal(game.updateLight(1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 30), 0);
    assert.equal(game.removeLight(1), 0);
    assert.equal(game.setCameraPosition3D(1, 2, 3), 0);
    assert.equal(game.setCameraRotation3D(1, 2, 3), 0);
    assert.equal(game.configureGrid(true, 10, 10), 0);
    assert.equal(game.setGridEnabled(true), 0);
    assert.equal(game.configureSkybox(true, 0, 0, 0, 1), 0);
    assert.equal(game.configureFog(true, 0, 0, 0, 0.1), 0);
    assert.equal(game.setFogEnabled(true), 0);
    assert.equal(game.render3D(), 0);

    assert.equal(game.drawSpriteRect(1, 0, 0, 10, 10, 0, 0, 0, 5, 5), 0);
    game.setViewport(0, 0, 100, 100);
    game.enableDepthTest();
    game.disableDepthTest();
    game.clearDepth();
    game.disableBlending();
    assert.equal(game.getRenderStats(), 0);
    assert.equal(game.getFpsStats(), 0);
    game.setFpsOverlayEnabled(true);
    game.setFpsUpdateInterval(0.25);
    game.setFpsOverlayCorner(1);

    assert.equal(game.mapActionKey('jump', 32), 0);
    assert.equal(game.isActionPressed('jump'), 0);
    assert.equal(game.isActionJustPressed('jump'), 0);
    assert.equal(game.isActionJustReleased('jump'), 0);

    assert.equal(game.collisionAabbAabb(0, 0, 1, 1, 2, 2, 1, 1), 0);
    assert.equal(game.collisionCircleCircle(0, 0, 1, 2, 2, 1), 0);
    assert.equal(game.collisionCircleAabb(0, 0, 1, 0, 0, 1, 1), 0);
    assert.equal(game.pointInRect(0, 0, 0, 0, 1, 1), 0);
    assert.equal(game.pointInCircle(0, 0, 0, 0, 1), 0);
    assert.equal(game.aabbOverlap(0, 0, 1, 1, 0, 0, 1, 1), 0);
    assert.equal(game.circleOverlap(0, 0, 1, 0, 0, 1), 0);
    assert.equal(game.distance(0, 0, 1, 1), 0);
    assert.equal(game.distanceSquared(0, 0, 1, 1), 0);

    assert.equal(game.physicsRaycastEx(0, 0, 1, 1, 100, 0xffff), 0);
    assert.equal(game.physicsCollisionEventsCount(), 0);
    assert.equal(game.physicsCollisionEventsRead(0), 0);
    assert.equal(game.physicsSetCollisionCallback(0, 0), 0);

    assert.equal(game.getRenderCapabilities(), 0);
    assert.equal(game.getPhysicsCapabilities(), 0);
    assert.equal(game.getAudioCapabilities(), 0);
    assert.equal(game.getInputCapabilities(), 0);
    assert.equal(game.getNetworkCapabilities(), 0);

    assert.equal(game.networkHost(1, 9000), 0);
    assert.equal(game.networkConnect(1, '127.0.0.1', 9000), 0);
    assert.equal(game.networkConnectWithPeer(1, '127.0.0.1', 9000), 0);
    assert.equal(game.networkDisconnect(9), 0);
    assert.equal(game.networkSend(9, 2, new Uint8Array([1, 2, 3]), 1), 4);
    assert.equal(game.networkReceive(9), 0);
    assert.equal(game.networkReceivePacket(9), 0);
    assert.equal(game.networkPoll(9), 0);
    assert.equal(game.getNetworkStats(9), 0);
    assert.equal(game.networkPeerCount(9), 0);
    assert.equal(game.setNetworkSimulation(9, { oneWayLatencyMs: 10, jitterMs: 2, packetLossPercent: 1 }), 9);
    assert.equal(game.clearNetworkSimulation(9), 0);
    assert.equal(game.setNetworkOverlayHandle(9), 0);
    assert.equal(game.clearNetworkOverlayHandle(), 0);
    assert.equal(game.getDebuggerSnapshotJson(), '{"version":1}');
    assert.equal(game.getDebuggerManifestJson(), '{"manifest_version":1}');
    game.setDebuggerPaused(true);
    game.stepDebugger(1, 2);
    game.setDebuggerTimeScale(0.5);
    game.setDebuggerDebugDrawEnabled(true);
    game.injectDebuggerKeyEvent(4, true);
    game.injectDebuggerMouseButton(1, false);
    game.injectDebuggerMousePosition({ x: 12, y: 13 });
    game.injectDebuggerScroll({ x: -1, y: 2 });
    game.setDebuggerProfilingEnabled(true);
    game.setDebuggerSelectedEntity(77);
    game.clearDebuggerSelectedEntity();
    assert.deepEqual(game.captureDebuggerFrame(), {
      imagePng: new Uint8Array([1, 2, 3]),
      metadataJson: '{"route":"game"}',
      snapshotJson: '{"frame":7}',
      metricsTraceJson: '{"version":1}',
    });
    game.startDebuggerRecording();
    assert.deepEqual(game.stopDebuggerRecording(), {
      manifestJson: '{"determinism":"limited"}',
      data: new Uint8Array([9, 8, 7]),
    });
    game.startDebuggerReplay(new Uint8Array([5, 4]));
    game.stopDebuggerReplay();
    assert.equal(game.getDebuggerReplayStatusJson(), '{"state":"idle"}');
    assert.equal(game.getDebuggerMetricsTraceJson(), '{"frames":[]}');
    assert.deepEqual(game.getMemorySummary(), {
      rendering: { currentBytes: 1, peakBytes: 2 },
      assets: { currentBytes: 3, peakBytes: 4 },
      ecs: { currentBytes: 5, peakBytes: 6 },
      ui: { currentBytes: 7, peakBytes: 8 },
      audio: { currentBytes: 9, peakBytes: 10 },
      network: { currentBytes: 11, peakBytes: 12 },
      debugger: { currentBytes: 13, peakBytes: 14 },
      other: { currentBytes: 15, peakBytes: 16 },
      totalCurrentBytes: 64,
      totalPeakBytes: 72,
    });

    assert.equal(game.audioPlay(Buffer.from([1])), 0);
    assert.equal(game.audioPlayOnChannel(Buffer.from([1]), 2), 0);
    assert.equal(game.audioPlayWithSettings(Buffer.from([1]), 0.5, 1.2, false, 1), 0);
    assert.equal(game.audioStop(1), 0);
    assert.equal(game.audioPause(1), 0);
    assert.equal(game.audioResume(1), 0);
    assert.equal(game.audioStopAll(), 0);
    assert.equal(game.audioSetGlobalVolume(0.9), 0);
    assert.equal(game.audioGetGlobalVolume(), 0);
    assert.equal(game.audioSetChannelVolume(1, 0.4), 0);
    assert.equal(game.audioGetChannelVolume(1), 0);
    assert.equal(game.audioIsPlaying(1), 0);
    assert.equal(game.audioActiveCount(), 0);
    assert.equal(game.audioCleanupFinished(), 0);
    assert.equal(game.audioPlaySpatial3d(Buffer.from([1]), 0, 0, 0, 1, 1, 1, 10, 1), 0);
    assert.equal(game.audioUpdateSpatial3d(1, 0, 0, 0, 1, 1, 1, 10, 1), 0);
    assert.equal(game.audioSetListenerPosition3d(0, 1, 2), 0);
    assert.equal(game.audioSetSourcePosition3d(1, 2, 3, 4, 10, 1), 0);
    assert.equal(game.audioSetPlayerVolume(1, 0.2), 0);
    assert.equal(game.audioSetPlayerSpeed(1, 1.1), 0);
    assert.equal(game.audioCrossfade(1, 2, 0.3), 0);
    assert.equal(game.audioCrossfadeTo(1, Buffer.from([1]), 1.2, 2), 0);
    assert.equal(game.audioMixWith(1, Buffer.from([1]), 0.1, 2), 0);
    assert.equal(game.audioUpdateCrossfades(0.016), 0);
    assert.equal(game.audioActiveCrossfadeCount(), 0);
    assert.equal(game.audioActivate(), 0);

    assert.equal(game.interpolationAlpha, 0.5);
    game.setFixedTimestep(0.016);
    game.setMaxFixedSteps(8);
    assert.equal(game.drawTextBatch([{ fontHandle: 1, text: 'hi', x: 0, y: 0, fontSize: 16, alignment: 0, direction: 0, maxWidth: 0, lineSpacing: 1, r: 1, g: 1, b: 1, a: 1 }]), 0);
    assert.equal(game.componentCount(1), 0);
    assert.equal(game.componentGetEntities(1, 0, 10), 0);
    assert.equal(game.componentGetAll(1, 0, 0, 10), 0);

    // Phase 0: SpriteBatch, RenderMetrics (TextBatch, FixedTimestep, GenericComponent already covered above)
    assert.equal(game.drawSpriteBatch([{ texture: 1, x: 0, y: 0, width: 32, height: 32, rotation: 0, srcX: 0, srcY: 0, srcW: 0, srcH: 0, r: 1, g: 1, b: 1, a: 1, zLayer: 0 }]), 0);
    assert.equal(game.getRenderMetrics(), 0);

    assert.equal(game.checkHotSwapShortcut(), 0);
    assert.equal(game.loadScene('s', '{}'), 0);
    assert.equal(game.unloadScene('s'), 0);
    assert.equal(game.setActiveScene(1, true), 0);

    assert.equal(game.animationLayerStackCreate(1), 0);
    assert.equal(game.animationLayerAdd(1, 'base', 0), 0);
    assert.equal(game.animationLayerSetWeight(1, 0, 0.5), 0);
    assert.equal(game.animationLayerPlay(1, 0), 0);
    assert.equal(game.animationLayerSetClip(1, 0, 4, 0.1, 0), 0);
    assert.equal(game.animationLayerAddFrame(1, 0, 0, 0, 16, 16), 0);
    assert.equal(game.animationLayerReset(1, 0), 0);
    assert.equal(game.animationClipAddEvent(1, 0, 'evt', 0, 1, 2.0, 'payload'), 0);
    assert.equal(game.animationEventsCount(), 0);
    assert.equal(game.animationEventsRead(0), 0);

    const sendCall = calls.find(([name]) => name === 'networkSend');
    assert.ok(sendCall);
    assert.ok(sendCall[1][2] instanceof Uint8Array);

    const simCall = calls.find(([name]) => name === 'setNetworkSimulation');
    assert.deepEqual(simCall[1][1], {
      oneWayLatencyMs: 10,
      jitterMs: 2,
      packetLossPercent: 1,
    });

    const replayStartCall = calls.find(([name]) => name === 'startDebuggerReplay');
    assert.ok(replayStartCall);
    assert.deepEqual(Array.from(replayStartCall[1][0]), [5, 4]);

    assert.deepEqual(parseDebuggerSnapshot(game), { version: 1 });
    assert.deepEqual(parseDebuggerManifest(game), { manifest_version: 1 });
  });

  it('covers preload normalization and in-flight guards', async () => {
    const native = makeProxyNative({
      loadTexture: async () => 42,
      loadFont: async () => 77,
      shouldClose: () => true,
    });

    const game = makeGameForTests(native);
    const progress = [];
    const handles = await game.preload([
      'sprite.png',
      { path: 'font.ttf' },
      { path: 'icon.webp', kind: 'texture' },
    ], {
      onProgress(update) {
        progress.push(update);
      },
    });

    assert.equal(handles['sprite.png'], 42);
    assert.equal(handles['font.ttf'], 77);
    assert.equal(handles['icon.webp'], 42);
    assert.equal(progress.length, 3);
    assert.equal(progress[2].progress, 1);

    await assert.rejects(
      () => game.preload(['bad.xyz']),
      /Unsupported preload asset type/,
    );

    game.preloadInFlight = true;
    assert.throws(
      () => game.run(() => {}),
      /must finish before game.run\(\) starts/,
    );
  });

  it('executes GoudContext forwarding and conversion paths', () => {
    const calls = [];
    const native = makeProxyNative(
      {
        getDebuggerSnapshotJson: () => '{"context":true}',
        getDebuggerManifestJson: () => '{"manifest":true}',
        captureDebuggerFrame: () => JSON.stringify({
          imagePng: [6, 5],
          metadataJson: '{"route":"context"}',
          snapshotJson: '{"frame":9}',
          metricsTraceJson: '{"version":2}',
        }),
        stopDebuggerRecording: () => JSON.stringify({
          manifestJson: '{"determinism":"documented"}',
          data: [1, 3, 5],
        }),
        getDebuggerReplayStatusJson: () => '{"state":"replaying"}',
        getDebuggerMetricsTraceJson: () => '{"frames":[1]}',
        getMemorySummary: () => ({
          rendering: { current_bytes: 2, peak_bytes: 3 },
          assets: { current_bytes: 4, peak_bytes: 5 },
          ecs: { current_bytes: 6, peak_bytes: 7 },
          ui: { current_bytes: 8, peak_bytes: 9 },
          audio: { current_bytes: 10, peak_bytes: 11 },
          network: { current_bytes: 12, peak_bytes: 13 },
          debugger: { current_bytes: 14, peak_bytes: 15 },
          other: { current_bytes: 16, peak_bytes: 17 },
          total_current_bytes: 72,
          total_peak_bytes: 80,
        }),
        networkSend(handle, peerId, data, channel) {
          calls.push(['networkSend', [handle, peerId, data, channel]]);
          return channel;
        },
        setNetworkSimulation(handle, config) {
          calls.push(['setNetworkSimulation', [handle, config]]);
          return handle;
        },
      },
      calls,
    );

    const ctx = Object.create(GoudContext.prototype);
    ctx.native = native;

    assert.equal(ctx.destroy(), 0);
    assert.equal(ctx.isValid(), 0);
    assert.equal(ctx.getNetworkCapabilities(), 0);
    assert.equal(ctx.networkHost(1, 9000), 0);
    assert.equal(ctx.networkConnect(1, '127.0.0.1', 9000), 0);
    assert.equal(ctx.networkConnectWithPeer(1, '127.0.0.1', 9000), 0);
    assert.equal(ctx.networkDisconnect(9), 0);
    assert.equal(ctx.networkSend(9, 2, new Uint8Array([9, 8]), 3), 3);
    assert.equal(ctx.networkReceive(9), 0);
    assert.equal(ctx.networkReceivePacket(9), 0);
    assert.equal(ctx.networkPoll(9), 0);
    assert.equal(ctx.getNetworkStats(9), 0);
    assert.equal(ctx.networkPeerCount(9), 0);
    assert.equal(ctx.setNetworkSimulation(9, { oneWayLatencyMs: 5, jitterMs: 1, packetLossPercent: 0 }), 9);
    assert.equal(ctx.clearNetworkSimulation(9), 0);
    assert.equal(ctx.setNetworkOverlayHandle(9), 0);
    assert.equal(ctx.clearNetworkOverlayHandle(), 0);
    assert.equal(ctx.getDebuggerSnapshotJson(), '{"context":true}');
    assert.equal(ctx.getDebuggerManifestJson(), '{"manifest":true}');
    ctx.setDebuggerPaused(false);
    ctx.stepDebugger(0, 1);
    ctx.setDebuggerTimeScale(1.5);
    ctx.setDebuggerDebugDrawEnabled(false);
    ctx.injectDebuggerKeyEvent(7, false);
    ctx.injectDebuggerMouseButton(2, true);
    ctx.injectDebuggerMousePosition({ x: 4, y: 5 });
    ctx.injectDebuggerScroll({ x: 0, y: -3 });
    ctx.setDebuggerProfilingEnabled(false);
    ctx.setDebuggerSelectedEntity(19);
    ctx.clearDebuggerSelectedEntity();
    assert.deepEqual(ctx.captureDebuggerFrame(), {
      imagePng: new Uint8Array([6, 5]),
      metadataJson: '{"route":"context"}',
      snapshotJson: '{"frame":9}',
      metricsTraceJson: '{"version":2}',
    });
    ctx.startDebuggerRecording();
    assert.deepEqual(ctx.stopDebuggerRecording(), {
      manifestJson: '{"determinism":"documented"}',
      data: new Uint8Array([1, 3, 5]),
    });
    ctx.startDebuggerReplay(new Uint8Array([2, 4, 6]));
    ctx.stopDebuggerReplay();
    assert.equal(ctx.getDebuggerReplayStatusJson(), '{"state":"replaying"}');
    assert.equal(ctx.getDebuggerMetricsTraceJson(), '{"frames":[1]}');
    assert.deepEqual(ctx.getMemorySummary(), {
      rendering: { currentBytes: 2, peakBytes: 3 },
      assets: { currentBytes: 4, peakBytes: 5 },
      ecs: { currentBytes: 6, peakBytes: 7 },
      ui: { currentBytes: 8, peakBytes: 9 },
      audio: { currentBytes: 10, peakBytes: 11 },
      network: { currentBytes: 12, peakBytes: 13 },
      debugger: { currentBytes: 14, peakBytes: 15 },
      other: { currentBytes: 16, peakBytes: 17 },
      totalCurrentBytes: 72,
      totalPeakBytes: 80,
    });

    const sendCall = calls.find(([name]) => name === 'networkSend');
    assert.ok(sendCall[1][2] instanceof Uint8Array);

    const simCall = calls.find(([name]) => name === 'setNetworkSimulation');
    assert.deepEqual(simCall[1][1], {
      oneWayLatencyMs: 5,
      jitterMs: 1,
      packetLossPercent: 0,
    });

    const contextReplayStartCall = calls.find(([name]) => name === 'startDebuggerReplay');
    assert.ok(contextReplayStartCall);
    assert.deepEqual(Array.from(contextReplayStartCall[1][0]), [2, 4, 6]);

    assert.deepEqual(parseDebuggerSnapshot(ctx), { context: true });
    assert.deepEqual(parseDebuggerManifest(ctx), { manifest: true });
  });

  it('executes PhysicsWorld2D, PhysicsWorld3D, EngineConfig, and UiManager wrappers', () => {
    const world2d = Object.create(PhysicsWorld2D.prototype);
    world2d.native = makeProxyNative();

    assert.equal(world2d.create(0, -9.8), 0);
    assert.equal(world2d.createWithBackend(0, -9.8, 1), 0);
    assert.equal(world2d.destroy(), 0);
    assert.equal(world2d.setGravity(0, -9.8), 0);
    assert.equal(world2d.addRigidBody(0, 0, 0, 1), 0);
    assert.equal(world2d.addRigidBodyEx(0, 0, 0, 1, true), 0);
    assert.equal(world2d.addCollider(1, 0, 1, 1, 0, 0.5, 0.2), 0);
    assert.equal(world2d.addColliderEx(1, 0, 1, 1, 0, 0.5, 0.2, false, 1, 0xffff), 0);
    assert.equal(world2d.removeBody(1), 0);
    assert.equal(world2d.createJoint(1, 2, 0, 0, 0, 1, 1, 1, 0, false, 0, 0, false, 0, 0), 0);
    assert.equal(world2d.removeJoint(1), 0);
    assert.equal(world2d.step(0.016), 0);
    assert.equal(world2d.getPosition(1), 0);
    assert.equal(world2d.getVelocity(1), 0);
    assert.equal(world2d.setVelocity(1, 1, 2), 0);
    assert.equal(world2d.applyForce(1, 1, 2), 0);
    assert.equal(world2d.applyImpulse(1, 1, 2), 0);
    assert.equal(world2d.raycast(0, 0, 1, 0, 100), 0);
    assert.equal(world2d.raycastEx(0, 0, 1, 0, 100, 0xffff), 0);
    assert.equal(world2d.collisionEventsCount(), 0);
    assert.equal(world2d.collisionEventsRead(0), 0);
    assert.equal(world2d.collisionEventCount(), 0);
    assert.equal(world2d.collisionEventRead(0), 0);
    assert.equal(world2d.setCollisionCallback(0, 0), 0);
    assert.equal(world2d.getGravity(), 0);
    assert.equal(world2d.setBodyGravityScale(1, 1), 0);
    assert.equal(world2d.getBodyGravityScale(1), 0);
    assert.equal(world2d.setColliderFriction(1, 0.5), 0);
    assert.equal(world2d.getColliderFriction(1), 0);
    assert.equal(world2d.setColliderRestitution(1, 0.2), 0);
    assert.equal(world2d.getColliderRestitution(1), 0);
    assert.equal(world2d.setTimestep(0.016), 0);
    assert.equal(world2d.getTimestep(), 0);

    const world3d = Object.create(PhysicsWorld3D.prototype);
    world3d.native = makeProxyNative();

    assert.equal(world3d.create(0, -9.8, 0), 0);
    assert.equal(world3d.destroy(), 0);
    assert.equal(world3d.setGravity(0, -9.8, 0), 0);
    assert.equal(world3d.addRigidBody(0, 0, 0, 0, 1), 0);
    assert.equal(world3d.addRigidBodyEx(0, 0, 0, 0, 1, true), 0);
    assert.equal(world3d.addCollider(1, 0, 1, 1, 1, 0, 0.5, 0.2), 0);
    assert.equal(world3d.removeBody(1), 0);
    assert.equal(world3d.createJoint(1, 2, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, false, 0, 0, false, 0, 0), 0);
    assert.equal(world3d.removeJoint(1), 0);
    assert.equal(world3d.step(0.016), 0);
    assert.equal(world3d.getPosition(1), 0);
    assert.equal(world3d.setVelocity(1, 1, 2, 3), 0);
    assert.equal(world3d.applyForce(1, 1, 2, 3), 0);
    assert.equal(world3d.applyImpulse(1, 1, 2, 3), 0);
    assert.equal(world3d.getGravity(), 0);
    assert.equal(world3d.setBodyGravityScale(1, 1), 0);
    assert.equal(world3d.getBodyGravityScale(1), 0);
    assert.equal(world3d.setColliderFriction(1, 0.5), 0);
    assert.equal(world3d.getColliderFriction(1), 0);
    assert.equal(world3d.setColliderRestitution(1, 0.2), 0);
    assert.equal(world3d.getColliderRestitution(1), 0);
    assert.equal(world3d.setTimestep(0.016), 0);
    assert.equal(world3d.getTimestep(), 0);

    const config = Object.create(EngineConfig.prototype);
    const fakeBuiltNative = makeProxyNative();
    config.native = {
      setTitle: () => {},
      setSize: () => {},
      setVsync: () => {},
      setFullscreen: () => {},
      setTargetFps: () => {},
      setFpsOverlay: () => {},
      setPhysicsDebug: () => {},
      setPhysicsBackend2D: () => {},
      setDebugger: () => {},
      build: () => fakeBuiltNative,
      destroy: () => {},
    };

    assert.equal(config.setTitle('x'), config);
    assert.equal(config.setSize(10, 20), config);
    assert.equal(config.setVsync(true), config);
    assert.equal(config.setFullscreen(false), config);
    assert.equal(config.setTargetFps(60), config);
    assert.equal(config.setFpsOverlay(true), config);
    assert.equal(config.setPhysicsDebug(false), config);
    assert.equal(config.setPhysicsBackend2D(1), config);
    assert.equal(config.setDebugger({ enabled: true, publishLocalAttach: false, routeLabel: 'test' }), config);

    const builtGame = config.build();
    assert.equal(Object.getPrototypeOf(builtGame), GoudGame.prototype);
    assert.equal(builtGame.native, fakeBuiltNative);
    config.destroy();

    const uiCalls = [];
    const ui = Object.create(UiManager.prototype);
    ui.native = makeProxyNative(
      {
        createNode(componentType) {
          uiCalls.push(['createNode', [componentType]]);
          return componentType + 100;
        },
      },
      uiCalls,
    );

    ui.update();
    ui.render();
    assert.equal(ui.nodeCount(), 0);
    assert.equal(ui.createNode(0), 100);
    assert.equal(ui.removeNode(100n), 0);
    assert.equal(ui.setParent(100n, 101n), 0);
    assert.equal(ui.getParent(100n), 0);
    assert.equal(ui.getChildCount(100n), 0);
    assert.equal(ui.getChildAt(100n, 0), 0);
    assert.equal(ui.setWidget(100n, 2), 0);
    assert.equal(ui.setStyle(100n, { borderWidth: 2 }), 0);
    assert.equal(ui.setLabelText(100n, 'hello'), 0);
    assert.equal(ui.setButtonEnabled(100n, true), 0);
    assert.equal(ui.setImageTexturePath(100n, '/tmp/a.png'), 0);
    assert.equal(ui.setSlider(100n, 0, 1, 0.5, true), 0);
    assert.equal(ui.eventCount(), 0);
    assert.equal(ui.eventRead(0), 0);

    assert.equal(ui.createPanel(), 100);
    assert.equal(ui.createLabel('label'), 102);
    assert.equal(ui.createButton(), 101);
    assert.equal(ui.createImage('img.png'), 103);
    assert.equal(ui.createSlider(0, 10, 3), 104);

    const componentTypes = uiCalls
      .filter(([name]) => name === 'createNode')
      .map(([, args]) => args[0]);

    assert.ok(componentTypes.includes(0));
    assert.ok(componentTypes.includes(1));
    assert.ok(componentTypes.includes(2));
    assert.ok(componentTypes.includes(3));
    assert.ok(componentTypes.includes(4));
  });

  it('covers constructor paths for EngineConfig and UiManager', () => {
    const cfg = new EngineConfig();
    cfg.destroy();

    const ui = new UiManager();
    ui.update();
  });

  it('covers DiagnosticMode native and fallback paths', () => {
    DiagnosticMode.setEnabled(true);
    assert.equal(DiagnosticMode.isEnabled, true);
    assert.equal(DiagnosticMode.lastBacktrace, '');

    DiagnosticMode.setEnabled(false);
    assert.equal(DiagnosticMode.isEnabled, false);
    assert.equal(DiagnosticMode.lastBacktrace, '');
  });

  it('covers DiagnosticMode native branch via require interception', () => {
    const originalRequire = Module.prototype.require;
    const fakeNative = {
      goud_diagnostic_set_enabled() {},
      goud_diagnostic_is_enabled() {
        return true;
      },
      goud_diagnostic_last_backtrace() {
        return 'native-backtrace';
      },
    };

    Module.prototype.require = function patchedRequire(request, ...args) {
      if (request === '../node/index.g.js') {
        return fakeNative;
      }
      return originalRequire.call(this, request, ...args);
    };

    try {
      DiagnosticMode.setEnabled(true);
      assert.equal(DiagnosticMode.isEnabled, true);
      assert.equal(DiagnosticMode.lastBacktrace, 'native-backtrace');
    } finally {
      Module.prototype.require = originalRequire;
    }
  });

  it('covers Color.fromHex in generated math types', () => {
    const c = Color.fromHex(0xff00aa);
    assert.ok(c.r > 0.99);
    assert.ok(c.g < 0.01);
    assert.ok(c.b > 0.66);
  });

  it('covers shared NetworkEndpoint overlay and simulation helpers', () => {
    const calls = [];
    const fakeContext = {
      networkHost() {
        return 17;
      },
      networkConnect() {
        return 21;
      },
      networkConnectWithPeer() {
        return { handle: 21, peerId: 88 };
      },
      networkDisconnect(handle) {
        calls.push(['networkDisconnect', handle]);
        return 0;
      },
      networkSend(handle, peerId, data, channel) {
        calls.push(['networkSend', handle, peerId, data, channel]);
        return 0;
      },
      networkReceive() {
        return null;
      },
      networkReceivePacket() {
        return null;
      },
      networkPoll(handle) {
        calls.push(['networkPoll', handle]);
        return 0;
      },
      getNetworkStats(handle) {
        calls.push(['getNetworkStats', handle]);
        return { bytesSent: 1, bytesReceived: 2, packetsSent: 3, packetsReceived: 4, peersConnected: 1 };
      },
      networkPeerCount() {
        return 1;
      },
      setNetworkSimulation(handle, config) {
        calls.push(['setNetworkSimulation', handle, config]);
        return 0;
      },
      clearNetworkSimulation(handle) {
        calls.push(['clearNetworkSimulation', handle]);
        return 0;
      },
      setNetworkOverlayHandle(handle) {
        calls.push(['setNetworkOverlayHandle', handle]);
        return 0;
      },
      clearNetworkOverlayHandle() {
        calls.push(['clearNetworkOverlayHandle']);
        return 0;
      },
    };

    const endpoint = new NetworkManager(fakeContext).host(1, 9999);
    assert.equal(endpoint.setSimulation({ oneWayLatencyMs: 10, jitterMs: 2, packetLossPercent: 1 }), 0);
    assert.equal(endpoint.clearSimulation(), 0);
    assert.equal(endpoint.setOverlayTarget(), 0);
    assert.equal(endpoint.clearOverlayTarget(), 0);
    assert.equal(endpoint.getStats().bytesReceived, 2);

    assert.ok(calls.some(([name]) => name === 'setNetworkSimulation'));
    assert.ok(calls.some(([name]) => name === 'clearNetworkSimulation'));
    assert.ok(calls.some(([name]) => name === 'setNetworkOverlayHandle'));
    assert.ok(calls.some(([name]) => name === 'clearNetworkOverlayHandle'));
  });

  it('covers shared NetworkEndpoint send() error path without default peer', () => {
    const manager = new NetworkManager({
      networkHost() {
        return 7;
      },
      networkConnect() {
        return 9;
      },
      networkDisconnect() {
        return 0;
      },
      networkSend() {
        return 0;
      },
      networkReceive() {
        return new Uint8Array();
      },
      networkReceivePacket() {
        return null;
      },
      networkPoll() {
        return 0;
      },
      getNetworkStats() {
        return { bytesSent: 0, bytesReceived: 0, packetsSent: 0, packetsReceived: 0, peersConnected: 0 };
      },
      networkPeerCount() {
        return 0;
      },
      setNetworkSimulation() {
        return 0;
      },
      clearNetworkSimulation() {
        return 0;
      },
      setNetworkOverlayHandle() {
        return 0;
      },
      clearNetworkOverlayHandle() {
        return 0;
      },
      networkConnectWithPeer() {
        return { handle: 9, peerId: 1 };
      },
    });

    const endpoint = manager.host(1, 12345);
    assert.throws(() => endpoint.send(new Uint8Array([1, 2, 3])), /no default peer ID/);
  });

  it('covers real PhysicsWorld constructor branches in generated node wrapper', () => {
    const world2d = new PhysicsWorld2D(0, -9.81);
    const world3d = new PhysicsWorld3D(0, -9.81, 0);

    assert.equal(typeof world2d.destroy(), 'number');
    assert.equal(typeof world3d.destroy(), 'number');
  });

  it('validates Phase 0 feature methods exist on GoudGame prototype', () => {
    // FixedTimestep
    assert.equal(typeof GoudGame.prototype.setFixedTimestep, 'function', 'Missing setFixedTimestep');
    assert.equal(typeof GoudGame.prototype.setMaxFixedSteps, 'function', 'Missing setMaxFixedSteps');
    assert.equal(typeof GoudGame.prototype.runWithFixedUpdate, 'function', 'Missing runWithFixedUpdate');

    // SpriteBatch / TextBatch
    assert.equal(typeof GoudGame.prototype.drawSpriteBatch, 'function', 'Missing drawSpriteBatch');
    assert.equal(typeof GoudGame.prototype.drawTextBatch, 'function', 'Missing drawTextBatch');

    // RenderMetrics
    assert.equal(typeof GoudGame.prototype.getRenderMetrics, 'function', 'Missing getRenderMetrics');

    // GenericComponent
    assert.equal(typeof GoudGame.prototype.componentCount, 'function', 'Missing componentCount');
    assert.equal(typeof GoudGame.prototype.componentGetEntities, 'function', 'Missing componentGetEntities');
    assert.equal(typeof GoudGame.prototype.componentGetAll, 'function', 'Missing componentGetAll');
  });

  it('validates Phase 0 type interfaces in generated engine types', () => {
    const { readFileSync } = require('node:fs');
    const typesSrc = readFileSync(
      path.join(repoRoot, 'src', 'generated', 'types', 'engine.g.ts'),
      'utf8',
    );

    // IRenderMetrics interface with all 13 fields
    assert.ok(typesSrc.includes('export interface IRenderMetrics {'), 'Missing IRenderMetrics interface');
    for (const field of [
      'drawCallCount', 'spritesSubmitted', 'spritesDrawn', 'spritesCulled',
      'batchesSubmitted', 'avgSpritesPerBatch', 'spriteRenderMs', 'textRenderMs',
      'uiRenderMs', 'totalRenderMs', 'textDrawCalls', 'textGlyphCount', 'uiDrawCalls',
    ]) {
      assert.ok(typesSrc.includes(field), `IRenderMetrics missing field: ${field}`);
    }

    // ISpriteCmd and ITextCmd interfaces
    assert.ok(typesSrc.includes('export interface ISpriteCmd {'), 'Missing ISpriteCmd interface');
    assert.ok(typesSrc.includes('export interface ITextCmd {'), 'Missing ITextCmd interface');

    // drawSpriteBatch / drawTextBatch in game interface
    assert.ok(typesSrc.includes('drawSpriteBatch(cmds: ISpriteCmd[]): number;'), 'Missing drawSpriteBatch in IGame');
    assert.ok(typesSrc.includes('drawTextBatch(cmds: ITextCmd[]): number;'), 'Missing drawTextBatch in IGame');

    // getRenderMetrics in game interface
    assert.ok(typesSrc.includes('getRenderMetrics(): IRenderMetrics;'), 'Missing getRenderMetrics in IGame');

    // interpolationAlpha property
    assert.ok(typesSrc.includes('interpolationAlpha'), 'Missing interpolationAlpha in IGame');

    // setFixedTimestep method
    assert.ok(typesSrc.includes('setFixedTimestep(stepSize: number): void;'), 'Missing setFixedTimestep in IGame');
  });

  it('validates Phase 0 getRenderMetrics returns structured data with fake native', () => {
    const native = makeProxyNative({
      getRenderMetrics: () => ({
        drawCallCount: 5,
        spritesSubmitted: 100,
        spritesDrawn: 90,
        spritesCulled: 10,
        batchesSubmitted: 3,
        avgSpritesPerBatch: 30,
        spriteRenderMs: 1.5,
        textRenderMs: 0.5,
        uiRenderMs: 0.3,
        totalRenderMs: 2.3,
        textDrawCalls: 2,
        textGlyphCount: 42,
        uiDrawCalls: 1,
      }),
    });
    const game = makeGameForTests(native);
    const metrics = game.getRenderMetrics();
    assert.equal(metrics.drawCallCount, 5);
    assert.equal(metrics.spritesSubmitted, 100);
    assert.equal(metrics.spritesDrawn, 90);
    assert.equal(metrics.spritesCulled, 10);
    assert.equal(metrics.batchesSubmitted, 3);
    assert.equal(metrics.textDrawCalls, 2);
    assert.equal(metrics.textGlyphCount, 42);
    assert.equal(metrics.uiDrawCalls, 1);
  });
});
