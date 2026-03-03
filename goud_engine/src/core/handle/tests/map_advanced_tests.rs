use super::TestResource;
use crate::core::handle::HandleMap;

#[test]
fn test_handle_map_stress() {
    // Stress test with many operations
    const COUNT: usize = 10_000;

    let mut map: HandleMap<TestResource, usize> = HandleMap::with_capacity(COUNT);

    // Phase 1: Insert many
    let handles: Vec<_> = (0..COUNT).map(|i| map.insert(i)).collect();

    assert_eq!(map.len(), COUNT);

    // Verify all values
    for (i, h) in handles.iter().enumerate() {
        assert_eq!(map.get(*h), Some(&i), "Value {} should match", i);
    }

    // Phase 2: Remove half
    for h in handles.iter().step_by(2) {
        map.remove(*h);
    }

    assert_eq!(map.len(), COUNT / 2);

    // Phase 3: Verify removals
    for (i, h) in handles.iter().enumerate() {
        if i % 2 == 0 {
            assert!(map.get(*h).is_none(), "Removed value {} should be None", i);
        } else {
            assert_eq!(map.get(*h), Some(&i), "Kept value {} should exist", i);
        }
    }

    // Phase 4: Insert again (reuses slots)
    let new_handles: Vec<_> = (0..COUNT / 2).map(|i| map.insert(i + COUNT)).collect();

    assert_eq!(map.len(), COUNT);
    assert_eq!(
        map.capacity(),
        COUNT,
        "Should reuse slots, not grow capacity"
    );

    // Verify new values
    for (i, h) in new_handles.iter().enumerate() {
        assert_eq!(map.get(*h), Some(&(i + COUNT)));
    }

    // Phase 5: Clear
    map.clear();

    assert!(map.is_empty());
    assert_eq!(map.capacity(), COUNT);

    // All handles should be stale
    for h in handles.iter().take(10) {
        assert!(map.get(*h).is_none());
    }
}

#[test]
fn test_handle_map_values_with_complex_types() {
    // Test with complex value types
    #[derive(Debug, Clone, PartialEq)]
    struct ComplexData {
        id: u64,
        name: String,
        values: Vec<f32>,
    }

    let mut map: HandleMap<TestResource, ComplexData> = HandleMap::new();

    let h1 = map.insert(ComplexData {
        id: 1,
        name: "first".to_string(),
        values: vec![1.0, 2.0, 3.0],
    });

    let h2 = map.insert(ComplexData {
        id: 2,
        name: "second".to_string(),
        values: vec![4.0, 5.0],
    });

    // Access and verify
    assert_eq!(map.get(h1).unwrap().id, 1);
    assert_eq!(map.get(h1).unwrap().name, "first");
    assert_eq!(map.get(h2).unwrap().values.len(), 2);

    // Modify
    if let Some(data) = map.get_mut(h1) {
        data.values.push(4.0);
    }

    assert_eq!(map.get(h1).unwrap().values.len(), 4);

    // Remove and verify
    let removed = map.remove(h1);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().name, "first");

    assert!(map.get(h1).is_none());
    assert!(map.get(h2).is_some());
}
