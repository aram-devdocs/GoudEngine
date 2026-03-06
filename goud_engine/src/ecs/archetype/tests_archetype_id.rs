//! Tests for [`ArchetypeId`].

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::ecs::archetype::ArchetypeId;

    // ==================== Structure Tests ====================

    #[test]
    fn test_archetype_id_empty_constant() {
        let empty = ArchetypeId::EMPTY;
        assert_eq!(empty.index(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_archetype_id_new() {
        let id = ArchetypeId::new(42);
        assert_eq!(id.index(), 42);
        assert!(!id.is_empty());
    }

    #[test]
    fn test_archetype_id_new_zero() {
        let id = ArchetypeId::new(0);
        assert_eq!(id.index(), 0);
        assert!(id.is_empty());
        assert_eq!(id, ArchetypeId::EMPTY);
    }

    #[test]
    fn test_archetype_id_max_value() {
        let id = ArchetypeId::new(u32::MAX);
        assert_eq!(id.index(), u32::MAX);
        assert!(!id.is_empty());
    }

    // ==================== Trait Implementation Tests ====================

    #[test]
    fn test_archetype_id_clone() {
        let id1 = ArchetypeId::new(5);
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_archetype_id_copy() {
        let id1 = ArchetypeId::new(5);
        let id2 = id1; // Copy
        assert_eq!(id1, id2);
        assert_eq!(id1.index(), 5);
    }

    #[test]
    fn test_archetype_id_equality() {
        let id1 = ArchetypeId::new(10);
        let id2 = ArchetypeId::new(10);
        let id3 = ArchetypeId::new(20);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_archetype_id_hash() {
        let mut set = HashSet::new();
        let id1 = ArchetypeId::new(1);
        let id2 = ArchetypeId::new(2);
        let id1_dup = ArchetypeId::new(1);

        set.insert(id1);
        set.insert(id2);
        set.insert(id1_dup); // Should not increase size

        assert_eq!(set.len(), 2);
        assert!(set.contains(&id1));
        assert!(set.contains(&id2));
    }

    #[test]
    fn test_archetype_id_as_hashmap_key() {
        let mut map: HashMap<ArchetypeId, &str> = HashMap::new();
        map.insert(ArchetypeId::EMPTY, "empty");
        map.insert(ArchetypeId::new(1), "first");
        map.insert(ArchetypeId::new(2), "second");

        assert_eq!(map.get(&ArchetypeId::EMPTY), Some(&"empty"));
        assert_eq!(map.get(&ArchetypeId::new(1)), Some(&"first"));
        assert_eq!(map.get(&ArchetypeId::new(2)), Some(&"second"));
        assert_eq!(map.get(&ArchetypeId::new(99)), None);
    }

    #[test]
    fn test_archetype_id_debug_empty() {
        assert_eq!(format!("{:?}", ArchetypeId::EMPTY), "ArchetypeId(EMPTY)");
    }

    #[test]
    fn test_archetype_id_debug_non_empty() {
        assert_eq!(format!("{:?}", ArchetypeId::new(42)), "ArchetypeId(42)");
    }

    #[test]
    fn test_archetype_id_display() {
        assert_eq!(format!("{}", ArchetypeId::EMPTY), "0");
        assert_eq!(format!("{}", ArchetypeId::new(42)), "42");
    }

    #[test]
    fn test_archetype_id_default() {
        let default = ArchetypeId::default();
        assert_eq!(default, ArchetypeId::EMPTY);
        assert!(default.is_empty());
    }

    // ==================== Conversion Tests ====================

    #[test]
    fn test_archetype_id_from_u32() {
        let id: ArchetypeId = 100u32.into();
        assert_eq!(id.index(), 100);
    }

    #[test]
    fn test_archetype_id_into_u32() {
        let id = ArchetypeId::new(100);
        let val: u32 = id.into();
        assert_eq!(val, 100);
    }

    #[test]
    fn test_archetype_id_into_usize() {
        let id = ArchetypeId::new(42);
        let idx: usize = id.into();
        assert_eq!(idx, 42);
    }

    // ==================== Size and Layout Tests ====================

    #[test]
    fn test_archetype_id_size() {
        assert_eq!(std::mem::size_of::<ArchetypeId>(), 4);
    }

    #[test]
    fn test_archetype_id_alignment() {
        assert_eq!(std::mem::align_of::<ArchetypeId>(), 4);
    }

    // ==================== Thread Safety Tests ====================

    #[test]
    fn test_archetype_id_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ArchetypeId>();
    }

    #[test]
    fn test_archetype_id_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<ArchetypeId>();
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_archetype_id_sequential_creation() {
        let ids: Vec<ArchetypeId> = (0..100).map(ArchetypeId::new).collect();
        for (i, id) in ids.iter().enumerate() {
            assert_eq!(id.index() as usize, i);
        }
    }

    #[test]
    fn test_archetype_id_const_new() {
        const ID: ArchetypeId = ArchetypeId::new(42);
        assert_eq!(ID.index(), 42);
    }

    #[test]
    fn test_archetype_id_const_empty() {
        const EMPTY: ArchetypeId = ArchetypeId::EMPTY;
        assert!(EMPTY.is_empty());
    }
}
