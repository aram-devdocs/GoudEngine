use super::TestResource;
use crate::core::handle::HandleAllocator;

#[test]
fn test_allocator_with_capacity() {
    // Test with_capacity creates allocator with reserved space
    let allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(100);

    assert_eq!(
        allocator.len(),
        0,
        "with_capacity should not allocate handles"
    );
    assert!(
        allocator.is_empty(),
        "with_capacity should leave allocator empty"
    );
    assert_eq!(
        allocator.capacity(),
        0,
        "capacity() reports active slots, not reserved"
    );

    // Verify we can allocate up to capacity without reallocation
    // (Can't directly test Vec capacity, but we can verify behavior)
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(100);
    for _ in 0..100 {
        allocator.allocate();
    }
    assert_eq!(allocator.len(), 100);
    assert_eq!(allocator.capacity(), 100);
}

#[test]
fn test_allocator_with_capacity_zero() {
    // Test with_capacity(0) is equivalent to new()
    let allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(0);

    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
}

#[test]
fn test_allocator_clear_basic() {
    // Test basic clear functionality
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let h1 = allocator.allocate();
    let h2 = allocator.allocate();
    let h3 = allocator.allocate();

    assert_eq!(allocator.len(), 3);
    assert!(allocator.is_alive(h1));
    assert!(allocator.is_alive(h2));
    assert!(allocator.is_alive(h3));

    allocator.clear();

    // All handles should be invalid
    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
    assert!(!allocator.is_alive(h1));
    assert!(!allocator.is_alive(h2));
    assert!(!allocator.is_alive(h3));

    // Capacity should be retained
    assert_eq!(allocator.capacity(), 3);
}

#[test]
fn test_allocator_clear_and_reallocate() {
    // Test that clear increments generations properly
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let h1 = allocator.allocate();
    assert_eq!(h1.generation(), 1);

    allocator.clear();

    // New allocation should have incremented generation
    let h2 = allocator.allocate();
    assert_eq!(h2.index(), h1.index(), "Should reuse same slot");
    assert_eq!(
        h2.generation(),
        2,
        "Generation should be incremented after clear"
    );

    // Old handle still not alive
    assert!(!allocator.is_alive(h1));
    assert!(allocator.is_alive(h2));
}

#[test]
fn test_allocator_clear_empty() {
    // Test clearing an empty allocator
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    allocator.clear();

    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
    assert_eq!(allocator.capacity(), 0);
}

#[test]
fn test_allocator_clear_with_some_deallocated() {
    // Test clear when some handles are already deallocated
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let h1 = allocator.allocate();
    let h2 = allocator.allocate();
    let h3 = allocator.allocate();

    // Deallocate middle one
    allocator.deallocate(h2);

    assert_eq!(allocator.len(), 2);

    allocator.clear();

    // All should be invalid
    assert_eq!(allocator.len(), 0);
    assert!(!allocator.is_alive(h1));
    assert!(!allocator.is_alive(h2)); // Already was invalid
    assert!(!allocator.is_alive(h3));

    // All slots should be in free list
    assert_eq!(allocator.capacity(), 3);
}

#[test]
fn test_allocator_shrink_to_fit() {
    // Test shrink_to_fit reduces free list memory
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    // Allocate many handles
    let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();

    // Deallocate most of them
    for h in handles.iter().skip(10) {
        allocator.deallocate(*h);
    }

    assert_eq!(allocator.len(), 10);

    // Shrink should work without panic
    allocator.shrink_to_fit();

    // Functionality should be preserved
    assert_eq!(allocator.len(), 10);
    assert_eq!(allocator.capacity(), 100);

    // Can still allocate (from free list)
    let new_handle = allocator.allocate();
    assert!(allocator.is_alive(new_handle));
}

#[test]
fn test_allocator_shrink_to_fit_empty() {
    // Test shrink_to_fit on empty allocator
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    allocator.shrink_to_fit(); // Should not panic

    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
}

#[test]
fn test_allocator_stress_100k_allocations() {
    // Stress test: 100K allocations to verify performance and correctness
    const COUNT: usize = 100_000;

    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::with_capacity(COUNT);

    // Phase 1: Allocate all
    let handles: Vec<_> = (0..COUNT).map(|_| allocator.allocate()).collect();

    assert_eq!(allocator.len(), COUNT);
    assert_eq!(allocator.capacity(), COUNT);

    // Verify all are alive and unique
    for (i, h) in handles.iter().enumerate() {
        assert!(allocator.is_alive(*h), "Handle {} should be alive", i);
        assert_eq!(h.index(), i as u32, "Handle {} should have index {}", i, i);
    }

    // Phase 2: Deallocate every other handle
    for (i, h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            assert!(allocator.deallocate(*h));
        }
    }

    assert_eq!(allocator.len(), COUNT / 2);

    // Phase 3: Verify deallocated handles are not alive
    for (i, h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            assert!(
                !allocator.is_alive(*h),
                "Deallocated handle {} should not be alive",
                i
            );
        } else {
            assert!(allocator.is_alive(*h), "Handle {} should still be alive", i);
        }
    }

    // Phase 4: Reallocate - should reuse free slots
    let new_handles: Vec<_> = (0..COUNT / 2).map(|_| allocator.allocate()).collect();

    assert_eq!(allocator.len(), COUNT);
    assert_eq!(
        allocator.capacity(),
        COUNT,
        "Capacity should not grow when reusing slots"
    );

    // Verify new handles are alive
    for (i, h) in new_handles.iter().enumerate() {
        assert!(allocator.is_alive(*h), "New handle {} should be alive", i);
    }

    // Phase 5: Clear and verify
    allocator.clear();

    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
    assert_eq!(allocator.capacity(), COUNT);

    // All handles should be invalid
    for h in handles.iter().take(10) {
        assert!(!allocator.is_alive(*h));
    }
    for h in new_handles.iter().take(10) {
        assert!(!allocator.is_alive(*h));
    }

    // Can still allocate after clear
    let after_clear = allocator.allocate();
    assert!(allocator.is_alive(after_clear));
    // Generation will be at least 2 (was 1 for fresh slots, or 2 for reallocated slots)
    // After clear, all generations are incremented by 1
    assert!(
        after_clear.generation() >= 2,
        "Generation should be at least 2 after clear, got {}",
        after_clear.generation()
    );
}

#[test]
fn test_allocator_clear_multiple_times() {
    // Test clearing multiple times increments generations correctly
    let mut allocator: HandleAllocator<TestResource> = HandleAllocator::new();

    let h1 = allocator.allocate();
    assert_eq!(h1.generation(), 1);

    allocator.clear();
    let h2 = allocator.allocate();
    assert_eq!(h2.generation(), 2);

    allocator.clear();
    let h3 = allocator.allocate();
    assert_eq!(h3.generation(), 3);

    allocator.clear();
    let h4 = allocator.allocate();
    assert_eq!(h4.generation(), 4);

    // Only the last one is alive
    assert!(!allocator.is_alive(h1));
    assert!(!allocator.is_alive(h2));
    assert!(!allocator.is_alive(h3));
    assert!(allocator.is_alive(h4));
}
