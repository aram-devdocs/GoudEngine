//! Tests for the [`Name`] component.

use crate::ecs::component::Component;
use crate::ecs::components::hierarchy::Name;

#[test]
fn test_name_new() {
    let name = Name::new("Player");
    assert_eq!(name.as_str(), "Player");
}

#[test]
fn test_name_new_string() {
    let name = Name::new(String::from("Enemy"));
    assert_eq!(name.as_str(), "Enemy");
}

#[test]
fn test_name_as_str() {
    let name = Name::new("Test");
    assert_eq!(name.as_str(), "Test");
}

#[test]
fn test_name_set() {
    let mut name = Name::new("Old");
    name.set("New");
    assert_eq!(name.as_str(), "New");
}

#[test]
fn test_name_set_string() {
    let mut name = Name::new("Old");
    name.set(String::from("New"));
    assert_eq!(name.as_str(), "New");
}

#[test]
fn test_name_len() {
    let name = Name::new("Test");
    assert_eq!(name.len(), 4);

    let empty = Name::new("");
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_name_is_empty() {
    let empty = Name::new("");
    assert!(empty.is_empty());

    let non_empty = Name::new("Player");
    assert!(!non_empty.is_empty());
}

#[test]
fn test_name_into_string() {
    let name = Name::new("Player");
    let s: String = name.into_string();
    assert_eq!(s, "Player");
}

#[test]
fn test_name_contains() {
    let name = Name::new("Player_01");
    assert!(name.contains("Player"));
    assert!(name.contains("01"));
    assert!(name.contains("_"));
    assert!(!name.contains("Enemy"));
}

#[test]
fn test_name_starts_with() {
    let name = Name::new("Player_01");
    assert!(name.starts_with("Player"));
    assert!(name.starts_with("Play"));
    assert!(!name.starts_with("Enemy"));
}

#[test]
fn test_name_ends_with() {
    let name = Name::new("Player_01");
    assert!(name.ends_with("01"));
    assert!(name.ends_with("_01"));
    assert!(!name.ends_with("Player"));
}

#[test]
fn test_name_default() {
    let name: Name = Default::default();
    assert!(name.is_empty());
    assert_eq!(name.as_str(), "");
}

#[test]
fn test_name_debug() {
    let name = Name::new("Test");
    let debug = format!("{:?}", name);
    assert!(debug.contains("Name"));
    assert!(debug.contains("Test"));
}

#[test]
fn test_name_display() {
    let name = Name::new("Player");
    let display = format!("{}", name);
    assert_eq!(display, "Player");
}

#[test]
fn test_name_from_str() {
    let name: Name = "Test".into();
    assert_eq!(name.as_str(), "Test");
}

#[test]
fn test_name_from_string() {
    let name: Name = String::from("Test").into();
    assert_eq!(name.as_str(), "Test");
}

#[test]
fn test_string_from_name() {
    let name = Name::new("Test");
    let s: String = name.into();
    assert_eq!(s, "Test");
}

#[test]
fn test_name_as_ref() {
    let name = Name::new("Test");
    let s: &str = name.as_ref();
    assert_eq!(s, "Test");
}

#[test]
fn test_name_borrow() {
    use std::borrow::Borrow;
    let name = Name::new("Test");
    let s: &str = name.borrow();
    assert_eq!(s, "Test");
}

#[test]
fn test_name_eq_str() {
    let name = Name::new("Test");
    assert!(name == *"Test");
    assert!(name == "Test");
    assert!(name != *"Other");
}

#[test]
fn test_name_eq_string() {
    let name = Name::new("Test");
    assert!(name == String::from("Test"));
    assert!(name != String::from("Other"));
}

#[test]
fn test_name_clone() {
    let name = Name::new("Test");
    let cloned = name.clone();
    assert_eq!(name, cloned);
}

#[test]
fn test_name_eq() {
    let n1 = Name::new("Test");
    let n2 = Name::new("Test");
    let n3 = Name::new("Other");

    assert_eq!(n1, n2);
    assert_ne!(n1, n3);
}

#[test]
fn test_name_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(Name::new("Player"));
    assert!(set.contains(&Name::new("Player")));
    assert!(!set.contains(&Name::new("Enemy")));
}

#[test]
fn test_name_is_component() {
    fn assert_component<T: Component>() {}
    assert_component::<Name>();
}

#[test]
fn test_name_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Name>();
}

#[test]
fn test_name_unicode() {
    let name = Name::new("玩家");
    assert_eq!(name.as_str(), "玩家");
    assert_eq!(name.len(), 6); // 2 characters, 3 bytes each in UTF-8
}

#[test]
fn test_name_emoji() {
    let name = Name::new("Player 🎮");
    assert!(name.contains("🎮"));
}
