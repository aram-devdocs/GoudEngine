/**
 * Smoke test for the GoudEngine Node.js SDK.
 * Verifies the napi-rs native addon loads and basic operations work.
 *
 * Run with: node --test test/smoke.test.mjs
 */

import { after, before, describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import generatedPkg from '../dist/generated/index.g.js';

import {
  GoudGame,
  Entity,
  colorWhite,
  colorRgba,
  colorFromHex,
  transform2DDefault,
  transform2DFromPosition,
  transform2DFromScale,
  transform2DFromRotation,
  spriteDefault,
  vec2,
  vec2Zero,
  vec2One,
} from '../index.js';
import { GoudGame as WrappedGoudGame } from '../dist/generated/node/index.g.js';
import { GoudGame as PackageGoudGame } from '../dist/index.js';

const repoRoot = path.resolve(process.cwd(), '..', '..');
const { EngineConfig, RenderBackendKind, WindowBackendKind } = generatedPkg;

describe('GoudGame', () => {
  let game;

  before(() => {
    game = new GoudGame();
  });

  after(() => {
    game.destroy();
  });

  it('creates with default config', () => {
    assert.equal(game.entityCount(), 0);
    assert.equal(game.title, 'GoudEngine');
    assert.equal(game.windowWidth, 800);
    assert.equal(game.windowHeight, 600);
  });

  it('uses the typed EngineConfig backend setters', () => {
    const config = new EngineConfig();
    assert.equal(config.setTitle('Test Game'), config);
    assert.equal(config.setSize(1280, 720), config);
    assert.equal(config.setRenderBackend(RenderBackendKind.Wgpu), config);
    assert.equal(config.setWindowBackend(WindowBackendKind.Winit), config);
    config.destroy();
  });

  it('spawns and despawns entities', () => {
    const entity = game.spawnEmpty();
    assert.equal(game.entityCount(), 1);
    assert.ok(game.isAlive(entity));

    const despawned = game.despawn(entity);
    assert.ok(despawned);
    assert.equal(game.entityCount(), 0);
    assert.ok(!game.isAlive(entity));
  });

  it('spawns batch of entities', () => {
    const entities = game.spawnBatch(100);
    assert.equal(entities.length, 100);
    assert.equal(game.entityCount(), 100);
    for (const entity of entities) {
      assert.ok(game.isAlive(entity));
      assert.ok(game.despawn(entity));
    }
    assert.equal(game.entityCount(), 0);
  });

  it('manages Transform2D components', () => {
    const entity = game.spawnEmpty();

    assert.ok(!game.hasTransform2D(entity));

    game.addTransform2D(entity, {
      positionX: 100,
      positionY: 200,
      rotation: 0,
      scaleX: 1,
      scaleY: 1,
    });
    assert.ok(game.hasTransform2D(entity));

    const t = game.getTransform2D(entity);
    assert.ok(t !== null);
    assert.equal(t.positionX, 100);
    assert.equal(t.positionY, 200);
    assert.equal(t.scaleX, 1);

    game.setTransform2D(entity, {
      positionX: 300,
      positionY: 400,
      rotation: 1.5,
      scaleX: 2,
      scaleY: 2,
    });
    const t2 = game.getTransform2D(entity);
    assert.equal(t2.positionX, 300);
    assert.equal(t2.positionY, 400);

    const removed = game.removeTransform2D(entity);
    assert.ok(removed);
    assert.ok(!game.hasTransform2D(entity));
    assert.ok(game.despawn(entity));
  });

  it('manages Name components', () => {
    const entity = game.spawnEmpty();

    game.addName(entity, 'Player');
    assert.ok(game.hasName(entity));
    assert.equal(game.getName(entity), 'Player');

    const removed = game.removeName(entity);
    assert.ok(removed);
    assert.ok(!game.hasName(entity));
    assert.equal(game.getName(entity), null);
    assert.ok(game.despawn(entity));
  });

  it('manages Sprite components', () => {
    const entity = game.spawnEmpty();

    const sprite = spriteDefault();
    game.addSprite(entity, sprite);
    assert.ok(game.hasSprite(entity));

    const s = game.getSprite(entity);
    assert.ok(s !== null);
    assert.equal(s.flipX, false);
    assert.equal(s.flipY, false);
    assert.equal(s.zLayer, 0);

    game.removeSprite(entity);
    assert.ok(!game.hasSprite(entity));
    assert.ok(game.despawn(entity));
  });

  it('updates frame and tracks timing', () => {
    const baselineFrames = game.frameCount;
    const baselineTime = game.totalTime;
    game.updateFrame(1 / 60);

    assert.ok(game.deltaTime > 0);
    assert.ok(game.totalTime > baselineTime);
    assert.ok(game.fps > 0);
    assert.equal(game.frameCount, baselineFrames + 1);

    game.updateFrame(1 / 60);
    assert.equal(game.frameCount, baselineFrames + 2);
  });

  it('exposes idiomatic scene wrapper API', () => {
    assert.equal(typeof game.loadScene, 'function');
    assert.equal(typeof game.unloadScene, 'function');
    assert.equal(typeof game.setActiveScene, 'function');

    const sceneName = 'sdk_scene_wrapper';
    const sceneJson = '{"name":"sdk_scene_wrapper_data","entities":[]}';
    const sceneId = game.loadScene(sceneName, sceneJson);
    assert.notEqual(sceneId, 0xFFFFFFFF);

    assert.equal(game.setActiveScene(sceneId, true), true);
    assert.equal(game.unloadScene(sceneName), true);
  });

  it('exposes native advanced audio methods for parity', () => {
    const requiredNativeMethods = [
      'audioPlay',
      'audioPlayOnChannel',
      'audioPlayWithSettings',
      'audioStop',
      'audioPause',
      'audioResume',
      'audioStopAll',
      'audioSetGlobalVolume',
      'audioGetGlobalVolume',
      'audioSetChannelVolume',
      'audioGetChannelVolume',
      'audioIsPlaying',
      'audioActiveCount',
      'audioPlaySpatial3d',
      'audioUpdateSpatial3d',
      'audioSetListenerPosition3d',
      'audioSetSourcePosition3d',
      'audioSetPlayerVolume',
      'audioSetPlayerSpeed',
      'audioCrossfade',
      'audioCrossfadeTo',
      'audioMixWith',
      'audioUpdateCrossfades',
      'audioActiveCrossfadeCount',
      'audioCleanupFinished',
      'audioActivate',
    ];

    for (const method of requiredNativeMethods) {
      assert.equal(typeof game[method], 'function', `Missing native method: ${method}`);
    }

    assert.equal(typeof game.loadAudioClip, 'undefined');
    assert.equal(typeof game.unloadAudioClip, 'undefined');
  });

  it('exposes preload on the built package wrapper', () => {
    assert.equal(typeof PackageGoudGame, 'function');
    assert.equal(typeof PackageGoudGame.prototype.preload, 'function');
    assert.equal(typeof PackageGoudGame.prototype.loadTexture, 'function');
  });
});

describe('Generated native audio bindings', () => {
  it('keeps generated wrapper audio methods aligned with the native runtime object', () => {
    const requiredWrapperMethods = [
      'audioPlay',
      'audioPlayOnChannel',
      'audioPlayWithSettings',
      'audioStop',
      'audioPause',
      'audioResume',
      'audioStopAll',
      'audioSetGlobalVolume',
      'audioGetGlobalVolume',
      'audioSetChannelVolume',
      'audioGetChannelVolume',
      'audioIsPlaying',
      'audioActiveCount',
      'audioPlaySpatial3d',
      'audioUpdateSpatial3d',
      'audioSetListenerPosition3d',
      'audioSetSourcePosition3d',
      'audioSetPlayerVolume',
      'audioSetPlayerSpeed',
      'audioCrossfade',
      'audioCrossfadeTo',
      'audioMixWith',
      'audioUpdateCrossfades',
      'audioActiveCrossfadeCount',
      'audioCleanupFinished',
      'audioActivate',
    ];

    for (const method of requiredWrapperMethods) {
      assert.equal(typeof GoudGame.prototype[method], 'function', `Missing package wrapper method: ${method}`);
      assert.equal(typeof WrappedGoudGame.prototype[method], 'function', `Missing generated wrapper method: ${method}`);
    }
  });
});

describe('Generated web UiManager bindings', () => {
  it('expands setStyle into the scalar wasm ABI while keeping the public object API', () => {
    const typesSrc = readFileSync(new URL('../src/generated/types/engine.g.ts', import.meta.url), 'utf8');
    const nodeSrc = readFileSync(new URL('../src/generated/node/index.g.ts', import.meta.url), 'utf8');
    const webSrc = readFileSync(new URL('../src/generated/web/index.g.ts', import.meta.url), 'utf8');

    assert.ok(
      typesSrc.includes('export type UiNodeId = number | bigint;'),
      'generated engine types should expose a shared UiNodeId alias',
    );

    for (const fragment of [
      'nodeId: UiNodeId;',
      'previousNodeId: UiNodeId;',
      'currentNodeId: UiNodeId;',
      'createNode(componentType: number): UiNodeId;',
      'setStyle(nodeId: UiNodeId, style: IUiStyle): number;',
      'createPanel(): UiNodeId;',
    ]) {
      assert.ok(
        typesSrc.includes(fragment),
        `generated engine types missing UiNodeId fragment: ${fragment}`,
      );
    }

    assert.ok(
      webSrc.includes(
        'set_style(node_id: bigint, background_r?: number, background_g?: number, background_b?: number, background_a?: number, foreground_r?: number, foreground_g?: number, foreground_b?: number, foreground_a?: number, border_r?: number, border_g?: number, border_b?: number, border_a?: number, border_width?: number, font_family?: string, font_size?: number, texture_path?: string, widget_spacing?: number): number;',
      ),
      'web wasm handle signature should expose scalar UiStyle fields with bigint node ids',
    );

    assert.ok(
      webSrc.includes('function toWasmUiNodeId(nodeId: UiNodeId): bigint {'),
      'web UiManager should normalize public node ids to the wasm bigint boundary',
    );

    assert.ok(
      webSrc.includes('setStyle(nodeId: UiNodeId, style: IUiStyle): number {'),
      'public web UiManager API should keep the object-shaped setStyle signature with UiNodeId typing',
    );

    assert.ok(
      nodeSrc.includes('interface NativeBindings {'),
      'generated node wrapper should declare a typed native bindings helper interface',
    );
    assert.ok(
      nodeSrc.includes("function getNativeBindings(): NativeBindings {"),
      'generated node wrapper should type the native bindings loader',
    );
    assert.ok(
      nodeSrc.includes("return eval('require')(\"../../../index\") as NativeBindings;"),
      'generated node wrapper should lazy-load the native addon entrypoint with a typed assertion',
    );

    for (const fragment of [
      'export interface IDebuggerConfig { enabled: boolean; publishLocalAttach: boolean; routeLabel: string; }',
      'export interface IContextConfig { debugger: IDebuggerConfig; }',
      'export interface IMemorySummary {',
      'getDebuggerSnapshotJson(): string;',
      'getDebuggerManifestJson(): string;',
      'setDebuggerProfilingEnabled(enabled: boolean): void;',
      'setDebuggerSelectedEntity(entityId: number): void;',
      'clearDebuggerSelectedEntity(): void;',
      'getMemorySummary(): IMemorySummary;',
      'setDebugger(debuggerConfig: IDebuggerConfig): IEngineConfig;',
    ]) {
      assert.ok(
        typesSrc.includes(fragment),
        `generated engine types missing debugger fragment: ${fragment}`,
      );
    }

    for (const fragment of [
      'constructor(config?: IContextConfig) {',
      'this.native = new (getNativeBindings().GoudContext)(nativeConfig as Record<string, unknown>);',
      'getDebuggerSnapshotJson(): string {',
      'getDebuggerManifestJson(): string {',
      'setDebuggerProfilingEnabled(enabled: boolean): void {',
      'setDebuggerSelectedEntity(entityId: number): void {',
      'clearDebuggerSelectedEntity(): void {',
      'getMemorySummary(): IMemorySummary {',
      'setDebugger(debuggerConfig: IDebuggerConfig): EngineConfig {',
    ]) {
      assert.ok(
        nodeSrc.includes(fragment),
        `generated node debugger wrapper missing fragment: ${fragment}`,
      );
    }

    for (const fragment of [
      'getDebuggerSnapshotJson(): string { return this.handle.getDebuggerSnapshotJson(); }',
      "setDebuggerPaused(paused: boolean): void { this.handle.dispatchDebuggerRequest(",
      'setDebugger(debuggerConfig: IDebuggerConfig): EngineConfig {',
    ]) {
      assert.ok(
        webSrc.includes(fragment),
        `generated web debugger surface missing fragment: ${fragment}`,
      );
    }

    for (const fragment of [
      'createNode(componentType: number): UiNodeId { return this.handle.create_node(componentType); }',
      'removeNode(nodeId: UiNodeId): number { return this.handle.remove_node(toWasmUiNodeId(nodeId)); }',
      'setParent(childId: UiNodeId, parentId: UiNodeId): number { return this.handle.set_parent(toWasmUiNodeId(childId), toWasmUiNodeId(parentId)); }',
      'toWasmUiNodeId(nodeId),',
      'style.backgroundColor?.r,',
      'style.backgroundColor?.a,',
      'style.foregroundColor?.r,',
      'style.borderColor?.a,',
      'style.borderWidth,',
      'style.fontFamily,',
      'style.fontSize,',
      'style.texturePath,',
      'style.widgetSpacing,',
    ]) {
      assert.ok(
        webSrc.includes(fragment),
        `web UiManager.setStyle bridge missing fragment: ${fragment}`,
      );
    }

    assert.equal(
      webSrc.includes('return this.handle.set_style(nodeId, style);'),
      false,
      'web UiManager.setStyle must not pass the whole style object directly to wasm',
    );

    assert.ok(
      nodeSrc.includes('function toNativeUiNodeId(nodeId: UiNodeId): number {'),
      'node UiManager should accept shared UiNodeId values and normalize them for the native addon',
    );

    for (const fragment of [
      'createNode(componentType: number): UiNodeId {',
      'removeNode(nodeId: UiNodeId): number {',
      'return this.native.removeNode(toNativeUiNodeId(nodeId));',
      'setParent(childId: UiNodeId, parentId: UiNodeId): number {',
      'return this.native.setParent(toNativeUiNodeId(childId), toNativeUiNodeId(parentId));',
    ]) {
      assert.ok(
        nodeSrc.includes(fragment),
        `node UiManager source missing UiNodeId fragment: ${fragment}`,
      );
    }
  });
});

describe('Entity', () => {
  it('creates from constructor', () => {
    const entity = new Entity(42, 7);
    assert.equal(entity.index, 42);
    assert.equal(entity.generation, 7);
    assert.ok(!entity.isPlaceholder);
  });

  it('creates placeholder', () => {
    const p = Entity.placeholder();
    assert.ok(p.isPlaceholder);
  });

  it('roundtrips through bits', () => {
    const entity = new Entity(42, 7);
    const bits = entity.toBits();
    const restored = Entity.fromBits(bits);
    assert.equal(restored.index, 42);
    assert.equal(restored.generation, 7);
  });

  it('displays correctly', () => {
    const entity = new Entity(42, 3);
    assert.equal(entity.display(), 'Entity(42:3)');
  });
});

describe('Factory functions', () => {
  it('creates Vec2 values', () => {
    const v = vec2(10, 20);
    assert.equal(v.x, 10);
    assert.equal(v.y, 20);

    const z = vec2Zero();
    assert.equal(z.x, 0);
    assert.equal(z.y, 0);

    const o = vec2One();
    assert.equal(o.x, 1);
    assert.equal(o.y, 1);
  });

  it('creates Color values', () => {
    const w = colorWhite();
    assert.equal(w.r, 1);
    assert.equal(w.g, 1);
    assert.equal(w.b, 1);
    assert.equal(w.a, 1);

    const custom = colorRgba(0.5, 0.6, 0.7, 0.8);
    assert.ok(Math.abs(custom.r - 0.5) < 0.01);
    assert.ok(Math.abs(custom.g - 0.6) < 0.01);

    const hex = colorFromHex(0xFF0000);
    assert.ok(Math.abs(hex.r - 1.0) < 0.01);
    assert.ok(Math.abs(hex.g) < 0.01);
    assert.ok(Math.abs(hex.b) < 0.01);
  });

  it('creates Transform2D data', () => {
    const def = transform2DDefault();
    assert.equal(def.positionX, 0);
    assert.equal(def.positionY, 0);
    assert.equal(def.scaleX, 1);
    assert.equal(def.scaleY, 1);

    const pos = transform2DFromPosition(100, 200);
    assert.equal(pos.positionX, 100);
    assert.equal(pos.positionY, 200);

    const scale = transform2DFromScale(2, 3);
    assert.equal(scale.scaleX, 2);
    assert.equal(scale.scaleY, 3);

    const rot = transform2DFromRotation(3.14);
    assert.ok(Math.abs(rot.rotation - 3.14) < 0.01);
  });
});
