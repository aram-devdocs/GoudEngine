/**
 * @goudengine/sdk — TypeScript SDK for GoudEngine
 *
 * Unified entry point that re-exports the appropriate backend.
 * In Node.js, this uses the napi-rs native addon for zero-overhead
 * Rust bindings. In browsers (Phase 4), this will use wasm-bindgen.
 *
 * Usage:
 *   import { GoudGame, Entity, vec2 } from '@goudengine/sdk';
 *
 *   const game = new GoudGame({ title: 'My Game', width: 800, height: 600 });
 *   const player = game.spawnEmpty();
 *   game.addTransform2D(player, transform2DFromPosition(100, 200));
 */

export * from './node';
export type {
  IGameConfig,
  IEntity,
  IGoudGame,
  IGameContext,
  ITransform2DData,
  ISpriteData,
  IVec2,
  IVec3,
  IColor,
  IRect,
} from './types';
export { Key, MouseButton } from './types';
