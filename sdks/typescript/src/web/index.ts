/**
 * Web backend stub for GoudEngine SDK (Phase 4).
 *
 * This module will be implemented in Phase 4 when the wasm-bindgen
 * backend is added. For now, it exports nothing and throws if imported
 * to prevent accidental use.
 */

export function createWebGame(): never {
  throw new Error(
    'GoudEngine web backend is not yet available. ' +
    'Use the Node.js backend for desktop development. ' +
    'Web support is planned for Phase 4.'
  );
}
