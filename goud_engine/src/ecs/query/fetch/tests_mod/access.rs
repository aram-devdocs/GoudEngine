//! Tests for Access, AccessConflict, ConflictInfo, and related types.

#[cfg(test)]
mod tests {
    use crate::ecs::component::ComponentId;
    use crate::ecs::query::fetch::{Access, AccessConflict, AccessType, ConflictInfo};
    use crate::ecs::Component;

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }
    impl Component for Position {}

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    impl Component for Velocity {}

    #[derive(Debug, Clone, Copy)]
    struct Player;
    impl Component for Player {}

    // =========================================================================
    // Access Tests
    // =========================================================================

    mod access_tests {
        use super::*;

        #[test]
        fn test_access_new_is_empty() {
            let access = Access::new();
            assert!(access.is_read_only());
            assert_eq!(access.writes().len(), 0);
            assert_eq!(access.reads().count(), 0);
        }

        #[test]
        fn test_access_add_read() {
            let mut access = Access::new();
            let id = ComponentId::of::<Position>();
            access.add_read(id);

            assert!(access.is_read_only());
            assert!(access.reads().any(|&x| x == id));
            assert!(!access.writes().contains(&id));
        }

        #[test]
        fn test_access_add_write() {
            let mut access = Access::new();
            let id = ComponentId::of::<Position>();
            access.add_write(id);

            assert!(!access.is_read_only());
            assert!(access.writes().contains(&id));
        }

        #[test]
        fn test_access_write_counts_as_read() {
            let mut access = Access::new();
            let id = ComponentId::of::<Position>();
            access.add_write(id);

            assert!(access.writes().contains(&id));
            assert!(access.reads().any(|&x| x == id));
        }

        #[test]
        fn test_access_reads_only() {
            let mut access = Access::new();
            let pos_id = ComponentId::of::<Position>();
            let vel_id = ComponentId::of::<Velocity>();

            access.add_read(pos_id);
            access.add_write(vel_id);

            let reads_only: Vec<_> = access.reads_only().cloned().collect();
            assert!(reads_only.contains(&pos_id));
            assert!(!reads_only.contains(&vel_id));
        }

        #[test]
        fn test_access_no_conflict_between_reads() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();
            let id = ComponentId::of::<Position>();

            access1.add_read(id);
            access2.add_read(id);

            assert!(!access1.conflicts_with(&access2));
            assert!(!access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_conflict_write_read() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();
            let id = ComponentId::of::<Position>();

            access1.add_write(id);
            access2.add_read(id);

            assert!(access1.conflicts_with(&access2));
            assert!(access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_conflict_write_write() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();
            let id = ComponentId::of::<Position>();

            access1.add_write(id);
            access2.add_write(id);

            assert!(access1.conflicts_with(&access2));
            assert!(access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_no_conflict_different_components() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();

            access1.add_write(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            assert!(!access1.conflicts_with(&access2));
            assert!(!access2.conflicts_with(&access1));
        }

        #[test]
        fn test_access_extend() {
            let mut access1 = Access::new();
            let mut access2 = Access::new();

            access1.add_read(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            access1.extend(&access2);

            assert!(access1.reads().any(|&x| x == ComponentId::of::<Position>()));
            assert!(access1.writes().contains(&ComponentId::of::<Velocity>()));
        }

        #[test]
        fn test_access_is_read_only() {
            let mut read_access = Access::new();
            read_access.add_read(ComponentId::of::<Position>());
            assert!(read_access.is_read_only());

            let mut write_access = Access::new();
            write_access.add_write(ComponentId::of::<Position>());
            assert!(!write_access.is_read_only());
        }

        #[test]
        fn test_access_complex_scenario() {
            let mut access_a = Access::new();
            access_a.add_read(ComponentId::of::<Position>());
            access_a.add_write(ComponentId::of::<Velocity>());

            let mut access_b = Access::new();
            access_b.add_read(ComponentId::of::<Position>());
            access_b.add_read(ComponentId::of::<Velocity>());

            assert!(access_a.conflicts_with(&access_b));
            assert!(access_b.conflicts_with(&access_a));
        }

        #[test]
        fn test_access_no_conflict_complex() {
            let mut access_a = Access::new();
            access_a.add_write(ComponentId::of::<Position>());

            let mut access_b = Access::new();
            access_b.add_write(ComponentId::of::<Velocity>());
            access_b.add_read(ComponentId::of::<Player>());

            assert!(!access_a.conflicts_with(&access_b));
            assert!(!access_b.conflicts_with(&access_a));
        }

        #[test]
        fn test_access_is_empty() {
            let access = Access::new();
            assert!(access.is_empty());

            let mut not_empty = Access::new();
            not_empty.add_read(ComponentId::of::<Position>());
            assert!(!not_empty.is_empty());
        }

        #[test]
        fn test_access_clear() {
            let mut access = Access::new();
            access.add_read(ComponentId::of::<Position>());
            access.add_write(ComponentId::of::<Velocity>());

            assert!(!access.is_empty());
            access.clear();
            assert!(access.is_empty());
        }
    }

    // =========================================================================
    // get_conflicts tests
    // =========================================================================

    mod get_conflicts {
        use super::*;

        #[test]
        fn test_get_conflicts_no_conflict() {
            let mut access1 = Access::new();
            access1.add_read(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            assert!(access1.get_conflicts(&access2).is_none());
        }

        #[test]
        fn test_get_conflicts_write_read() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert_eq!(conflict.component_count(), 1);
            assert!(!conflict.has_write_write());

            let comp_conflict = &conflict.component_conflicts()[0];
            assert_eq!(comp_conflict.component_id, ComponentId::of::<Position>());
            assert_eq!(comp_conflict.first_access, AccessType::Write);
            assert_eq!(comp_conflict.second_access, AccessType::Read);
        }

        #[test]
        fn test_get_conflicts_write_write() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_write(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert!(conflict.has_write_write());
            assert!(conflict.component_conflicts()[0].is_write_write());
        }

        #[test]
        fn test_get_conflicts_multiple_components() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());
            access1.add_write(ComponentId::of::<Velocity>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());
            access2.add_read(ComponentId::of::<Velocity>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert_eq!(conflict.component_count(), 2);
            assert_eq!(conflict.total_count(), 2);
        }

        #[test]
        fn test_get_conflicts_partial() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());
            access2.add_write(ComponentId::of::<Velocity>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert_eq!(conflict.component_count(), 1);

            let conflicting: Vec<_> = conflict.conflicting_components().collect();
            assert_eq!(conflicting[0], ComponentId::of::<Position>());
        }

        #[test]
        fn test_get_conflicts_read_vs_write() {
            let mut access1 = Access::new();
            access1.add_read(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_write(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let comp_conflict = &conflict.component_conflicts()[0];
            assert_eq!(comp_conflict.first_access, AccessType::Read);
            assert_eq!(comp_conflict.second_access, AccessType::Write);
            assert!(comp_conflict.is_read_write());
        }
    }

    // =========================================================================
    // ConflictInfo Tests
    // =========================================================================

    mod conflict_info {
        use super::*;

        #[test]
        fn test_conflict_info_new() {
            let info = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );

            assert_eq!(info.component_id, ComponentId::of::<Position>());
            assert_eq!(info.first_access, AccessType::Write);
            assert_eq!(info.second_access, AccessType::Read);
        }

        #[test]
        fn test_conflict_info_is_write_write() {
            let ww = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Write,
            );
            assert!(ww.is_write_write());

            let wr = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );
            assert!(!wr.is_write_write());
        }

        #[test]
        fn test_conflict_info_display() {
            let info = ConflictInfo::new(
                ComponentId::of::<Position>(),
                AccessType::Write,
                AccessType::Read,
            );
            let display = format!("{}", info);
            assert!(display.contains("Component"));
            assert!(display.contains("Write"));
            assert!(display.contains("Read"));
        }
    }

    // =========================================================================
    // AccessConflict Tests
    // =========================================================================

    mod access_conflict_struct {
        use super::*;

        #[test]
        fn test_access_conflict_new() {
            let conflict = AccessConflict::new();
            assert!(conflict.is_empty());
            assert_eq!(conflict.component_count(), 0);
            assert_eq!(conflict.resource_count(), 0);
            assert_eq!(conflict.non_send_count(), 0);
            assert_eq!(conflict.total_count(), 0);
        }

        #[test]
        fn test_access_conflict_default() {
            let conflict: AccessConflict = Default::default();
            assert!(conflict.is_empty());
        }

        #[test]
        fn test_access_conflict_display() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let display = format!("{}", conflict);

            assert!(display.contains("AccessConflict"));
            assert!(display.contains("Component"));
        }

        #[test]
        fn test_access_conflict_has_write_write() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_write(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            assert!(conflict.has_write_write());
        }

        #[test]
        fn test_access_conflict_clone() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let cloned = conflict.clone();

            assert_eq!(conflict.total_count(), cloned.total_count());
        }

        #[test]
        fn test_access_conflict_conflicting_components_iter() {
            let mut access1 = Access::new();
            access1.add_write(ComponentId::of::<Position>());
            access1.add_write(ComponentId::of::<Velocity>());

            let mut access2 = Access::new();
            access2.add_read(ComponentId::of::<Position>());
            access2.add_read(ComponentId::of::<Velocity>());

            let conflict = access1.get_conflicts(&access2).unwrap();
            let components: Vec<_> = conflict.conflicting_components().collect();

            assert_eq!(components.len(), 2);
            assert!(components.contains(&ComponentId::of::<Position>()));
            assert!(components.contains(&ComponentId::of::<Velocity>()));
        }
    }
}
