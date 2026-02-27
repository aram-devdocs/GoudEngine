/**
 * Shared TypeScript interfaces for the GoudEngine SDK.
 *
 * These interfaces define the contract that both the Node.js (napi-rs)
 * and future Web (wasm-bindgen) backends must satisfy.
 */

export interface IGameConfig {
  title?: string;
  width?: number;
  height?: number;
  vsync?: boolean;
  fullscreen?: boolean;
  resizable?: boolean;
  targetFps?: number;
  debugRendering?: boolean;
}

export interface IEntity {
  readonly index: number;
  readonly generation: number;
  readonly isPlaceholder: boolean;
  toBits(): bigint;
  toString(): string;
}

export interface IGameContext {
  readonly deltaTime: number;
  readonly totalTime: number;
  readonly fps: number;
  readonly frameCount: number;
}

export interface IGoudGame {
  // Entity operations
  spawnEmpty(): IEntity;
  spawnBatch(count: number): IEntity[];
  despawn(entity: IEntity): boolean;
  entityCount(): number;
  isAlive(entity: IEntity): boolean;

  // Transform2D component
  addTransform2d(entity: IEntity, data: ITransform2DData): void;
  getTransform2d(entity: IEntity): ITransform2DData | null;
  setTransform2d(entity: IEntity, data: ITransform2DData): void;
  hasTransform2d(entity: IEntity): boolean;
  removeTransform2d(entity: IEntity): boolean;

  // Name component
  addName(entity: IEntity, name: string): void;
  getName(entity: IEntity): string | null;
  hasName(entity: IEntity): boolean;
  removeName(entity: IEntity): boolean;

  // Game loop
  updateFrame(deltaTime: number): void;

  // Timing (read after updateFrame)
  readonly deltaTime: number;
  readonly totalTime: number;
  readonly fps: number;
  readonly frameCount: number;

  // Config
  readonly title: string;
  readonly windowWidth: number;
  readonly windowHeight: number;
}

export interface ITransform2DData {
  positionX: number;
  positionY: number;
  rotation: number;
  scaleX: number;
  scaleY: number;
}

export interface ISpriteData {
  color: IColor;
  flipX: boolean;
  flipY: boolean;
  anchorX: number;
  anchorY: number;
  customWidth?: number;
  customHeight?: number;
  sourceRectX?: number;
  sourceRectY?: number;
  sourceRectWidth?: number;
  sourceRectHeight?: number;
}

export interface IVec2 {
  x: number;
  y: number;
}

export interface IVec3 {
  x: number;
  y: number;
  z: number;
}

export interface IColor {
  r: number;
  g: number;
  b: number;
  a: number;
}

export interface IRect {
  x: number;
  y: number;
  width: number;
  height: number;
}
