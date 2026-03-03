//! Tests for SystemSet, SystemSetConfig, ChainedSystems, chain(), and
//! LabeledOrderingConstraint.

use crate::ecs::schedule::*;
use crate::ecs::system::SystemId;

// ========================================================================
// SystemSet Tests
// ========================================================================

mod system_set {
    use super::*;

    #[test]
    fn test_new() {
        let set = SystemSet::new("PhysicsSet");
        assert_eq!(set.name(), "PhysicsSet");
        assert!(set.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let set = SystemSet::with_capacity("Test", 10);
        assert_eq!(set.name(), "Test");
        assert!(set.is_empty());
    }

    #[test]
    fn test_add() {
        let mut set = SystemSet::new("Test");
        let id = SystemId::from_raw(1);
        assert!(set.add(id));
        assert_eq!(set.len(), 1);
        assert!(set.contains(id));
    }

    #[test]
    fn test_add_duplicate() {
        let mut set = SystemSet::new("Test");
        let id = SystemId::from_raw(1);
        assert!(set.add(id));
        assert!(!set.add(id));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut set = SystemSet::new("Test");
        let id = SystemId::from_raw(1);
        set.add(id);
        assert!(set.remove(id));
        assert!(!set.contains(id));
        assert!(set.is_empty());
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut set = SystemSet::new("Test");
        assert!(!set.remove(SystemId::from_raw(1)));
    }

    #[test]
    fn test_contains() {
        let mut set = SystemSet::new("Test");
        let id1 = SystemId::from_raw(1);
        let id2 = SystemId::from_raw(2);
        set.add(id1);
        assert!(set.contains(id1));
        assert!(!set.contains(id2));
    }

    #[test]
    fn test_len() {
        let mut set = SystemSet::new("Test");
        assert_eq!(set.len(), 0);
        set.add(SystemId::from_raw(1));
        assert_eq!(set.len(), 1);
        set.add(SystemId::from_raw(2));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_is_empty() {
        let mut set = SystemSet::new("Test");
        assert!(set.is_empty());
        set.add(SystemId::from_raw(1));
        assert!(!set.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut set = SystemSet::new("Test");
        set.add(SystemId::from_raw(1));
        set.add(SystemId::from_raw(2));
        let ids: Vec<_> = set.iter().collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut set = SystemSet::new("Test");
        set.add(SystemId::from_raw(1));
        set.add(SystemId::from_raw(2));
        set.clear();
        assert!(set.is_empty());
    }

    #[test]
    fn test_default() {
        let set = SystemSet::default();
        assert_eq!(set.name(), "DefaultSet");
    }

    #[test]
    fn test_debug() {
        let set = SystemSet::new("Test");
        let debug = format!("{:?}", set);
        assert!(debug.contains("Test"));
    }

    #[test]
    fn test_clone() {
        let mut set = SystemSet::new("Test");
        set.add(SystemId::from_raw(1));
        let cloned = set.clone();
        assert_eq!(cloned.len(), 1);
        assert!(cloned.contains(SystemId::from_raw(1)));
    }
}

// ========================================================================
// SystemSetConfig Tests
// ========================================================================

mod system_set_config {
    use super::*;

    #[test]
    fn test_new() {
        let config = SystemSetConfig::new();
        assert!(config.enabled);
        assert!(config.before_labels.is_empty());
        assert!(config.after_labels.is_empty());
    }

    #[test]
    fn test_default() {
        let config = SystemSetConfig::default();
        assert!(config.enabled);
    }

    #[test]
    fn test_before() {
        let config = SystemSetConfig::new().before(CoreSystemLabel::Physics);
        assert_eq!(config.before_labels.len(), 1);
        assert_eq!(config.before_labels[0].name(), "Physics");
    }

    #[test]
    fn test_after() {
        let config = SystemSetConfig::new().after(CoreSystemLabel::Input);
        assert_eq!(config.after_labels.len(), 1);
        assert_eq!(config.after_labels[0].name(), "Input");
    }

    #[test]
    fn test_enabled() {
        let config = SystemSetConfig::new().enabled(false);
        assert!(!config.enabled);
    }

    #[test]
    fn test_chained_config() {
        let config = SystemSetConfig::new()
            .before(CoreSystemLabel::Physics)
            .after(CoreSystemLabel::Input)
            .enabled(true);
        assert_eq!(config.before_labels.len(), 1);
        assert_eq!(config.after_labels.len(), 1);
        assert!(config.enabled);
    }

    #[test]
    fn test_clone() {
        let config = SystemSetConfig::new().before(CoreSystemLabel::Physics);
        let cloned = config.clone();
        assert_eq!(cloned.before_labels.len(), 1);
    }
}

// ========================================================================
// ChainedSystems Tests
// ========================================================================

mod chained_systems {
    use super::*;

    #[test]
    fn test_new() {
        let chain = ChainedSystems::new("TestChain");
        assert_eq!(chain.name(), "TestChain");
        assert!(chain.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let chain = ChainedSystems::with_capacity("Test", 10);
        assert_eq!(chain.name(), "Test");
        assert!(chain.is_empty());
    }

    #[test]
    fn test_add() {
        let mut chain = ChainedSystems::new("Test");
        chain.add(SystemId::from_raw(1));
        chain.add(SystemId::from_raw(2));
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_add_after() {
        let mut chain = ChainedSystems::new("Test");
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        chain.add(a);
        chain.add(c);
        chain.add_after(b, a);
        let ids: Vec<_> = chain.iter().collect();
        assert_eq!(ids, vec![a, b, c]);
    }

    #[test]
    fn test_add_after_not_found() {
        let mut chain = ChainedSystems::new("Test");
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        chain.add(a);
        chain.add_after(b, c);
        let ids: Vec<_> = chain.iter().collect();
        assert_eq!(ids, vec![a, b]);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut chain = ChainedSystems::new("Test");
        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());
        chain.add(SystemId::from_raw(1));
        assert_eq!(chain.len(), 1);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut chain = ChainedSystems::new("Test");
        chain.add(SystemId::from_raw(1));
        chain.add(SystemId::from_raw(2));
        let ids: Vec<_> = chain.iter().collect();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_to_orderings_empty() {
        let chain = ChainedSystems::new("Test");
        assert!(chain.to_orderings().is_empty());
    }

    #[test]
    fn test_to_orderings_single() {
        let mut chain = ChainedSystems::new("Test");
        chain.add(SystemId::from_raw(1));
        assert!(chain.to_orderings().is_empty());
    }

    #[test]
    fn test_to_orderings_multiple() {
        let mut chain = ChainedSystems::new("Test");
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        chain.add(a);
        chain.add(b);
        chain.add(c);
        let orderings = chain.to_orderings();
        assert_eq!(orderings.len(), 2);
        assert_eq!(orderings[0].first(), a);
        assert_eq!(orderings[0].second(), b);
        assert_eq!(orderings[1].first(), b);
        assert_eq!(orderings[1].second(), c);
    }

    #[test]
    fn test_clear() {
        let mut chain = ChainedSystems::new("Test");
        chain.add(SystemId::from_raw(1));
        chain.clear();
        assert!(chain.is_empty());
    }

    #[test]
    fn test_default() {
        let chain = ChainedSystems::default();
        assert_eq!(chain.name(), "Chain");
    }

    #[test]
    fn test_clone() {
        let mut chain = ChainedSystems::new("Test");
        chain.add(SystemId::from_raw(1));
        let cloned = chain.clone();
        assert_eq!(cloned.len(), 1);
    }
}

// ========================================================================
// chain Function Tests
// ========================================================================

mod chain_function {
    use super::*;

    #[test]
    fn test_chain_empty() {
        let orderings = chain([]);
        assert!(orderings.is_empty());
    }

    #[test]
    fn test_chain_single() {
        let orderings = chain([SystemId::from_raw(1)]);
        assert!(orderings.is_empty());
    }

    #[test]
    fn test_chain_two() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let orderings = chain([a, b]);
        assert_eq!(orderings.len(), 1);
        assert_eq!(orderings[0].first(), a);
        assert_eq!(orderings[0].second(), b);
    }

    #[test]
    fn test_chain_three() {
        let a = SystemId::from_raw(1);
        let b = SystemId::from_raw(2);
        let c = SystemId::from_raw(3);
        let orderings = chain([a, b, c]);
        assert_eq!(orderings.len(), 2);
    }

    #[test]
    fn test_chain_from_vec() {
        let ids = vec![
            SystemId::from_raw(1),
            SystemId::from_raw(2),
            SystemId::from_raw(3),
        ];
        let orderings = chain(ids);
        assert_eq!(orderings.len(), 2);
    }
}

// ========================================================================
// LabeledOrderingConstraint Tests
// ========================================================================

mod labeled_ordering_constraint {
    use super::*;

    #[test]
    fn test_before_label() {
        let c = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
        assert!(c.is_label_based());
        assert!(!c.is_system_based());
    }

    #[test]
    fn test_after_label() {
        let c = LabeledOrderingConstraint::after_label(CoreSystemLabel::Input);
        assert!(c.is_label_based());
    }

    #[test]
    fn test_before_system() {
        let c = LabeledOrderingConstraint::before_system(SystemId::from_raw(1));
        assert!(c.is_system_based());
        assert!(!c.is_label_based());
    }

    #[test]
    fn test_after_system() {
        let c = LabeledOrderingConstraint::after_system(SystemId::from_raw(1));
        assert!(c.is_system_based());
    }

    #[test]
    fn test_display_before_label() {
        let c = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
        let display = format!("{}", c);
        assert!(display.contains("before label"));
        assert!(display.contains("Physics"));
    }

    #[test]
    fn test_display_after_label() {
        let c = LabeledOrderingConstraint::after_label(CoreSystemLabel::Input);
        let display = format!("{}", c);
        assert!(display.contains("after label"));
        assert!(display.contains("Input"));
    }

    #[test]
    fn test_display_before_system() {
        let c = LabeledOrderingConstraint::before_system(SystemId::from_raw(42));
        let display = format!("{}", c);
        assert!(display.contains("before system"));
        assert!(display.contains("42"));
    }

    #[test]
    fn test_display_after_system() {
        let c = LabeledOrderingConstraint::after_system(SystemId::from_raw(99));
        let display = format!("{}", c);
        assert!(display.contains("after system"));
        assert!(display.contains("99"));
    }

    #[test]
    fn test_clone() {
        let c = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
        let cloned = c.clone();
        assert!(cloned.is_label_based());
    }

    #[test]
    fn test_debug() {
        let c = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
        let debug = format!("{:?}", c);
        assert!(debug.contains("BeforeLabel"));
    }
}
