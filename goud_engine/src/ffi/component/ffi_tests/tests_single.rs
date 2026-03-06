//! Single-entity component operation tests.

use super::{
    goud_component_add, goud_component_get, goud_component_get_mut, goud_component_has,
    goud_component_remove, register_test_type, setup_test_context, TestComponent, TEST_TYPE_ID,
};
use crate::ffi::{GoudEntityId, GOUD_INVALID_CONTEXT_ID};

// ============================================================================
// Component Add Tests
// ============================================================================

#[test]
fn test_component_add_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 2);

    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };
    assert_ne!(entity_id, crate::ffi::entity::GOUD_INVALID_ENTITY_ID);

    let component = TestComponent { x: 10.0, y: 20.0 };
    let result = unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            TEST_TYPE_ID + 2,
            &component as *const _ as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert!(result.is_ok(), "Component add should succeed");
}

#[test]
fn test_component_add_invalid_context() {
    register_test_type(TEST_TYPE_ID + 3);

    let component = TestComponent { x: 10.0, y: 20.0 };
    let result = unsafe {
        goud_component_add(
            GOUD_INVALID_CONTEXT_ID,
            GoudEntityId::new(0),
            TEST_TYPE_ID + 3,
            &component as *const _ as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert!(result.is_err(), "Add with invalid context should fail");
}

#[test]
fn test_component_add_unregistered_type() {
    let context_id = setup_test_context();
    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

    let component = TestComponent { x: 10.0, y: 20.0 };
    let result = unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            99999, // Unregistered type
            &component as *const _ as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert!(result.is_err(), "Add with unregistered type should fail");
}

#[test]
fn test_component_add_null_data() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 4);
    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

    let result = unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            TEST_TYPE_ID + 4,
            std::ptr::null(),
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert!(result.is_err(), "Add with null data pointer should fail");
}

#[test]
fn test_component_add_wrong_size() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 5);

    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };
    let component = TestComponent { x: 10.0, y: 20.0 };

    let result = unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            TEST_TYPE_ID + 5,
            &component as *const _ as *const u8,
            999, // Wrong size
        )
    };

    assert!(result.is_err(), "Add with wrong size should fail");
}

// ============================================================================
// Component Remove Tests
// ============================================================================

#[test]
fn test_component_remove_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 6);

    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

    let result = goud_component_remove(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 6);

    // Should succeed even if component doesn't exist
    assert!(result.is_ok());
}

// ============================================================================
// Component Has Tests
// ============================================================================

#[test]
fn test_component_has_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 7);

    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

    // Before adding component - should return false
    let has_component =
        goud_component_has(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 7);
    assert!(!has_component);

    // Add component
    let component = TestComponent { x: 1.0, y: 2.0 };
    unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            TEST_TYPE_ID + 7,
            &component as *const _ as *const u8,
            std::mem::size_of::<TestComponent>(),
        );
    }

    // After adding - should return true
    let has_component =
        goud_component_has(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 7);
    assert!(has_component);
}

// ============================================================================
// Component Get / Get Mut Tests
// ============================================================================

#[test]
fn test_component_get_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 8);

    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

    // Before adding - should return null
    let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 8);
    assert!(ptr.is_null());

    // Add component
    let component = TestComponent { x: 42.0, y: 99.0 };
    unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            TEST_TYPE_ID + 8,
            &component as *const _ as *const u8,
            std::mem::size_of::<TestComponent>(),
        );
    }

    // After adding - should return valid pointer with correct data
    let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 8);
    assert!(!ptr.is_null());

    // Read back the component data and verify
    let read_component = unsafe { *(ptr as *const TestComponent) };
    assert_eq!(read_component.x, 42.0);
    assert_eq!(read_component.y, 99.0);
}

#[test]
fn test_component_get_mut_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID + 9);

    let entity_id = unsafe { crate::ffi::entity::goud_entity_spawn_empty(context_id) };

    // Before adding - should return null
    let ptr = goud_component_get_mut(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);
    assert!(ptr.is_null());

    // Add component
    let component = TestComponent { x: 10.0, y: 20.0 };
    unsafe {
        goud_component_add(
            context_id,
            GoudEntityId::new(entity_id),
            TEST_TYPE_ID + 9,
            &component as *const _ as *const u8,
            std::mem::size_of::<TestComponent>(),
        );
    }

    // Get mutable pointer
    let ptr = goud_component_get_mut(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);
    assert!(!ptr.is_null());

    // Modify the component through the mutable pointer
    unsafe {
        let comp = &mut *(ptr as *mut TestComponent);
        comp.x = 100.0;
        comp.y = 200.0;
    }

    // Read back and verify changes
    let ptr = goud_component_get(context_id, GoudEntityId::new(entity_id), TEST_TYPE_ID + 9);
    let read_component = unsafe { *(ptr as *const TestComponent) };
    assert_eq!(read_component.x, 100.0);
    assert_eq!(read_component.y, 200.0);
}
