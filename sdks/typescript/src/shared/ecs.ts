/**
 * FNV-1a hash for component type names.
 *
 * This is the only intentional local-logic exception in the TypeScript SDK.
 * It mirrors the Rust-side FNV-1a hash used to identify component types
 * so that SDK users can query components by name string.
 *
 * @param typeName - The component type name (e.g. "Transform2D", "Sprite")
 * @returns A 64-bit hash as a BigInt, suitable for passing to componentCount / componentGetEntities / componentGetAll
 */
export function componentTypeHash(typeName: string): bigint {
  const FNV_OFFSET_BASIS = 0xcbf29ce484222325n;
  const FNV_PRIME = 0x100000001b3n;
  const MASK_64 = (1n << 64n) - 1n;

  let hash = FNV_OFFSET_BASIS;
  for (let i = 0; i < typeName.length; i++) {
    hash ^= BigInt(typeName.charCodeAt(i));
    hash = (hash * FNV_PRIME) & MASK_64;
  }
  return hash;
}
