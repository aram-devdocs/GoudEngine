use crate::ecs::schedule::named_system_sets::{DefaultSystemSet, NamedSystemSets, SetNameLabel};
use crate::ecs::schedule::stage::Stage;
use crate::ecs::schedule::{SystemSetConfig, SystemStage};
use crate::ecs::system::{System, SystemId};
use crate::ecs::World;
use std::sync::{Arc, Mutex};

// =========================================================================
// NamedSystemSets unit tests
// =========================================================================

#[test]
fn test_register_and_add_to_set() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("Physics");

    let id_a = SystemId::from_raw(1);
    let id_b = SystemId::from_raw(2);

    assert!(sets.add_to_set("Physics", id_a));
    assert!(sets.add_to_set("Physics", id_b));
    // Duplicate returns false
    assert!(!sets.add_to_set("Physics", id_a));

    let set = sets.get_set("Physics").unwrap();
    assert_eq!(set.len(), 2);
    assert!(set.contains(id_a));
    assert!(set.contains(id_b));
}

#[test]
#[should_panic(expected = "has not been registered")]
fn test_add_to_unregistered_set_panics() {
    let mut sets = NamedSystemSets::new();
    sets.add_to_set("Nonexistent", SystemId::from_raw(1));
}

#[test]
fn test_configure_set_before() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("A");
    sets.register_set("B");

    let a1 = SystemId::from_raw(1);
    let a2 = SystemId::from_raw(2);
    let b1 = SystemId::from_raw(3);

    sets.add_to_set("A", a1);
    sets.add_to_set("A", a2);
    sets.add_to_set("B", b1);

    // A before B
    sets.configure_set("A", SystemSetConfig::new().before(SetNameLabel("B")));

    let orderings = sets.resolve_orderings();
    // Should have 2 orderings: a1 before b1, a2 before b1
    assert_eq!(orderings.len(), 2);

    let edges: Vec<(SystemId, SystemId)> = orderings.iter().map(|o| o.as_edge()).collect();
    assert!(edges.contains(&(a1, b1)));
    assert!(edges.contains(&(a2, b1)));
}

#[test]
fn test_configure_set_after() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("A");
    sets.register_set("B");

    let a1 = SystemId::from_raw(1);
    let b1 = SystemId::from_raw(2);
    let b2 = SystemId::from_raw(3);

    sets.add_to_set("A", a1);
    sets.add_to_set("B", b1);
    sets.add_to_set("B", b2);

    // A after B: every system in A runs after every system in B
    sets.configure_set("A", SystemSetConfig::new().after(SetNameLabel("B")));

    let orderings = sets.resolve_orderings();
    assert_eq!(orderings.len(), 2);

    // "after" means b runs first, then a
    let edges: Vec<(SystemId, SystemId)> = orderings.iter().map(|o| o.as_edge()).collect();
    assert!(edges.contains(&(b1, a1)));
    assert!(edges.contains(&(b2, a1)));
}

#[test]
fn test_empty_set_ordering_is_noop() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("A");
    sets.register_set("B");

    // A before B but A is empty
    sets.configure_set("A", SystemSetConfig::new().before(SetNameLabel("B")));
    sets.add_to_set("B", SystemId::from_raw(1));

    let orderings = sets.resolve_orderings();
    assert!(orderings.is_empty());
}

#[test]
fn test_default_system_set_variants() {
    let all = DefaultSystemSet::all();
    assert_eq!(all.len(), 6);

    assert_eq!(DefaultSystemSet::PreUpdate.as_str(), "PreUpdate");
    assert_eq!(DefaultSystemSet::Update.as_str(), "Update");
    assert_eq!(DefaultSystemSet::PostUpdate.as_str(), "PostUpdate");
    assert_eq!(DefaultSystemSet::PreRender.as_str(), "PreRender");
    assert_eq!(DefaultSystemSet::Render.as_str(), "Render");
    assert_eq!(DefaultSystemSet::PostRender.as_str(), "PostRender");

    // Display
    assert_eq!(format!("{}", DefaultSystemSet::Update), "Update");
}

#[test]
fn test_set_names() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("Alpha");
    sets.register_set("Beta");

    let mut names = sets.set_names();
    names.sort();
    assert_eq!(names, vec!["Alpha", "Beta"]);
}

#[test]
fn test_register_set_idempotent() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("X");
    sets.add_to_set("X", SystemId::from_raw(1));

    // Re-registering should not clear the set
    sets.register_set("X");
    let set = sets.get_set("X").unwrap();
    assert_eq!(set.len(), 1);
}

#[test]
#[should_panic(expected = "has not been registered")]
fn test_configure_unregistered_set_panics() {
    let mut sets = NamedSystemSets::new();
    sets.configure_set("Missing", SystemSetConfig::new());
}

#[test]
fn test_get_config_returns_none_for_unconfigured() {
    let mut sets = NamedSystemSets::new();
    sets.register_set("X");
    assert!(sets.get_config("X").is_none());
}

#[test]
fn test_get_set_returns_none_for_unregistered() {
    let sets = NamedSystemSets::new();
    assert!(sets.get_set("NoSuchSet").is_none());
}

// =========================================================================
// Integration with SystemStage
// =========================================================================

#[test]
fn test_systems_in_set_run_in_correct_order() {
    let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

    struct LogSystem {
        label: &'static str,
        log: Arc<Mutex<Vec<&'static str>>>,
    }
    impl System for LogSystem {
        fn name(&self) -> &'static str {
            self.label
        }
        fn run(&mut self, _world: &mut World) {
            self.log.lock().unwrap().push(self.label);
        }
    }

    let mut stage = SystemStage::new("Test");

    // Add systems in reverse order to prove ordering works
    let id_b = stage.add_system(LogSystem {
        label: "B",
        log: Arc::clone(&log),
    });
    let id_a = stage.add_system(LogSystem {
        label: "A",
        log: Arc::clone(&log),
    });

    stage.register_set("First");
    stage.register_set("Second");
    stage.add_system_to_set("First", id_a);
    stage.add_system_to_set("Second", id_b);
    stage.configure_named_set(
        "First",
        SystemSetConfig::new().before(SetNameLabel("Second")),
    );

    let mut world = World::new();
    stage.run(&mut world);

    let order = log.lock().unwrap();
    assert_eq!(*order, vec!["A", "B"]);
}

#[test]
fn test_mixed_set_and_individual_ordering() {
    let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));

    struct LogSystem {
        label: &'static str,
        log: Arc<Mutex<Vec<&'static str>>>,
    }
    impl System for LogSystem {
        fn name(&self) -> &'static str {
            self.label
        }
        fn run(&mut self, _world: &mut World) {
            self.log.lock().unwrap().push(self.label);
        }
    }

    let mut stage = SystemStage::new("Test");

    let id_c = stage.add_system(LogSystem {
        label: "C",
        log: Arc::clone(&log),
    });
    let id_b = stage.add_system(LogSystem {
        label: "B",
        log: Arc::clone(&log),
    });
    let id_a = stage.add_system(LogSystem {
        label: "A",
        log: Arc::clone(&log),
    });

    // Set ordering: set "Early" (A) before set "Late" (C)
    stage.register_set("Early");
    stage.register_set("Late");
    stage.add_system_to_set("Early", id_a);
    stage.add_system_to_set("Late", id_c);
    stage.configure_named_set("Early", SystemSetConfig::new().before(SetNameLabel("Late")));

    // Individual ordering: B before C
    stage.add_ordering(id_b, id_c);

    let mut world = World::new();
    stage.run(&mut world);

    let order = log.lock().unwrap();
    // A must be before C (set ordering), B must be before C (individual)
    let pos_a = order.iter().position(|&x| x == "A").unwrap();
    let pos_b = order.iter().position(|&x| x == "B").unwrap();
    let pos_c = order.iter().position(|&x| x == "C").unwrap();
    assert!(pos_a < pos_c, "A should run before C");
    assert!(pos_b < pos_c, "B should run before C");
}

#[test]
fn test_cycle_detection_between_sets() {
    let mut stage = SystemStage::new("Test");

    struct Noop;
    impl System for Noop {
        fn name(&self) -> &'static str {
            "Noop"
        }
        fn run(&mut self, _world: &mut World) {}
    }

    let id_a = stage.add_system(Noop);
    let id_b = stage.add_system(Noop);

    stage.register_set("X");
    stage.register_set("Y");
    stage.add_system_to_set("X", id_a);
    stage.add_system_to_set("Y", id_b);

    // X before Y AND Y before X => cycle
    stage.configure_named_set("X", SystemSetConfig::new().before(SetNameLabel("Y")));
    stage.configure_named_set("Y", SystemSetConfig::new().before(SetNameLabel("X")));

    // Running should trigger cycle detection (logged as warning, not panic)
    let mut world = World::new();
    stage.run(&mut world); // Should not panic but will warn about cycle
}
