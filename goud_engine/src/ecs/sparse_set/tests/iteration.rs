//! Iteration tests for [`SparseSet`].

#[cfg(test)]
mod tests {
    use crate::ecs::{Entity, SparseSet};

    #[test]
    fn test_iter() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(1, 1);
        let e3 = Entity::new(2, 1);

        set.insert(e1, 10);
        set.insert(e2, 20);
        set.insert(e3, 30);

        let mut items: Vec<_> = set.iter().map(|(e, v)| (e, *v)).collect();
        items.sort_by_key(|(e, _)| e.index());

        assert_eq!(items, vec![(e1, 10), (e2, 20), (e3, 30)]);
    }

    #[test]
    fn test_iter_empty() {
        let set: SparseSet<i32> = SparseSet::new();
        let items: Vec<_> = set.iter().collect();
        assert!(items.is_empty());
    }

    #[test]
    fn test_iter_mut() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        for (_, value) in set.iter_mut() {
            *value *= 10;
        }

        assert_eq!(set.get(Entity::new(0, 1)), Some(&10));
        assert_eq!(set.get(Entity::new(1, 1)), Some(&20));
        assert_eq!(set.get(Entity::new(2, 1)), Some(&30));
    }

    #[test]
    fn test_entities() {
        let mut set = SparseSet::new();

        let e1 = Entity::new(0, 1);
        let e2 = Entity::new(5, 1);
        let e3 = Entity::new(10, 1);

        set.insert(e1, "a");
        set.insert(e2, "b");
        set.insert(e3, "c");

        let entities: Vec<_> = set.entities().collect();
        assert_eq!(entities.len(), 3);
        assert!(entities.contains(&e1));
        assert!(entities.contains(&e2));
        assert!(entities.contains(&e3));
    }

    #[test]
    fn test_values() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 10);
        set.insert(Entity::new(1, 1), 20);
        set.insert(Entity::new(2, 1), 30);

        let sum: i32 = set.values().sum();
        assert_eq!(sum, 60);
    }

    #[test]
    fn test_values_mut() {
        let mut set = SparseSet::new();

        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        for value in set.values_mut() {
            *value += 10;
        }

        let sum: i32 = set.values().sum();
        assert_eq!(sum, 36); // 11 + 12 + 13
    }

    #[test]
    fn test_into_iter_ref() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        let mut count = 0;
        for (_, _) in &set {
            count += 1;
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn test_into_iter_mut() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 42);

        for (_, value) in &mut set {
            *value = 100;
        }

        assert_eq!(set.get(Entity::new(0, 1)), Some(&100));
    }

    #[test]
    fn test_iter_size_hint() {
        let mut set = SparseSet::new();
        set.insert(Entity::new(0, 1), 1);
        set.insert(Entity::new(1, 1), 2);
        set.insert(Entity::new(2, 1), 3);

        let iter = set.iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.len(), 3);
    }
}
