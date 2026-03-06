//! Tests for AssetHandleAllocator.

use super::super::allocator::AssetHandleAllocator;
use super::super::typed::AssetHandle;
use super::common::TestTexture;

#[test]
fn test_new() {
    let allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();
    assert_eq!(allocator.len(), 0);
    assert!(allocator.is_empty());
    assert_eq!(allocator.capacity(), 0);
}

#[test]
fn test_with_capacity() {
    let allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::with_capacity(100);
    assert!(allocator.is_empty());
}

#[test]
fn test_allocate() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let h1 = allocator.allocate();
    assert!(h1.is_valid());
    assert_eq!(h1.index(), 0);
    assert_eq!(h1.generation(), 1);

    let h2 = allocator.allocate();
    assert!(h2.is_valid());
    assert_eq!(h2.index(), 1);
    assert_eq!(h2.generation(), 1);

    assert_eq!(allocator.len(), 2);
}

#[test]
fn test_allocate_unique() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();

    // All handles should be unique
    for i in 0..handles.len() {
        for j in (i + 1)..handles.len() {
            assert_ne!(handles[i], handles[j]);
        }
    }
}

#[test]
fn test_deallocate() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let handle = allocator.allocate();
    assert!(allocator.is_alive(handle));
    assert_eq!(allocator.len(), 1);

    assert!(allocator.deallocate(handle));
    assert!(!allocator.is_alive(handle));
    assert_eq!(allocator.len(), 0);
}

#[test]
fn test_deallocate_invalid() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    // Invalid handle
    assert!(!allocator.deallocate(AssetHandle::INVALID));

    // Handle that was never allocated
    let fake_handle = AssetHandle::new(100, 1);
    assert!(!allocator.deallocate(fake_handle));
}

#[test]
fn test_deallocate_twice() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let handle = allocator.allocate();
    assert!(allocator.deallocate(handle));
    assert!(!allocator.deallocate(handle)); // Already deallocated
}

#[test]
fn test_is_alive() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let handle = allocator.allocate();
    assert!(allocator.is_alive(handle));

    allocator.deallocate(handle);
    assert!(!allocator.is_alive(handle));

    // INVALID is never alive
    assert!(!allocator.is_alive(AssetHandle::INVALID));
}

#[test]
fn test_slot_reuse() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let h1 = allocator.allocate();
    let original_index = h1.index();
    let original_gen = h1.generation();

    allocator.deallocate(h1);

    let h2 = allocator.allocate();

    // Same index, different generation
    assert_eq!(h2.index(), original_index);
    assert_eq!(h2.generation(), original_gen + 1);
    assert_ne!(h1, h2);
}

#[test]
fn test_slot_reuse_multiple() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    // Allocate 10 handles
    let handles: Vec<_> = (0..10).map(|_| allocator.allocate()).collect();

    // Deallocate first 5
    for h in &handles[..5] {
        allocator.deallocate(*h);
    }

    assert_eq!(allocator.len(), 5);
    assert_eq!(allocator.capacity(), 10);

    // Allocate 5 more - should reuse slots
    for _ in 0..5 {
        let h = allocator.allocate();
        assert!(h.index() < 5); // Reused slot
        assert_eq!(h.generation(), 2); // Second generation
    }

    assert_eq!(allocator.len(), 10);
    assert_eq!(allocator.capacity(), 10); // No new slots needed
}

#[test]
fn test_len_and_capacity() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 0);

    let h1 = allocator.allocate();
    let h2 = allocator.allocate();
    let h3 = allocator.allocate();

    assert_eq!(allocator.len(), 3);
    assert_eq!(allocator.capacity(), 3);

    allocator.deallocate(h2);

    assert_eq!(allocator.len(), 2);
    assert_eq!(allocator.capacity(), 3); // Capacity unchanged
}

#[test]
fn test_clear() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    let h1 = allocator.allocate();
    let h2 = allocator.allocate();
    let h3 = allocator.allocate();

    assert_eq!(allocator.len(), 3);

    allocator.clear();

    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 3); // Capacity retained

    // Old handles are stale
    assert!(!allocator.is_alive(h1));
    assert!(!allocator.is_alive(h2));
    assert!(!allocator.is_alive(h3));

    // New handles have incremented generations
    let h4 = allocator.allocate();
    assert_eq!(h4.generation(), 2);
}

#[test]
fn test_generation_at() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    assert_eq!(allocator.generation_at(0), None);

    let handle = allocator.allocate();
    assert_eq!(allocator.generation_at(0), Some(1));

    allocator.deallocate(handle);
    assert_eq!(allocator.generation_at(0), Some(2));
}

#[test]
fn test_default() {
    let allocator: AssetHandleAllocator<TestTexture> = Default::default();
    assert!(allocator.is_empty());
}

#[test]
fn test_debug() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();
    allocator.allocate();
    allocator.allocate();

    let debug_str = format!("{:?}", allocator);
    assert!(debug_str.contains("AssetHandleAllocator"));
    assert!(debug_str.contains("TestTexture"));
    assert!(debug_str.contains("len"));
    assert!(debug_str.contains("2"));
}

#[test]
fn test_stress_allocate_deallocate() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    // Allocate many
    let handles: Vec<_> = (0..10000).map(|_| allocator.allocate()).collect();
    assert_eq!(allocator.len(), 10000);

    // Deallocate all
    for h in handles {
        assert!(allocator.deallocate(h));
    }
    assert_eq!(allocator.len(), 0);
    assert_eq!(allocator.capacity(), 10000);
}

#[test]
fn test_stress_churn() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    // Simulate churn: allocate/deallocate repeatedly
    for _ in 0..100 {
        let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();
        for h in &handles[..50] {
            allocator.deallocate(*h);
        }
        // Leave 50 alive
    }

    // Should have 5000 alive (100 iterations * 50 kept)
    assert_eq!(allocator.len(), 5000);
}

#[test]
fn test_generation_wrap() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    // Manually test generation wrapping
    let handle = allocator.allocate();

    // Deallocate original handle first - now it's stale
    allocator.deallocate(handle);
    assert!(!allocator.is_alive(handle));

    // Simulate many allocations/deallocations on the same slot
    for _ in 0..10 {
        let new_handle = allocator.allocate();
        allocator.deallocate(new_handle);
    }

    // Original handle should still be stale (gen is now 12, original was 1)
    assert!(!allocator.is_alive(handle));

    // Verify generation increased
    let gen = allocator.generation_at(0).unwrap();
    assert!(gen > handle.generation());
}

#[test]
fn test_shrink_to_fit() {
    let mut allocator: AssetHandleAllocator<TestTexture> = AssetHandleAllocator::new();

    // Allocate then deallocate many
    let handles: Vec<_> = (0..100).map(|_| allocator.allocate()).collect();
    for h in handles {
        allocator.deallocate(h);
    }

    // Free list should be large; shrink should not panic
    allocator.shrink_to_fit();
}

#[test]
fn test_is_send() {
    fn requires_send<T: Send>() {}
    requires_send::<AssetHandleAllocator<TestTexture>>();
}

// Note: AssetHandleAllocator is NOT Sync (contains Vec which isn't Sync for &mut)
// This is intentional - allocators should be accessed through synchronization primitives
