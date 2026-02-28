/**
 * @goudengine/sdk — TypeScript SDK for GoudEngine
 *
 * Unified entry point that re-exports the appropriate backend.
 * In Node.js, this uses the napi-rs native addon for zero-overhead
 * Rust bindings. In browsers, use the `@goudengine/sdk/web` export
 * which loads the wasm-bindgen module.
 *
 * Node.js usage:
 *   import { GoudGame, Entity, vec2 } from '@goudengine/sdk';
 *
 * Browser usage:
 *   import { createWebGame } from '@goudengine/sdk/web';
 *   const game = await createWebGame({ title: 'My Game', width: 800, height: 600 });
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
