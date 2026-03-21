//! Bulk component query operation tests.

use super::{
    goud_component_add, goud_component_remove, register_test_type, setup_test_context,
    TestComponent, TEST_TYPE_ID,
};
use crate::ffi::component::query::{
    goud_component_count, goud_component_get_all, goud_component_get_entities,
};
use crate::ffi::entity::goud_entity_spawn_empty;
use crate::ffi::{GoudEntityId, GOUD_INVALID_CONTEXT_ID};

/// Unique type-ID offset so parallel tests never collide with other suites.
const QID: u64 = TEST_TYPE_ID + 100;

// ============================================================================
// goud_component_count
// ============================================================================

#[test]
fn test_component_count_empty() {
    let ctx = setup_test_context();
    register_test_type(QID);

    assert_eq!(goud_component_count(ctx, QID), 0);
}

#[test]
fn test_component_count_after_add() {
    let ctx = setup_test_context();
    register_test_type(QID + 1);

    for i in 0..5u32 {
        // SAFETY: ctx is a valid context from setup_test_context.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        let comp = TestComponent {
            x: i as f32,
            y: i as f32,
        };
        // SAFETY: comp is a valid stack-allocated TestComponent; size matches the registered type.
        let result = unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 1,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            )
        };
        assert!(result.is_ok(), "add should succeed for entity {}", i);
    }

    assert_eq!(goud_component_count(ctx, QID + 1), 5);
}

#[test]
fn test_component_count_after_remove() {
    let ctx = setup_test_context();
    register_test_type(QID + 2);

    let mut entities = Vec::new();
    for i in 0..3u32 {
        // SAFETY: ctx is a valid context.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        entities.push(bits);
        let comp = TestComponent {
            x: i as f32,
            y: 0.0,
        };
        // SAFETY: comp is valid, size matches.
        unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 2,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }
    }

    // Remove the first entity's component.
    // SAFETY: entities[0] is a valid entity ID from goud_entity_spawn_empty.
    let rem = unsafe { goud_component_remove(ctx, GoudEntityId::new(entities[0]), QID + 2) };
    assert!(rem.is_ok(), "remove should succeed");

    assert_eq!(goud_component_count(ctx, QID + 2), 2);
}

#[test]
fn test_component_count_unregistered() {
    let ctx = setup_test_context();
    // 0xDEAD is never registered
    assert_eq!(goud_component_count(ctx, 0xDEAD), 0);
}

#[test]
fn test_component_count_invalid_context() {
    register_test_type(QID + 3);
    assert_eq!(goud_component_count(GOUD_INVALID_CONTEXT_ID, QID + 3), 0);
}

// ============================================================================
// goud_component_get_entities
// ============================================================================

#[test]
fn test_get_entities_basic() {
    let ctx = setup_test_context();
    register_test_type(QID + 4);

    let mut spawned = Vec::new();
    for i in 0..3u32 {
        // SAFETY: ctx is valid.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        spawned.push(bits);
        let comp = TestComponent {
            x: i as f32,
            y: 0.0,
        };
        // SAFETY: comp is valid, size matches.
        unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 4,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }
    }

    let mut buf = [0u64; 16];
    // SAFETY: buf is a valid stack-allocated buffer with capacity 16.
    let n = unsafe { goud_component_get_entities(ctx, QID + 4, buf.as_mut_ptr(), 16) };
    assert_eq!(n, 3);

    // All spawned entity bits should appear in the output (order may differ).
    let returned: std::collections::HashSet<u64> = buf[..n as usize].iter().copied().collect();
    for bits in &spawned {
        assert!(returned.contains(bits), "missing entity bits {}", bits);
    }
}

#[test]
fn test_get_entities_max_count_clamp() {
    let ctx = setup_test_context();
    register_test_type(QID + 5);

    for _ in 0..5 {
        // SAFETY: ctx is valid.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        let comp = TestComponent { x: 0.0, y: 0.0 };
        // SAFETY: comp is valid, size matches.
        unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 5,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }
    }

    let mut buf = [0u64; 3];
    // SAFETY: buf has capacity 3.
    let n = unsafe { goud_component_get_entities(ctx, QID + 5, buf.as_mut_ptr(), 3) };
    assert_eq!(n, 3);
}

#[test]
fn test_get_entities_null_pointer() {
    let ctx = setup_test_context();
    register_test_type(QID + 6);

    // SAFETY: passing null intentionally to test the null check.
    let n = unsafe { goud_component_get_entities(ctx, QID + 6, std::ptr::null_mut(), 10) };
    assert_eq!(n, 0);
}

// ============================================================================
// goud_component_get_all
// ============================================================================

#[test]
fn test_get_all_basic() {
    let ctx = setup_test_context();
    register_test_type(QID + 7);

    for i in 0..3u32 {
        // SAFETY: ctx is valid.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        let comp = TestComponent {
            x: i as f32,
            y: i as f32 * 10.0,
        };
        // SAFETY: comp is valid, size matches.
        unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 7,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }
    }

    let mut ents = [0u64; 16];
    let mut ptrs: [*const u8; 16] = [std::ptr::null(); 16];
    // SAFETY: ents and ptrs are valid stack-allocated buffers with capacity 16.
    let n =
        unsafe { goud_component_get_all(ctx, QID + 7, ents.as_mut_ptr(), ptrs.as_mut_ptr(), 16) };
    assert_eq!(n, 3);

    for i in 0..n as usize {
        assert!(!ptrs[i].is_null(), "data ptr {} should be non-null", i);
    }
}

#[test]
fn test_get_all_data_integrity() {
    let ctx = setup_test_context();
    register_test_type(QID + 8);

    let values: [(f32, f32); 3] = [(1.0, 2.0), (3.0, 4.0), (5.0, 6.0)];
    let mut spawned = Vec::new();

    for &(x, y) in &values {
        // SAFETY: ctx is valid.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        spawned.push(bits);
        let comp = TestComponent { x, y };
        // SAFETY: comp is valid, size matches.
        unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 8,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }
    }

    let mut ents = [0u64; 16];
    let mut ptrs: [*const u8; 16] = [std::ptr::null(); 16];
    // SAFETY: ents and ptrs are valid buffers.
    let n =
        unsafe { goud_component_get_all(ctx, QID + 8, ents.as_mut_ptr(), ptrs.as_mut_ptr(), 16) };
    assert_eq!(n, 3);

    // Build a map from entity bits -> TestComponent for validation.
    let mut result_map = std::collections::HashMap::new();
    for i in 0..n as usize {
        // SAFETY: ptrs[i] was obtained from dense_data and points to a valid
        // TestComponent allocation with correct size and alignment.
        let comp = unsafe { &*(ptrs[i] as *const TestComponent) };
        result_map.insert(ents[i], (comp.x, comp.y));
    }

    for (idx, &bits) in spawned.iter().enumerate() {
        let (x, y) = result_map
            .get(&bits)
            .unwrap_or_else(|| panic!("entity {} not found in results", bits));
        assert_eq!(*x, values[idx].0, "x mismatch for entity {}", bits);
        assert_eq!(*y, values[idx].1, "y mismatch for entity {}", bits);
    }
}

#[test]
fn test_get_all_after_remove() {
    let ctx = setup_test_context();
    register_test_type(QID + 9);

    let mut spawned = Vec::new();
    for i in 0..5u32 {
        // SAFETY: ctx is valid.
        let bits = unsafe { goud_entity_spawn_empty(ctx) };
        spawned.push(bits);
        let comp = TestComponent {
            x: i as f32,
            y: 0.0,
        };
        // SAFETY: comp is valid, size matches.
        unsafe {
            goud_component_add(
                ctx,
                GoudEntityId::new(bits),
                QID + 9,
                &comp as *const _ as *const u8,
                std::mem::size_of::<TestComponent>(),
            );
        }
    }

    // Remove entities at index 1 and 3.
    for &idx in &[1usize, 3] {
        // SAFETY: spawned[idx] is a valid entity from goud_entity_spawn_empty.
        let r = unsafe { goud_component_remove(ctx, GoudEntityId::new(spawned[idx]), QID + 9) };
        assert!(r.is_ok(), "remove should succeed");
    }

    let mut ents = [0u64; 16];
    let mut ptrs: [*const u8; 16] = [std::ptr::null(); 16];
    // SAFETY: ents and ptrs are valid buffers.
    let n =
        unsafe { goud_component_get_all(ctx, QID + 9, ents.as_mut_ptr(), ptrs.as_mut_ptr(), 16) };
    assert_eq!(n, 3);

    let returned: std::collections::HashSet<u64> = ents[..n as usize].iter().copied().collect();
    // Removed entities should not appear.
    assert!(!returned.contains(&spawned[1]));
    assert!(!returned.contains(&spawned[3]));
    // Remaining entities should appear.
    for &idx in &[0usize, 2, 4] {
        assert!(
            returned.contains(&spawned[idx]),
            "entity {} should remain",
            spawned[idx]
        );
    }
}
