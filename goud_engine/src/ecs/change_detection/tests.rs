//! Unit tests for Tick and ComponentTicks.

use super::{ComponentTicks, Tick};

#[test]
fn tick_new_stores_value() {
    let t = Tick::new(42);
    assert_eq!(t.get(), 42);
}

#[test]
fn tick_default_is_zero() {
    let t = Tick::default();
    assert_eq!(t.get(), 0);
}

#[test]
fn tick_is_newer_than_compares_correctly() {
    let t1 = Tick::new(1);
    let t2 = Tick::new(5);

    assert!(t2.is_newer_than(t1));
    assert!(!t1.is_newer_than(t2));
    assert!(!t1.is_newer_than(t1));
}

#[test]
fn tick_from_u32_round_trip() {
    let value: u32 = 99;
    let tick = Tick::from(value);
    let back: u32 = tick.into();
    assert_eq!(back, value);
}

#[test]
fn tick_ordering() {
    let t1 = Tick::new(1);
    let t2 = Tick::new(2);
    let t3 = Tick::new(2);

    assert!(t1 < t2);
    assert_eq!(t2, t3);
    assert!(t2 >= t1);
}

#[test]
fn component_ticks_new_sets_both() {
    let ticks = ComponentTicks::new(Tick::new(7));
    assert_eq!(ticks.added(), Tick::new(7));
    assert_eq!(ticks.changed(), Tick::new(7));
}

#[test]
fn component_ticks_set_changed_updates_only_changed() {
    let mut ticks = ComponentTicks::new(Tick::new(3));
    ticks.set_changed(Tick::new(10));

    assert_eq!(ticks.added(), Tick::new(3));
    assert_eq!(ticks.changed(), Tick::new(10));
}

#[test]
fn component_ticks_is_added_checks_against_threshold() {
    let ticks = ComponentTicks::new(Tick::new(5));

    assert!(ticks.is_added(Tick::new(3)));
    assert!(!ticks.is_added(Tick::new(5)));
    assert!(!ticks.is_added(Tick::new(7)));
}

#[test]
fn component_ticks_is_changed_checks_against_threshold() {
    let mut ticks = ComponentTicks::new(Tick::new(2));
    ticks.set_changed(Tick::new(8));

    assert!(ticks.is_changed(Tick::new(5)));
    assert!(!ticks.is_changed(Tick::new(8)));
    assert!(!ticks.is_changed(Tick::new(10)));
}
