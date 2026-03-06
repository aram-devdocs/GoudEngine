//! Insert and remove tests for [`SparseSet`].

#[cfg(test)]
mod tests {
    use crate::ecs::{Entity, SparseSet};

    // =========================================================================
    // Insert Tests
    // =========================================================================

    #[test]
    fn test_insert_single() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        let old = set.insert(entity, 42);
        assert_eq!(old, None);
        assert_eq!(set.len(), 1);
        assert!(set.contains(entity));
        assert_eq!(set.get(entity), Some(&42));
    }

    #[test]
    fn test_insert_multiple() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(5, 1);
        let e3 = Entity::new(10, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");
        set.insert(e3, "c");

        assert_eq!(set.len(), 3);
        assert_eq!(set.get(e1), Some(&"a"));
        assert_eq!(set.get(e2), Some(&"b"));
        assert_eq!(set.get(e3), Some(&"c"));
    }

    #[test]
    fn test_insert_replace() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        let old1 = set.insert(entity, 10);
        assert_eq!(old1, None);

        let old2 = set.insert(entity, 20);
        assert_eq!(old2, Some(10));

        assert_eq!(set.len(), 1); // Still just one entry
        assert_eq!(set.get(entity), Some(&20));
    }

    #[test]
    #[should_panic(expected = "Cannot insert with placeholder")]
    fn test_insert_placeholder_panics() {
        let mut set: SparseSet<i32> = SparseSet::new();
        set.insert(Entity::PLACEHOLDER, 42);
    }

    #[test]
    fn test_insert_sparse_indices() {
        // Test with widely spread entity indices
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1000, 1);
        let e3 = Entity::new(5000, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);
        set.insert(e3, 3);

        assert_eq!(set.len(), 3);
        assert_eq!(set.get(e1), Some(&1));
        assert_eq!(set.get(e2), Some(&2));
        assert_eq!(set.get(e3), Some(&3));
    }

    // =========================================================================
    // Remove Tests
    // =========================================================================

    #[test]
    fn test_remove_single() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);
        let removed = set.remove(entity);

        assert_eq!(removed, Some(42));
        assert!(set.is_empty());
        assert!(!set.contains(entity));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut set: SparseSet<i32> = SparseSet::new();
        let entity = Entity::new(0, 1);

        let removed = set.remove(entity);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_remove_placeholder() {
        let mut set: SparseSet<i32> = SparseSet::new();
        let removed = set.remove(Entity::PLACEHOLDER);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_remove_double() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);

        let first = set.remove(entity);
        let second = set.remove(entity);

        assert_eq!(first, Some(42));
        assert_eq!(second, None);
    }

    #[test]
    fn test_remove_swap_correctness() {
        // Test that swap-remove maintains correctness
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        set.insert(e1, "first");
        set.insert(e2, "second");
        set.insert(e3, "third");

        // Remove e1 (first) - e3 should be swapped in
        set.remove(e1);

        // e2 and e3 should still be accessible
        assert_eq!(set.get(e2), Some(&"second"));
        assert_eq!(set.get(e3), Some(&"third"));
        assert_eq!(set.len(), 2);

        // Dense array should have e3 at index 0, e2 at index 1
        let entities: Vec<_> = set.entities().collect();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    #[test]
    fn test_remove_middle() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);
        set.insert(e3, 3);

        // Remove middle element
        set.remove(e2);

        assert_eq!(set.get(e1), Some(&1));
        assert_eq!(set.get(e2), None);
        assert_eq!(set.get(e3), Some(&3));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_remove_last() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);

        // Remove last element (no swap needed)
        set.remove(e2);

        assert_eq!(set.get(e1), Some(&1));
        assert_eq!(set.get(e2), None);
        assert_eq!(set.len(), 1);
    }
}
