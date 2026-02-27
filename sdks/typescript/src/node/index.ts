/**
 * Node.js backend for GoudEngine SDK.
 *
 * Re-exports from the napi-rs generated native addon.
 * The addon is built by `napi build` and loads the platform-specific
 * .node binary at runtime. napi-rs generates index.js and index.d.ts
 * at the package root with full TypeScript declarations.
 *
 * For Phase 4, the web backend will implement the same interfaces
 * defined in ../types/ using wasm-bindgen instead.
 */

export {
  GoudGame,
  Entity,
  type GameConfig,
  type Transform2DData,
  type SpriteData,
  type Vec2,
  type Vec3,
  type Color,
  type Rect,
  colorWhite,
  colorBlack,
  colorRed,
  colorGreen,
  colorBlue,
  colorYellow,
  colorTransparent,
  colorRgba,
  colorRgb,
  colorFromHex,
  transform2DDefault,
  transform2DFromPosition,
  transform2DFromScale,
  transform2DFromRotation,
  spriteDefault,
  vec2,
  vec2Zero,
  vec2One,
} from '../../index';
