/**
 * Smoke test for the GoudEngine Node.js SDK.
 * Verifies the napi-rs native addon loads and basic operations work.
 *
 * Run with: node --test test/smoke.test.mjs
 */

import { describe, it } from 'node:test';
import assert from 'node:assert/strict';

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

describe('GoudGame', () => {
  it('creates with default config', () => {
    const game = new GoudGame();
    assert.equal(game.entityCount(), 0);
    assert.equal(game.title, 'GoudEngine');
    assert.equal(game.windowWidth, 800);
    assert.equal(game.windowHeight, 600);
  });

  it('creates with custom config', () => {
    const game = new GoudGame({
      title: 'Test Game',
      width: 1280,
      height: 720,
    });
    assert.equal(game.title, 'Test Game');
    assert.equal(game.windowWidth, 1280);
    assert.equal(game.windowHeight, 720);
  });

  it('spawns and despawns entities', () => {
    const game = new GoudGame();
    const entity = game.spawnEmpty();
    assert.equal(game.entityCount(), 1);
    assert.ok(game.isAlive(entity));

    const despawned = game.despawn(entity);
    assert.ok(despawned);
    assert.equal(game.entityCount(), 0);
    assert.ok(!game.isAlive(entity));
  });

  it('spawns batch of entities', () => {
    const game = new GoudGame();
    const entities = game.spawnBatch(100);
    assert.equal(entities.length, 100);
    assert.equal(game.entityCount(), 100);
    for (const e of entities) {
      assert.ok(game.isAlive(e));
    }
  });

  it('manages Transform2D components', () => {
    const game = new GoudGame();
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
  });

  it('manages Name components', () => {
    const game = new GoudGame();
    const entity = game.spawnEmpty();

    game.addName(entity, 'Player');
    assert.ok(game.hasName(entity));
    assert.equal(game.getName(entity), 'Player');

    const removed = game.removeName(entity);
    assert.ok(removed);
    assert.ok(!game.hasName(entity));
    assert.equal(game.getName(entity), null);
  });

  it('manages Sprite components', () => {
    const game = new GoudGame();
    const entity = game.spawnEmpty();

    const sprite = spriteDefault();
    game.addSprite(entity, sprite);
    assert.ok(game.hasSprite(entity));

    const s = game.getSprite(entity);
    assert.ok(s !== null);
    assert.equal(s.flipX, false);
    assert.equal(s.flipY, false);

    game.removeSprite(entity);
    assert.ok(!game.hasSprite(entity));
  });

  it('updates frame and tracks timing', () => {
    const game = new GoudGame();
    game.updateFrame(1 / 60);

    assert.ok(game.deltaTime > 0);
    assert.ok(game.totalTime > 0);
    assert.ok(game.fps > 0);
    assert.equal(game.frameCount, 1);

    game.updateFrame(1 / 60);
    assert.equal(game.frameCount, 2);
  });

  it('exposes animation control methods and accepts entity/string params', () => {
    const game = new GoudGame();
    const entity = game.spawnEmpty();

    assert.equal(typeof game.play, 'function');
    assert.equal(typeof game.stop, 'function');
    assert.equal(typeof game.setState, 'function');
    assert.equal(typeof game.setParameterBool, 'function');
    assert.equal(typeof game.setParameterFloat, 'function');

    assert.equal(typeof game.play(entity), 'number');
    assert.equal(typeof game.stop(entity), 'number');
    assert.equal(typeof game.setState(entity, 'idle'), 'number');
    assert.equal(typeof game.setParameterBool(entity, 'moving', true), 'number');
    assert.equal(typeof game.setParameterFloat(entity, 'speed', 1.5), 'number');
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
