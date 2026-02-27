/**
 * Component type interfaces for the GoudEngine SDK.
 *
 * These types mirror the Rust ECS component types exposed through the
 * napi-rs native addon. They serve as the shared contract between
 * the Node and (future) Web backends.
 */

export { ITransform2DData, ISpriteData, IVec2, IVec3, IColor, IRect } from './engine';
