//! Advanced accessor, clone, stress, and thread-safety tests for [`SparseSet`].

#[cfg(test)]
mod tests {
    use crate::ecs::{Entity, SparseSet};

    // =========================================================================
    // Advanced Method Tests
    // =========================================================================

    #[test]
    fn test_dense() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");

        let dense = set.dense();
        assert_eq!(dense.len(), 2);
        assert_eq!(dense[0], e1);
        assert_eq!(dense[1], e2);
    }

    #[test]
    fn test_dense_index() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(5, 1);
        let e2 = Entity::new(10, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");

        assert_eq!(set.dense_index(e1), Some(0));
        assert_eq!(set.dense_index(e2), Some(1));
        assert_eq!(set.dense_index(Entity::new(0, 1)), None);
        assert_eq!(set.dense_index(Entity::PLACEHOLDER), None);
    }

    #[test]
    fn test_get_by_dense_index() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), "first");
        set.insert(Entity::new(1, 1), "second");

        assert_eq!(set.get_by_dense_index(0), Some(&"first"));
        assert_eq!(set.get_by_dense_index(1), Some(&"second"));
        assert_eq!(set.get_by_dense_index(2), None);
    }

    #[test]
    fn test_get_mut_by_dense_index() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 10);
        set.insert(Entity::new(1, 1), 20);

        if let Some(value) = set.get_mut_by_dense_index(0) {
            *value = 100;
        }

        assert_eq!(set.get(Entity::new(0, 1)), Some(&100));
    }

    // =========================================================================
    // Clone Tests
    // =========================================================================

    #[test]
    fn test_clone() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);

        set.insert(e1, 10);
        set.insert(e2, 20);

        let cloned = set.clone();

        assert_eq!(cloned.len(), 2);
        assert_eq!(cloned.get(e1), Some(&10));
        assert_eq!(cloned.get(e2), Some(&20));

        // Modifications to original don't affect clone
        set.insert(e1, 100);
        assert_eq!(cloned.get(e1), Some(&10));
    }

    // =========================================================================
    // Stress Tests
    // =========================================================================

    #[test]
    fn test_stress_many_entities() {
        let mut set = SparseSet::new();
        const COUNT: usize = 10_000;

        // Insert many entities
        for i in 0..COUNT {
            let entity = Entity::new(i as u32, 1);
            set.insert(entity, i as i32);
        }

        assert_eq!(set.len(), COUNT);

        // Verify all accessible
        for i in 0..COUNT {
            let entity = Entity::new(i as u32, 1);
            assert_eq!(set.get(entity), Some(&(i as i32)));
        }

        // Remove half
        for i in (0..COUNT).step_by(2) {
            let entity = Entity::new(i as u32, 1);
            set.remove(entity);
        }

        assert_eq!(set.len(), COUNT / 2);

        // Verify removed vs remaining
        for i in 0..COUNT {
            let entity = Entity::new(i as u32, 1);
            if i % 2 == 0 {
                assert_eq!(set.get(entity), None);
            } else {
                assert_eq!(set.get(entity), Some(&(i as i32)));
            }
        }
    }

    #[test]
    fn test_stress_sparse_indices() {
        let mut set = SparseSet::new();

        // Insert with very sparse indices
        let indices: Vec<u32> = vec![0, 100, 1000, 10000, 50000];

        for (i, &idx) in indices.iter().enumerate() {
            let entity = Entity::new(idx, 1);
            set.insert(entity, i as i32);
        }

        assert_eq!(set.len(), 5);

        // Verify all accessible
        for (i, &idx) in indices.iter().enumerate() {
            let entity = Entity::new(idx, 1);
            assert_eq!(set.get(entity), Some(&(i as i32)));
        }
    }

    #[test]
    fn test_stress_insert_remove_cycle() {
        let mut set = SparseSet::new();
        const ITERATIONS: usize = 100;
        const BATCH_SIZE: usize = 100;

        for _ in 0..ITERATIONS {
            // Insert batch
            for i in 0..BATCH_SIZE {
                let entity = Entity::new(i as u32, 1);
                set.insert(entity, i as i32);
            }

            assert_eq!(set.len(), BATCH_SIZE);

            // Remove all
            for i in 0..BATCH_SIZE {
                let entity = Entity::new(i as u32, 1);
                set.remove(entity);
            }

            assert!(set.is_empty());
        }
    }

    // =========================================================================
    // Thread Safety Tests (Compile-time)
    // =========================================================================

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        // SparseSet<T> is Send if T is Send
        assert_send::<SparseSet<i32>>();
        assert_send::<SparseSet<String>>();

        // SparseSet<T> is Sync if T is Sync
        assert_sync::<SparseSet<i32>>();
        assert_sync::<SparseSet<String>>();
    }
}
