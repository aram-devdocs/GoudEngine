use super::TestResource;
use crate::core::handle::{Handle, HandleMap};

#[test]
fn test_handle_map_new() {
    // Test that new() creates an empty map
    let map: HandleMap<TestResource, i32> = HandleMap::new();

    assert_eq!(map.len(), 0, "New map should have len 0");
    assert!(map.is_empty(), "New map should be empty");
    assert_eq!(map.capacity(), 0, "New map should have capacity 0");
}

#[test]
fn test_handle_map_default() {
    // Test that Default is same as new()
    let map: HandleMap<TestResource, String> = HandleMap::default();

    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_handle_map_with_capacity() {
    // Test with_capacity pre-allocates
    let map: HandleMap<TestResource, i32> = HandleMap::with_capacity(100);

    assert_eq!(map.len(), 0, "with_capacity should not insert values");
    assert!(map.is_empty());
}

#[test]
fn test_handle_map_insert_single() {
    // Test inserting a single value
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let handle = map.insert(42);

    assert!(handle.is_valid(), "Returned handle should be valid");
    assert_eq!(map.len(), 1, "Map should have 1 entry");
    assert!(!map.is_empty(), "Map should not be empty");
    assert_eq!(map.capacity(), 1);
}

#[test]
fn test_handle_map_insert_multiple() {
    // Test inserting multiple values
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let h1 = map.insert(10);
    let h2 = map.insert(20);
    let h3 = map.insert(30);

    // All should be valid and unique
    assert!(h1.is_valid());
    assert!(h2.is_valid());
    assert!(h3.is_valid());

    assert_ne!(h1, h2, "Handles should be unique");
    assert_ne!(h2, h3, "Handles should be unique");
    assert_ne!(h1, h3, "Handles should be unique");

    assert_eq!(map.len(), 3);
}

#[test]
fn test_handle_map_get() {
    // Test retrieving values by handle
    let mut map: HandleMap<TestResource, String> = HandleMap::new();

    let handle = map.insert("hello".to_string());

    let value = map.get(handle);
    assert!(value.is_some(), "get should return Some for valid handle");
    assert_eq!(value.unwrap(), "hello");
}

#[test]
fn test_handle_map_get_invalid_handle() {
    // Test get with invalid/stale handles
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    // Get with INVALID handle
    assert!(
        map.get(Handle::INVALID).is_none(),
        "get with INVALID should return None"
    );

    // Get with fabricated handle
    let fake = Handle::<TestResource>::new(100, 1);
    assert!(
        map.get(fake).is_none(),
        "get with out-of-bounds handle should return None"
    );

    // Insert and then get with wrong generation
    let handle = map.insert(42);
    let wrong_gen = Handle::<TestResource>::new(handle.index(), handle.generation() + 1);
    assert!(
        map.get(wrong_gen).is_none(),
        "get with wrong generation should return None"
    );
}

#[test]
fn test_handle_map_get_mut() {
    // Test mutable access
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let handle = map.insert(100);

    // Modify the value
    if let Some(value) = map.get_mut(handle) {
        *value = 200;
    }

    // Verify modification
    assert_eq!(map.get(handle), Some(&200));
}

#[test]
fn test_handle_map_get_mut_invalid() {
    // Test get_mut with invalid handles
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    assert!(map.get_mut(Handle::INVALID).is_none());

    let handle = map.insert(42);
    map.remove(handle);
    assert!(
        map.get_mut(handle).is_none(),
        "get_mut on removed handle should return None"
    );
}

#[test]
fn test_handle_map_contains() {
    // Test contains method
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let handle = map.insert(42);

    assert!(
        map.contains(handle),
        "contains should return true for valid handle"
    );
    assert!(
        !map.contains(Handle::INVALID),
        "contains should return false for INVALID"
    );

    map.remove(handle);
    assert!(
        !map.contains(handle),
        "contains should return false after removal"
    );
}

#[test]
fn test_handle_map_remove() {
    // Test removing values
    let mut map: HandleMap<TestResource, String> = HandleMap::new();

    let handle = map.insert("to_remove".to_string());
    assert_eq!(map.len(), 1);

    let removed = map.remove(handle);
    assert!(removed.is_some(), "remove should return Some");
    assert_eq!(removed.unwrap(), "to_remove");
    assert_eq!(map.len(), 0);
    assert!(map.is_empty());
}

#[test]
fn test_handle_map_remove_returns_none_for_invalid() {
    // Test remove with invalid handles
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    // Remove INVALID
    assert!(map.remove(Handle::INVALID).is_none());

    // Remove fabricated handle
    let fake = Handle::<TestResource>::new(100, 1);
    assert!(map.remove(fake).is_none());

    // Double remove
    let handle = map.insert(42);
    assert!(map.remove(handle).is_some());
    assert!(
        map.remove(handle).is_none(),
        "Second remove should return None"
    );
}

#[test]
fn test_handle_map_remove_drops_value() {
    // Test that removed values are actually dropped
    use std::cell::RefCell;
    use std::rc::Rc;

    let drop_counter = Rc::new(RefCell::new(0));

    struct DropTracker {
        counter: Rc<RefCell<i32>>,
    }

    impl Drop for DropTracker {
        fn drop(&mut self) {
            *self.counter.borrow_mut() += 1;
        }
    }

    let mut map: HandleMap<TestResource, DropTracker> = HandleMap::new();

    let handle = map.insert(DropTracker {
        counter: drop_counter.clone(),
    });

    assert_eq!(*drop_counter.borrow(), 0, "Not dropped yet");

    let removed = map.remove(handle);
    assert_eq!(*drop_counter.borrow(), 0, "Still held by removed");

    drop(removed);
    assert_eq!(
        *drop_counter.borrow(),
        1,
        "Dropped after remove result dropped"
    );
}

#[test]
fn test_handle_map_slot_reuse() {
    // Test that removed slots are reused
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let h1 = map.insert(100);
    assert_eq!(h1.index(), 0);
    assert_eq!(h1.generation(), 1);

    map.remove(h1);

    // Next insert should reuse slot 0 with incremented generation
    let h2 = map.insert(200);
    assert_eq!(h2.index(), 0, "Should reuse slot 0");
    assert_eq!(h2.generation(), 2, "Generation should be incremented");

    // Verify values
    assert!(map.get(h1).is_none(), "Old handle should be stale");
    assert_eq!(map.get(h2), Some(&200), "New handle should work");

    // Capacity should not have grown
    assert_eq!(map.capacity(), 1);
}

#[test]
fn test_handle_map_generation_safety() {
    // Test that generational indices prevent ABA problem
    let mut map: HandleMap<TestResource, &str> = HandleMap::new();

    // Insert value A at slot 0
    let h_a = map.insert("A");
    assert_eq!(map.get(h_a), Some(&"A"));

    // Remove A
    map.remove(h_a);

    // Insert value B (reuses slot 0)
    let h_b = map.insert("B");
    assert_eq!(h_a.index(), h_b.index(), "Same slot reused");

    // Old handle h_a should NOT access B (generation mismatch)
    assert!(map.get(h_a).is_none(), "Stale handle should return None");
    assert_eq!(map.get(h_b), Some(&"B"), "New handle should work");
}

#[test]
fn test_handle_map_clear() {
    // Test clearing the map
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let h1 = map.insert(10);
    let h2 = map.insert(20);
    let h3 = map.insert(30);

    assert_eq!(map.len(), 3);

    map.clear();

    assert_eq!(map.len(), 0);
    assert!(map.is_empty());

    // All handles should be invalid
    assert!(!map.contains(h1));
    assert!(!map.contains(h2));
    assert!(!map.contains(h3));

    assert!(map.get(h1).is_none());
    assert!(map.get(h2).is_none());
    assert!(map.get(h3).is_none());

    // Capacity retained
    assert_eq!(map.capacity(), 3);
}

#[test]
fn test_handle_map_clear_and_reinsert() {
    // Test reinserting after clear
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let h1 = map.insert(100);
    assert_eq!(h1.generation(), 1);

    map.clear();

    let h2 = map.insert(200);
    assert_eq!(h2.index(), h1.index(), "Should reuse slot");
    assert_eq!(h2.generation(), 2, "Generation should be incremented");

    assert!(map.get(h1).is_none());
    assert_eq!(map.get(h2), Some(&200));
}

#[test]
fn test_handle_map_len_and_capacity() {
    // Test len() and capacity() behavior
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    assert_eq!(map.len(), 0);
    assert_eq!(map.capacity(), 0);

    let h1 = map.insert(1);
    let h2 = map.insert(2);
    let h3 = map.insert(3);

    assert_eq!(map.len(), 3);
    assert_eq!(map.capacity(), 3);

    // Remove one
    map.remove(h2);

    assert_eq!(map.len(), 2, "len should decrease");
    assert_eq!(map.capacity(), 3, "capacity should not decrease");

    // Insert one (reuses slot)
    let _h4 = map.insert(4);

    assert_eq!(map.len(), 3);
    assert_eq!(map.capacity(), 3, "capacity unchanged when reusing");

    // Insert another (new slot)
    let _h5 = map.insert(5);

    assert_eq!(map.len(), 4);
    assert_eq!(map.capacity(), 4);

    // Clean up to verify
    map.remove(h1);
    map.remove(h3);

    assert_eq!(map.len(), 2);
    assert_eq!(map.capacity(), 4);
}

#[test]
fn test_handle_map_debug() {
    // Test Debug formatting
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();
    map.insert(1);
    map.insert(2);

    let debug_str = format!("{:?}", map);

    assert!(
        debug_str.contains("HandleMap"),
        "Debug should contain type name"
    );
    assert!(debug_str.contains("len"), "Debug should show len");
    assert!(debug_str.contains("capacity"), "Debug should show capacity");
}

#[test]
fn test_handle_map_reserve() {
    // Test reserve functionality
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.reserve(100);

    // Can insert without reallocation
    for i in 0..100 {
        map.insert(i);
    }

    assert_eq!(map.len(), 100);
}

#[test]
fn test_handle_map_shrink_to_fit() {
    // Test shrink_to_fit
    let mut map: HandleMap<TestResource, i32> = HandleMap::with_capacity(100);

    for i in 0..10 {
        map.insert(i);
    }

    map.shrink_to_fit();

    // Functionality preserved
    assert_eq!(map.len(), 10);
}
