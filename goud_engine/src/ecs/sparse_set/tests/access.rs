//! Get, contains, len, and clear tests for [`SparseSet`].

#[cfg(test)]
mod tests {
    use crate::ecs::{Entity, SparseSet};

    // =========================================================================
    // Get Tests
    // =========================================================================

    #[test]
    fn test_get() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);

        assert_eq!(set.get(entity), Some(&42));
    }

    #[test]
    fn test_get_nonexistent() {
        let set: SparseSet<i32> = SparseSet::new();
        let entity = Entity::new(0, 1);

        assert_eq!(set.get(entity), None);
    }

    #[test]
    fn test_get_placeholder() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        assert_eq!(set.get(Entity::PLACEHOLDER), None);
    }

    #[test]
    fn test_get_out_of_bounds() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        // Entity index beyond sparse array
        let entity = Entity::new(1000, 1);
        assert_eq!(set.get(entity), None);
    }

    #[test]
    fn test_get_mut() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);

        if let Some(value) = set.get_mut(entity) {
            *value = 100;
        }

        assert_eq!(set.get(entity), Some(&100));
    }

    #[test]
    fn test_get_mut_nonexistent() {
        let mut set: SparseSet<i32> = SparseSet::new();
        let entity = Entity::new(0, 1);

        assert_eq!(set.get_mut(entity), None);
    }

    // =========================================================================
    // Contains Tests
    // =========================================================================

    #[test]
    fn test_contains() {
        let mut set = SparseSet::new();
        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 42);

        assert!(set.contains(e1));
        assert!(!set.contains(e2));
    }

    #[test]
    fn test_contains_placeholder() {
        let set: SparseSet<i32> = SparseSet::new();
        assert!(!set.contains(Entity::PLACEHOLDER));
    }

    #[test]
    fn test_contains_after_remove() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 42);
        assert!(set.contains(entity));

        set.remove(entity);
        assert!(!set.contains(entity));
    }

    // =========================================================================
    // Len / IsEmpty Tests
    // =========================================================================

    #[test]
    fn test_len_empty() {
        let set: SparseSet<i32> = SparseSet::new();
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_len_after_insert() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        assert_eq!(set.len(), 1);

        set.insert(Entity::new(1, 1), 2);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_len_after_remove() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 1);
        set.insert(e2, 2);
        assert_eq!(set.len(), 2);

        set.remove(e1);
        assert_eq!(set.len(), 1);

        set.remove(e2);
        assert_eq!(set.len(), 0);
        assert!(set.is_empty());
    }

    #[test]
    fn test_len_after_replace() {
        let mut set = SparseSet::new();
        let entity = Entity::new(0, 1);

        set.insert(entity, 1);
        assert_eq!(set.len(), 1);

        set.insert(entity, 2); // Replace
        assert_eq!(set.len(), 1); // Still 1
    }

    // =========================================================================
    // Clear Tests
    // =========================================================================

    #[test]
    fn test_clear() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        assert_eq!(set.len(), 3);

        set.clear();

        assert!(set.is_empty());
        assert_eq!(set.get(Entity::new(0, 1)), None);
        assert_eq!(set.get(Entity::new(1, 1)), None);
        assert_eq!(set.get(Entity::new(2, 1)), None);
    }
}
