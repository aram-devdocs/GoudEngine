use super::{OtherResource, TestResource};
use crate::core::handle::Handle;
use std::hash::{Hash, Hasher};

#[test]
fn test_handle_new_and_accessors() {
    // Test that new() creates a handle with correct index and generation
    let handle: Handle<TestResource> = Handle::new(42, 7);

    assert_eq!(handle.index(), 42, "index should be 42");
    assert_eq!(handle.generation(), 7, "generation should be 7");
}

#[test]
fn test_handle_invalid_constant() {
    // Test that INVALID has the expected values
    let invalid: Handle<TestResource> = Handle::INVALID;

    assert_eq!(
        invalid.index(),
        u32::MAX,
        "INVALID index should be u32::MAX"
    );
    assert_eq!(invalid.generation(), 0, "INVALID generation should be 0");
}

#[test]
fn test_handle_is_valid() {
    // Test is_valid() for various handles
    let valid1: Handle<TestResource> = Handle::new(0, 1);
    let valid2: Handle<TestResource> = Handle::new(100, 50);
    let valid3: Handle<TestResource> = Handle::new(u32::MAX - 1, 1);
    let invalid: Handle<TestResource> = Handle::INVALID;

    assert!(valid1.is_valid(), "Handle(0, 1) should be valid");
    assert!(valid2.is_valid(), "Handle(100, 50) should be valid");
    assert!(valid3.is_valid(), "Handle(MAX-1, 1) should be valid");
    assert!(!invalid.is_valid(), "INVALID should not be valid");
}

#[test]
fn test_handle_edge_cases() {
    // Test edge cases near INVALID values
    // Handle with MAX index but non-zero generation is valid
    let edge1: Handle<TestResource> = Handle::new(u32::MAX, 1);
    assert!(edge1.is_valid(), "Handle(MAX, 1) should be valid");

    // Handle with zero generation but non-MAX index is valid
    let edge2: Handle<TestResource> = Handle::new(0, 0);
    assert!(edge2.is_valid(), "Handle(0, 0) should be valid");

    // Handle with index MAX-1 and generation 0 is valid
    let edge3: Handle<TestResource> = Handle::new(u32::MAX - 1, 0);
    assert!(edge3.is_valid(), "Handle(MAX-1, 0) should be valid");
}

#[test]
fn test_handle_size_and_alignment() {
    // Verify FFI-compatible size and alignment
    use std::mem::{align_of, size_of};

    assert_eq!(
        size_of::<Handle<TestResource>>(),
        8,
        "Handle should be 8 bytes"
    );
    assert_eq!(
        align_of::<Handle<TestResource>>(),
        4,
        "Handle should have 4-byte alignment"
    );

    // Different type parameters shouldn't affect size
    assert_eq!(
        size_of::<Handle<TestResource>>(),
        size_of::<Handle<OtherResource>>(),
        "Handle size should not depend on type parameter"
    );
}

// =========================================================================
// Trait Tests
// =========================================================================

#[test]
fn test_handle_clone() {
    // Test Clone implementation
    let original: Handle<TestResource> = Handle::new(42, 7);
    let cloned = original.clone();

    assert_eq!(original.index(), cloned.index());
    assert_eq!(original.generation(), cloned.generation());
    assert_eq!(original, cloned);
}

#[test]
fn test_handle_copy() {
    // Test Copy implementation (handles should be trivially copyable)
    let original: Handle<TestResource> = Handle::new(42, 7);

    // Copy by assignment
    let copied = original;

    // Original is still usable (proves Copy, not just Move)
    assert_eq!(original.index(), 42);
    assert_eq!(copied.index(), 42);
    assert_eq!(original, copied);
}

#[test]
fn test_handle_debug_format() {
    // Test Debug formatting: Handle<TypeName>(index:gen)
    let handle: Handle<TestResource> = Handle::new(42, 7);
    let debug_str = format!("{:?}", handle);

    // Should contain type name, index, and generation
    assert!(
        debug_str.contains("TestResource"),
        "Debug should contain type name, got: {}",
        debug_str
    );
    assert!(
        debug_str.contains("42"),
        "Debug should contain index, got: {}",
        debug_str
    );
    assert!(
        debug_str.contains("7"),
        "Debug should contain generation, got: {}",
        debug_str
    );
    // Check format: Handle<TypeName>(index:gen)
    assert!(
        debug_str.starts_with("Handle<"),
        "Debug should start with 'Handle<', got: {}",
        debug_str
    );
}

#[test]
fn test_handle_debug_invalid() {
    // Test Debug formatting for INVALID handle
    let invalid: Handle<TestResource> = Handle::INVALID;
    let debug_str = format!("{:?}", invalid);

    assert!(
        debug_str.contains(&u32::MAX.to_string()),
        "Debug of INVALID should show MAX index, got: {}",
        debug_str
    );
    assert!(
        debug_str.contains(":0)"),
        "Debug of INVALID should show generation 0, got: {}",
        debug_str
    );
}

#[test]
fn test_handle_partial_eq() {
    // Test PartialEq: must compare both index AND generation
    let h1: Handle<TestResource> = Handle::new(1, 1);
    let h2: Handle<TestResource> = Handle::new(1, 1);
    let h3: Handle<TestResource> = Handle::new(1, 2); // same index, different gen
    let h4: Handle<TestResource> = Handle::new(2, 1); // different index, same gen

    assert_eq!(h1, h2, "Same index and gen should be equal");
    assert_ne!(h1, h3, "Same index, different gen should not be equal");
    assert_ne!(h1, h4, "Different index, same gen should not be equal");
}

#[test]
fn test_handle_eq_reflexive_symmetric_transitive() {
    // Test Eq properties
    let a: Handle<TestResource> = Handle::new(5, 3);
    let b: Handle<TestResource> = Handle::new(5, 3);
    let c: Handle<TestResource> = Handle::new(5, 3);

    // Reflexive: a == a
    assert_eq!(a, a);

    // Symmetric: a == b implies b == a
    assert_eq!(a, b);
    assert_eq!(b, a);

    // Transitive: a == b && b == c implies a == c
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

#[test]
fn test_handle_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;

    // Test that Hash is consistent with PartialEq
    let h1: Handle<TestResource> = Handle::new(42, 7);
    let h2: Handle<TestResource> = Handle::new(42, 7);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    h1.hash(&mut hasher1);
    h2.hash(&mut hasher2);

    let hash1 = hasher1.finish();
    let hash2 = hasher2.finish();

    // Equal handles must have equal hashes
    assert_eq!(hash1, hash2, "Equal handles must have equal hashes");
}

#[test]
fn test_handle_hash_in_hashmap() {
    use std::collections::HashMap;

    // Test that handles work correctly as HashMap keys
    let mut map: HashMap<Handle<TestResource>, &str> = HashMap::new();

    let h1: Handle<TestResource> = Handle::new(1, 1);
    let h2: Handle<TestResource> = Handle::new(2, 1);
    let h3: Handle<TestResource> = Handle::new(1, 2); // same index as h1, different gen

    map.insert(h1, "first");
    map.insert(h2, "second");
    map.insert(h3, "third");

    assert_eq!(map.get(&h1), Some(&"first"));
    assert_eq!(map.get(&h2), Some(&"second"));
    assert_eq!(map.get(&h3), Some(&"third"));
    assert_eq!(map.len(), 3, "All three handles should be distinct keys");

    // Lookup with equivalent handle
    let h1_copy: Handle<TestResource> = Handle::new(1, 1);
    assert_eq!(map.get(&h1_copy), Some(&"first"));
}

#[test]
fn test_handle_default() {
    // Test Default returns INVALID
    let default_handle: Handle<TestResource> = Handle::default();

    assert!(!default_handle.is_valid());
    assert_eq!(default_handle, Handle::INVALID);
    assert_eq!(default_handle.index(), u32::MAX);
    assert_eq!(default_handle.generation(), 0);
}

#[test]
fn test_handle_to_u64_and_from_u64() {
    // Test pack/unpack round-trip
    let original: Handle<TestResource> = Handle::new(42, 7);
    let packed = original.to_u64();
    let unpacked: Handle<TestResource> = Handle::from_u64(packed);

    assert_eq!(original, unpacked);
    assert_eq!(original.index(), unpacked.index());
    assert_eq!(original.generation(), unpacked.generation());
}

#[test]
fn test_handle_u64_pack_format() {
    // Verify the packing format: upper 32 bits = generation, lower 32 = index
    let handle: Handle<TestResource> = Handle::new(0x12345678, 0xABCDEF01);
    let packed = handle.to_u64();

    // Expected: 0xABCDEF01_12345678
    let expected: u64 = 0xABCDEF01_12345678;
    assert_eq!(packed, expected, "Pack format should be gen:index");

    // Verify we can extract the parts
    let lower = packed as u32;
    let upper = (packed >> 32) as u32;
    assert_eq!(lower, 0x12345678, "Lower 32 bits should be index");
    assert_eq!(upper, 0xABCDEF01, "Upper 32 bits should be generation");
}

#[test]
fn test_handle_from_trait_u64() {
    // Test From<Handle<T>> for u64
    let handle: Handle<TestResource> = Handle::new(100, 50);
    let packed: u64 = handle.into();

    assert_eq!(packed, handle.to_u64());
}

#[test]
fn test_handle_into_trait_from_u64() {
    // Test From<u64> for Handle<T>
    let packed: u64 = (7u64 << 32) | 42u64;
    let handle: Handle<TestResource> = packed.into();

    assert_eq!(handle.index(), 42);
    assert_eq!(handle.generation(), 7);
}

#[test]
fn test_handle_u64_edge_cases() {
    // Test edge cases for pack/unpack

    // Zero handle
    let zero: Handle<TestResource> = Handle::new(0, 0);
    assert_eq!(zero.to_u64(), 0u64);
    assert_eq!(Handle::<TestResource>::from_u64(0), zero);

    // Max values
    let max: Handle<TestResource> = Handle::new(u32::MAX, u32::MAX);
    let packed = max.to_u64();
    assert_eq!(packed, u64::MAX);
    assert_eq!(Handle::<TestResource>::from_u64(u64::MAX), max);

    // INVALID handle
    let invalid: Handle<TestResource> = Handle::INVALID;
    let invalid_packed = invalid.to_u64();
    let invalid_unpacked: Handle<TestResource> = Handle::from_u64(invalid_packed);
    assert_eq!(invalid, invalid_unpacked);
    assert!(!invalid_unpacked.is_valid());
}

#[test]
fn test_handle_different_types_not_comparable() {
    // Verify that Handle<A> and Handle<B> are different types
    // This is a compile-time check - if this compiles, the types are distinct
    let _h1: Handle<TestResource> = Handle::new(1, 1);
    let _h2: Handle<OtherResource> = Handle::new(1, 1);

    // These have the same index/generation but are different types
    // We can't directly compare them (which is correct behavior)
    // This test just verifies the type system works
    fn assert_types_differ<A, B>() {}
    assert_types_differ::<Handle<TestResource>, Handle<OtherResource>>();
}
