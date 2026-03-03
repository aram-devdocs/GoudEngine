use super::TestResource;
use crate::core::handle::{Handle, HandleAllocator};

#[test]
fn test_allocator_new() {
    // Test that new() creates an empty allocator
    let allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    assert_eq!(allocator.len(), 0, "New allocator should have len 0");
    assert_eq!(
        allocator.capacity(),
        0,
        "New allocator should have capacity 0"
    );
    assert!(allocator.is_empty(), "New allocator should be empty");
}

#[test]
fn test_allocator_default() {
    // Test that Default creates an empty allocator (same as new)
    let allocator: HandleAllocator<TestResource> = HandleAllocator::default();

    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
}

#[test]
fn test_allocator_allocate_single() {
    // Test allocating a single handle
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let handle = allocator.allocate();

    assert!(handle.is_valid(), "Allocated handle should be valid");
    assert_eq!(handle.index(), 0, "First allocation should have index 0");
    assert_eq!(
        handle.generation(),
        1,
        "First allocation should have generation 1"
    );
    assert_eq!(allocator.len(), 1, "Allocator should have 1 handle");
    assert_eq!(allocator.capacity(), 1, "Capacity should be 1");
    assert!(!allocator.is_empty(), "Allocator should not be empty");
}

#[test]
fn test_allocator_allocate_multiple() {
    // Test allocating multiple handles
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let h1 = allocator.allocate();
    let h2 = allocator.allocate();
    let h3 = allocator.allocate();

    // All should be valid and unique
    assert!(h1.is_valid());
    assert!(h2.is_valid());
    assert!(h3.is_valid());

    assert_ne!(h1, h2, "Handles should be unique");
    assert_ne!(h2, h3, "Handles should be unique");
    assert_ne!(h1, h3, "Handles should be unique");

    // Indices should be sequential
    assert_eq!(h1.index(), 0);
    assert_eq!(h2.index(), 1);
    assert_eq!(h3.index(), 2);

    // All should have generation 1
    assert_eq!(h1.generation(), 1);
    assert_eq!(h2.generation(), 1);
    assert_eq!(h3.generation(), 1);

    assert_eq!(allocator.len(), 3);
    assert_eq!(allocator.capacity(), 3);
}

#[test]
fn test_allocator_is_alive() {
    // Test is_alive for various scenarios
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // INVALID handle should never be alive
    assert!(
        !allocator.is_alive(Handle::INVALID),
        "INVALID should not be alive"
    );

    // Allocated handle should be alive
    let handle = allocator.allocate();
    assert!(
        allocator.is_alive(handle),
        "Allocated handle should be alive"
    );

    // Fabricated handle with wrong index should not be alive
    let fake = Handle::<TestResource>::new(100, 1);
    assert!(
        !allocator.is_alive(fake),
        "Handle with out-of-bounds index should not be alive"
    );

    // Fabricated handle with wrong generation should not be alive
    let wrong_gen = Handle::<TestResource>::new(0, 99);
    assert!(
        !allocator.is_alive(wrong_gen),
        "Handle with wrong generation should not be alive"
    );
}

#[test]
fn test_allocator_deallocate() {
    // Test basic deallocation
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let handle = allocator.allocate();
    assert!(allocator.is_alive(handle));

    // Deallocate should succeed and return true
    assert!(allocator.deallocate(handle), "Deallocation should succeed");

    // Handle should no longer be alive
    assert!(
        !allocator.is_alive(handle),
        "Deallocated handle should not be alive"
    );

    // Allocator should be empty
    assert_eq!(allocator.len(), 0, "Allocator should have 0 handles");
    assert_eq!(allocator.capacity(), 1, "Capacity should still be 1");
}

#[test]
fn test_allocator_deallocate_invalid() {
    // Test deallocating invalid handles
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // Deallocating INVALID should fail
    assert!(
        !allocator.deallocate(Handle::INVALID),
        "Deallocating INVALID should fail"
    );

    // Deallocating out-of-bounds handle should fail
    let fake = Handle::<TestResource>::new(100, 1);
    assert!(
        !allocator.deallocate(fake),
        "Deallocating out-of-bounds handle should fail"
    );

    // Allocate then try to deallocate with wrong generation
    let handle = allocator.allocate();
    let wrong_gen = Handle::<TestResource>::new(handle.index(), handle.generation() + 1);
    assert!(
        !allocator.deallocate(wrong_gen),
        "Deallocating with wrong generation should fail"
    );

    // Original handle should still be alive
    assert!(allocator.is_alive(handle));
}

#[test]
fn test_allocator_double_deallocate() {
    // Test that double deallocation fails gracefully
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let handle = allocator.allocate();

    // First deallocation succeeds
    assert!(allocator.deallocate(handle));

    // Second deallocation fails (generation mismatch)
    assert!(
        !allocator.deallocate(handle),
        "Double deallocation should fail"
    );
}

#[test]
fn test_allocator_slot_reuse() {
    // Test that deallocated slots are reused
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // Allocate and deallocate
    let h1 = allocator.allocate();
    assert_eq!(h1.index(), 0);
    assert_eq!(h1.generation(), 1);

    allocator.deallocate(h1);

    // Allocate again - should reuse slot 0 with incremented generation
    let h2 = allocator.allocate();
    assert_eq!(h2.index(), 0, "Should reuse slot 0");
    assert_eq!(h2.generation(), 2, "Generation should be incremented");

    // Capacity should still be 1 (slot was reused)
    assert_eq!(allocator.capacity(), 1);

    // h1 should be dead, h2 should be alive
    assert!(!allocator.is_alive(h1), "Old handle should be dead");
    assert!(allocator.is_alive(h2), "New handle should be alive");
}

#[test]
fn test_allocator_generation_prevents_aba() {
    // Test that generational indices prevent ABA problem
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // Allocate slot 0
    let h1 = allocator.allocate();
    assert!(allocator.is_alive(h1));

    // Deallocate slot 0
    allocator.deallocate(h1);

    // Allocate slot 0 again (different generation)
    let h2 = allocator.allocate();
    assert_eq!(h1.index(), h2.index(), "Same slot should be reused");
    assert_ne!(
        h1.generation(),
        h2.generation(),
        "Generations should differ"
    );

    // h1 is stale (references old generation)
    assert!(!allocator.is_alive(h1), "Old handle should be stale");
    assert!(allocator.is_alive(h2), "New handle should be alive");

    // Attempting to use h1 for operations should fail
    assert!(
        !allocator.deallocate(h1),
        "Deallocation with stale handle should fail"
    );
}

#[test]
fn test_allocator_generation_wrapping() {
    // Test that generation increments correctly on reuse
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // Allocate slot 0 with generation 1
    let h1 = allocator.allocate();
    assert_eq!(h1.index(), 0);
    assert_eq!(h1.generation(), 1);

    // Deallocate and re-allocate multiple times to increment generation
    allocator.deallocate(h1);
    let h2 = allocator.allocate();
    assert_eq!(h2.index(), 0, "Should reuse slot 0");
    assert_eq!(h2.generation(), 2, "Generation should be 2");

    allocator.deallocate(h2);
    let h3 = allocator.allocate();
    assert_eq!(h3.index(), 0, "Should reuse slot 0");
    assert_eq!(h3.generation(), 3, "Generation should be 3");

    allocator.deallocate(h3);
    let h4 = allocator.allocate();
    assert_eq!(h4.index(), 0, "Should reuse slot 0");
    assert_eq!(h4.generation(), 4, "Generation should be 4");

    // Proper test: deallocate then allocate sequentially
    allocator.deallocate(h4);
    for expected_gen in 5..=10 {
        let h = allocator.allocate();
        assert_eq!(h.index(), 0, "Should reuse slot 0");
        assert_eq!(h.generation(), expected_gen, "Generation should increment");
        allocator.deallocate(h);
    }
}

#[test]
fn test_allocator_len_and_capacity() {
    // Test len() and capacity() behavior
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // Empty state
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0);

    // Allocate some handles
    let h1 = allocator.allocate();
    let h2 = allocator.allocate();
    let h3 = allocator.allocate();

    assert_eq!(allocator.len(), 3);
    assert_eq!(allocator.capacity(), 3);

    // Deallocate one
    allocator.deallocate(h2);

    assert_eq!(allocator.len(), 2, "len should decrease on deallocation");
    assert_eq!(
        allocator.capacity(),
        3,
        "capacity should not change on deallocation"
    );

    // Allocate one more (should reuse h2's slot)
    let h4 = allocator.allocate();

    assert_eq!(allocator.len(), 3);
    assert_eq!(
        allocator.capacity(),
        3,
        "capacity should not increase when reusing"
    );

    // Allocate another (new slot)
    let _h5 = allocator.allocate();

    assert_eq!(allocator.len(), 4);
    assert_eq!(allocator.capacity(), 4);

    // Deallocate all
    allocator.deallocate(h1);
    allocator.deallocate(h4);
    allocator.deallocate(h3);
    let h5_deallocate = allocator.allocate(); // h5 was moved, allocate fresh for test
    allocator.deallocate(h5_deallocate);

    // Actually, let's restart to make this clearer
    let mut allocator2: HandleAllocator<TestResource> = HandleAllocator::new();

    let handles: Vec<_> = (0..5).map(|_| allocator2.allocate()).collect();
    assert_eq!(allocator2.len(), 5);
    assert_eq!(allocator2.capacity(), 5);

    for h in &handles {
        allocator2.deallocate(*h);
    }

    assert_eq!(allocator2.len(), 0, "All handles deallocated");
    assert_eq!(
        allocator2.capacity(),
        5,
        "Capacity unchanged after deallocation"
    );
    assert!(allocator2.is_empty());
}

#[test]
fn test_allocator_debug_format() {
    // Test Debug formatting
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();
    allocator.allocate();
    allocator.allocate();
    let h = allocator.allocate();
    allocator.deallocate(h);

    let debug_str = format!("{:?}", allocator);

    assert!(
        debug_str.contains("HandleAllocator"),
        "Debug should contain type name"
    );
    assert!(debug_str.contains("len"), "Debug should show len");
    assert!(debug_str.contains("capacity"), "Debug should show capacity");
}
