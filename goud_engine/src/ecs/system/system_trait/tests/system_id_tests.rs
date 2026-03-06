//! Tests for [`SystemId`].

use crate::ecs::system::SystemId;

#[test]
fn test_new_creates_unique_ids() {
    let id1 = SystemId::new();
    let id2 = SystemId::new();
    let id3 = SystemId::new();

    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_invalid_constant() {
    let invalid = SystemId::INVALID;
    assert!(invalid.is_invalid());
    assert!(!invalid.is_valid());
    assert_eq!(invalid.raw(), 0);
}

#[test]
fn test_valid_ids() {
    let id = SystemId::new();
    assert!(id.is_valid());
    assert!(!id.is_invalid());
}

#[test]
fn test_from_raw() {
    let id = SystemId::from_raw(42);
    assert_eq!(id.raw(), 42);
    assert!(id.is_valid());
}

#[test]
fn test_from_raw_zero_is_invalid() {
    let id = SystemId::from_raw(0);
    assert!(id.is_invalid());
}

#[test]
fn test_default_is_invalid() {
    let id = SystemId::default();
    assert!(id.is_invalid());
}

#[test]
fn test_equality() {
    let id1 = SystemId::from_raw(10);
    let id2 = SystemId::from_raw(10);
    let id3 = SystemId::from_raw(20);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_ordering() {
    let id1 = SystemId::from_raw(10);
    let id2 = SystemId::from_raw(20);

    assert!(id1 < id2);
    assert!(id2 > id1);
}

#[test]
fn test_hash() {
    use std::collections::HashSet;

    let id1 = SystemId::from_raw(10);
    let id2 = SystemId::from_raw(20);
    let id3 = SystemId::from_raw(10);

    let mut set = HashSet::new();
    set.insert(id1);
    set.insert(id2);
    set.insert(id3);

    assert_eq!(set.len(), 2); // id1 and id3 are the same
}

#[test]
fn test_debug_format_invalid() {
    let id = SystemId::INVALID;
    let debug = format!("{:?}", id);
    assert!(debug.contains("INVALID"));
}

#[test]
fn test_debug_format_valid() {
    let id = SystemId::from_raw(42);
    let debug = format!("{:?}", id);
    assert!(debug.contains("42"));
}

#[test]
fn test_display_format() {
    let id = SystemId::from_raw(42);
    let display = format!("{}", id);
    assert_eq!(display, "42");

    let invalid = SystemId::INVALID;
    let display_invalid = format!("{}", invalid);
    assert_eq!(display_invalid, "INVALID");
}

#[test]
fn test_copy_and_clone() {
    let id1 = SystemId::new();
    let id2 = id1; // Copy
    let id3 = id1.clone(); // Clone

    assert_eq!(id1, id2);
    assert_eq!(id1, id3);
}

#[test]
fn test_thread_safety() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SystemId>();
}
