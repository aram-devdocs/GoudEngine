use super::TestResource;
use crate::core::handle::HandleMap;

#[test]
fn test_handle_map_iter_basic() {
    // Test basic iteration over handle-value pairs
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let h1 = map.insert(10);
    let h2 = map.insert(20);
    let h3 = map.insert(30);

    // Collect all pairs
    let pairs: Vec<_> = map.iter().collect();

    assert_eq!(pairs.len(), 3, "Should iterate over 3 entries");

    // Verify all handles are present
    let handles: Vec<_> = pairs.iter().map(|(h, _)| *h).collect();
    assert!(handles.contains(&h1));
    assert!(handles.contains(&h2));
    assert!(handles.contains(&h3));

    // Verify all values are present
    let values: Vec<_> = pairs.iter().map(|(_, v)| **v).collect();
    assert!(values.contains(&10));
    assert!(values.contains(&20));
    assert!(values.contains(&30));
}

#[test]
fn test_handle_map_iter_empty() {
    // Test iteration over empty map
    let map: HandleMap<TestResource, i32> = HandleMap::new();

    let pairs: Vec<_> = map.iter().collect();

    assert!(pairs.is_empty(), "Empty map should yield no items");
}

#[test]
fn test_handle_map_iter_skips_removed() {
    // Test that iteration skips removed entries
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    let h1 = map.insert(10);
    let h2 = map.insert(20);
    let h3 = map.insert(30);

    // Remove the middle entry
    map.remove(h2);

    // Iterate and collect
    let pairs: Vec<_> = map.iter().collect();

    assert_eq!(pairs.len(), 2, "Should only iterate over 2 entries");

    let handles: Vec<_> = pairs.iter().map(|(h, _)| *h).collect();
    assert!(handles.contains(&h1), "Should contain h1");
    assert!(!handles.contains(&h2), "Should NOT contain removed h2");
    assert!(handles.contains(&h3), "Should contain h3");
}

#[test]
fn test_handle_map_iter_mut_basic() {
    // Test mutable iteration
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.insert(1);
    map.insert(2);
    map.insert(3);

    // Double all values
    for (_, value) in map.iter_mut() {
        *value *= 2;
    }

    // Verify values are doubled
    let values: Vec<_> = map.values().cloned().collect();
    assert!(values.contains(&2), "1*2 should be 2");
    assert!(values.contains(&4), "2*2 should be 4");
    assert!(values.contains(&6), "3*2 should be 6");
}

#[test]
fn test_handle_map_iter_mut_modifies_in_place() {
    // Test that modifications are visible via handles
    let mut map: HandleMap<TestResource, String> = HandleMap::new();

    let h1 = map.insert("hello".to_string());
    let h2 = map.insert("world".to_string());

    // Append to all values
    for (_, value) in map.iter_mut() {
        value.push_str("!");
    }

    assert_eq!(map.get(h1).unwrap(), "hello!");
    assert_eq!(map.get(h2).unwrap(), "world!");
}

#[test]
fn test_handle_map_handles_iterator() {
    // Test handles() iterator
    let mut map: HandleMap<TestResource, &str> = HandleMap::new();

    let h1 = map.insert("a");
    let h2 = map.insert("b");
    let h3 = map.insert("c");

    let handles: Vec<_> = map.handles().collect();

    assert_eq!(handles.len(), 3);
    assert!(handles.contains(&h1));
    assert!(handles.contains(&h2));
    assert!(handles.contains(&h3));
}

#[test]
fn test_handle_map_values_iterator() {
    // Test values() iterator
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.insert(100);
    map.insert(200);
    map.insert(300);

    // Use values iterator to sum
    let sum: i32 = map.values().sum();
    assert_eq!(sum, 600);

    // Collect values
    let mut values: Vec<_> = map.values().cloned().collect();
    values.sort();
    assert_eq!(values, vec![100, 200, 300]);
}

#[test]
fn test_handle_map_values_mut_iterator() {
    // Test values_mut() iterator
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.insert(1);
    map.insert(2);
    map.insert(3);

    // Add 100 to all values
    for value in map.values_mut() {
        *value += 100;
    }

    let mut values: Vec<_> = map.values().cloned().collect();
    values.sort();
    assert_eq!(values, vec![101, 102, 103]);
}

#[test]
fn test_handle_map_into_iterator_ref() {
    // Test IntoIterator for &HandleMap (for loop support)
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.insert(10);
    map.insert(20);

    let mut count = 0;
    for (handle, value) in &map {
        assert!(handle.is_valid());
        assert!(*value == 10 || *value == 20);
        count += 1;
    }

    assert_eq!(count, 2);
}

#[test]
fn test_handle_map_into_iterator_mut_ref() {
    // Test IntoIterator for &mut HandleMap (mutable for loop support)
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.insert(1);
    map.insert(2);
    map.insert(3);

    // Triple all values using for loop
    for (_, value) in &mut map {
        *value *= 3;
    }

    let mut values: Vec<_> = map.values().cloned().collect();
    values.sort();
    assert_eq!(values, vec![3, 6, 9]);
}

#[test]
fn test_handle_map_iter_with_gaps() {
    // Test iteration with multiple gaps from removals
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    // Insert 10 values
    let handles: Vec<_> = (0..10).map(|i| map.insert(i)).collect();

    // Remove every other one (0, 2, 4, 6, 8)
    for i in (0..10).step_by(2) {
        map.remove(handles[i]);
    }

    // Should have 5 values remaining (1, 3, 5, 7, 9)
    let remaining: Vec<_> = map.values().cloned().collect();
    assert_eq!(remaining.len(), 5);

    let mut sorted = remaining.clone();
    sorted.sort();
    assert_eq!(sorted, vec![1, 3, 5, 7, 9]);
}

#[test]
fn test_handle_map_iter_count() {
    // Test that iter().count() matches len()
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    for i in 0..100 {
        map.insert(i);
    }

    assert_eq!(map.iter().count(), 100);
    assert_eq!(map.iter().count(), map.len());

    // Remove some
    let handles: Vec<_> = map.handles().take(30).collect();
    for h in handles {
        map.remove(h);
    }

    assert_eq!(map.iter().count(), 70);
    assert_eq!(map.iter().count(), map.len());
}

#[test]
fn test_handle_map_iter_size_hint() {
    // Test size_hint is reasonable
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    for i in 0..10 {
        map.insert(i);
    }

    let iter = map.iter();
    let (lower, upper) = iter.size_hint();

    // Lower bound is 0 (conservative)
    assert_eq!(lower, 0);

    // Upper bound should be at most the capacity
    assert!(upper.is_some());
    assert!(upper.unwrap() <= 10);
}

#[test]
fn test_handle_map_iter_after_clear() {
    // Test iteration after clear
    let mut map: HandleMap<TestResource, i32> = HandleMap::new();

    map.insert(1);
    map.insert(2);
    map.insert(3);

    map.clear();

    let count = map.iter().count();
    assert_eq!(count, 0, "Iteration after clear should yield nothing");

    // Insert new values
    map.insert(100);
    map.insert(200);

    let count = map.iter().count();
    assert_eq!(count, 2, "Should iterate over new values");
}

#[test]
fn test_handle_map_iter_stress() {
    // Stress test iteration with many operations
    const COUNT: usize = 1000;

    let mut map: HandleMap<TestResource, usize> = HandleMap::new();

    // Insert values
    let handles: Vec<_> = (0..COUNT).map(|i| map.insert(i)).collect();

    // Verify iteration count
    assert_eq!(map.iter().count(), COUNT);

    // Remove half
    for (i, h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            map.remove(*h);
        }
    }

    // Verify iteration count
    assert_eq!(map.iter().count(), COUNT / 2);

    // Verify remaining values
    let remaining: std::collections::HashSet<_> = map.values().cloned().collect();
    for i in 0..COUNT {
        if i % 2 == 0 {
            assert!(
                !remaining.contains(&i),
                "Removed value {} should not be present",
                i
            );
        } else {
            assert!(remaining.contains(&i), "Kept value {} should be present", i);
        }
    }
}
