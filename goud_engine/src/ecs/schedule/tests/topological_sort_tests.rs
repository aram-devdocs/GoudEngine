//! Tests for OrderingCycleError and TopologicalSorter.

use crate::ecs::schedule::*;
use crate::ecs::system::SystemId;

// ========================================================================
// OrderingCycleError Tests
// ========================================================================

mod cycle_error {
    use super::*;

    #[test]
    fn test_cycle_error_new() {
        let ids = vec![
            SystemId::from_raw(1),
            SystemId::from_raw(2),
            SystemId::from_raw(3),
        ];
        let names = vec!["SystemA", "SystemB", "SystemC"];
        let err = OrderingCycleError::new(ids.clone(), names.clone());
        assert_eq!(err.cycle.len(), 3);
        assert_eq!(err.names.len(), 3);
    }

    #[test]
    fn test_cycle_error_describe() {
        let ids = vec![SystemId::from_raw(1), SystemId::from_raw(2)];
        let names = vec!["A", "B"];
        let err = OrderingCycleError::new(ids, names);
        let desc = err.describe();
        assert!(desc.contains("A"));
        assert!(desc.contains("B"));
        assert!(desc.contains("->"));
    }

    #[test]
    fn test_cycle_error_display() {
        let ids = vec![SystemId::from_raw(1)];
        let names = vec!["Test"];
        let err = OrderingCycleError::new(ids, names);
        let display = format!("{}", err);
        assert!(display.contains("cycle"));
    }

    #[test]
    fn test_cycle_error_empty() {
        let err = OrderingCycleError::new(Vec::new(), Vec::new());
        let desc = err.describe();
        assert!(desc.contains("Empty"));
    }
}

// ========================================================================
// TopologicalSorter Tests
// ========================================================================

mod sorter {
    use super::*;

    #[test]
    fn test_sorter_new() {
        let sorter = TopologicalSorter::new();
        assert!(sorter.is_empty());
        assert_eq!(sorter.system_count(), 0);
        assert_eq!(sorter.edge_count(), 0);
    }

    #[test]
    fn test_sorter_with_capacity() {
        let sorter = TopologicalSorter::with_capacity(10, 20);
        assert!(sorter.is_empty());
    }

    #[test]
    fn test_sorter_add_system() {
        let mut sorter = TopologicalSorter::new();
        let id = SystemId::from_raw(1);
        sorter.add_system(id, "TestSystem");
        assert_eq!(sorter.system_count(), 1);
        assert!(!sorter.is_empty());
    }

    #[test]
    fn test_sorter_add_system_duplicate() {
        let mut sorter = TopologicalSorter::new();
        let id = SystemId::from_raw(1);
        sorter.add_system(id, "TestSystem");
        sorter.add_system(id, "TestSystem");
        assert_eq!(sorter.system_count(), 1);
    }

    #[test]
    fn test_sorter_add_ordering() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_ordering(a, b);
        assert_eq!(sorter.edge_count(), 1);
    }

    #[test]
    fn test_sorter_add_ordering_missing_system() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        sorter.add_system(a, "A");
        sorter.add_ordering(a, b);
        assert_eq!(sorter.edge_count(), 0);
    }

    #[test]
    fn test_sorter_add_ordering_self() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        sorter.add_system(a, "A");
        sorter.add_ordering(a, a);
        assert_eq!(sorter.edge_count(), 0);
    }

    #[test]
    fn test_sorter_add_ordering_duplicate() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_ordering(a, b);
        sorter.add_ordering(a, b);
        assert_eq!(sorter.edge_count(), 1);
    }

    #[test]
    fn test_sorter_clear() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_ordering(a, b);
        sorter.clear();
        assert!(sorter.is_empty());
        assert_eq!(sorter.edge_count(), 0);
    }

    #[test]
    fn test_sorter_sort_empty() {
        let sorter = TopologicalSorter::new();
        let result = sorter.sort();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_sorter_sort_single() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        sorter.add_system(a, "A");
        let result = sorter.sort().unwrap();
        assert_eq!(result, vec![a]);
    }

    #[test]
    fn test_sorter_sort_no_constraints() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        let result = sorter.sort().unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.contains(&a));
        assert!(result.contains(&b));
    }

    #[test]
    fn test_sorter_sort_linear_chain() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_system(c, "C");
        sorter.add_ordering(a, b);
        sorter.add_ordering(b, c);
        let result = sorter.sort().unwrap();
        assert_eq!(result, vec![a, b, c]);
    }

    #[test]
    fn test_sorter_sort_diamond() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        let d = SystemId::from_raw(4);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_system(c, "C");
        sorter.add_system(d, "D");
        sorter.add_ordering(a, b);
        sorter.add_ordering(a, c);
        sorter.add_ordering(b, d);
        sorter.add_ordering(c, d);
        let result = sorter.sort().unwrap();
        assert_eq!(result[0], a);
        assert_eq!(result[3], d);
        let middle: Vec<_> = result[1..3].to_vec();
        assert!(middle.contains(&b));
        assert!(middle.contains(&c));
    }

    #[test]
    fn test_sorter_sort_cycle_detection() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_system(c, "C");
        sorter.add_ordering(a, b);
        sorter.add_ordering(b, c);
        sorter.add_ordering(c, a);
        let result = sorter.sort();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.cycle.is_empty());
    }

    #[test]
    fn test_sorter_sort_self_cycle() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        sorter.add_system(a, "A");
        sorter.add_ordering(a, a);
        assert!(sorter.sort().is_ok());
    }

    #[test]
    fn test_sorter_would_cycle() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        sorter.add_system(a, "A");
        sorter.add_system(b, "B");
        sorter.add_ordering(a, b);
        sorter.add_ordering(b, a);
        assert!(sorter.would_cycle());
    }

    #[test]
    fn test_sorter_clone() {
        let mut sorter = TopologicalSorter::new();
        let a = SystemId::from_raw(1);
        sorter.add_system(a, "A");
        let cloned = sorter.clone();
        assert_eq!(cloned.system_count(), sorter.system_count());
    }
}

// ========================================================================
// Stress Tests
// ========================================================================

mod stress {
    use super::*;

    #[test]
    fn test_sorter_many_systems() {
        let mut sorter = TopologicalSorter::new();
        for i in 0..100 {
            sorter.add_system(SystemId::from_raw(i), "System");
        }
        for i in 0..99 {
            sorter.add_ordering(SystemId::from_raw(i), SystemId::from_raw(i + 1));
        }
        let result = sorter.sort().unwrap();
        assert_eq!(result.len(), 100);
        for i in 0..100 {
            assert_eq!(result[i as usize], SystemId::from_raw(i));
        }
    }

    #[test]
    fn test_sorter_complex_dag() {
        let mut sorter = TopologicalSorter::new();
        for i in 0..10 {
            sorter.add_system(SystemId::from_raw(i), "System");
        }
        let edges = [
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 4),
            (2, 4),
            (3, 5),
            (4, 6),
            (5, 6),
            (6, 7),
            (6, 8),
            (6, 9),
        ];
        for (from, to) in edges {
            sorter.add_ordering(SystemId::from_raw(from), SystemId::from_raw(to));
        }
        let result = sorter.sort().unwrap();
        assert_eq!(result.len(), 10);
        for (from, to) in edges {
            let from_pos = result
                .iter()
                .position(|id| *id == SystemId::from_raw(from))
                .unwrap();
            let to_pos = result
                .iter()
                .position(|id| *id == SystemId::from_raw(to))
                .unwrap();
            assert!(from_pos < to_pos, "Edge {}->{} violated", from, to);
        }
    }
}
