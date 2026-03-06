//! Batch component operation tests.

use super::{
    goud_component_add_batch, goud_component_has, goud_component_has_batch,
    goud_component_remove_batch, register_test_type, setup_test_context, TestComponent,
    TEST_TYPE_ID,
};
use crate::ffi::{GoudEntityId, GOUD_INVALID_CONTEXT_ID};

// ============================================================================
// Batch Add Tests
// ============================================================================

#[test]
fn test_component_add_batch_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID);

    let mut entities = [0u64; 5];
    // SAFETY: entities has capacity for 5 u64 values.
    unsafe {
        crate::ffi::entity::goud_entity_spawn_batch(context_id, 5, entities.as_mut_ptr());
    }

    let components = [
        TestComponent { x: 1.0, y: 2.0 },
        TestComponent { x: 3.0, y: 4.0 },
        TestComponent { x: 5.0, y: 6.0 },
        TestComponent { x: 7.0, y: 8.0 },
        TestComponent { x: 9.0, y: 10.0 },
    ];

    // SAFETY: entities and components are valid slices of length 5; size matches the registered type.
    let added = unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            5,
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert_eq!(added, 5);
}

#[test]
fn test_component_add_batch_invalid_context() {
    let entities = [1u64, 2, 3];
    let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

    // SAFETY: entities and components are valid slices; the function handles invalid context gracefully.
    let added = unsafe {
        goud_component_add_batch(
            GOUD_INVALID_CONTEXT_ID,
            entities.as_ptr(),
            3,
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert_eq!(added, 0);
}

#[test]
fn test_component_add_batch_null_entities() {
    let context_id = setup_test_context();
    let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

    // SAFETY: Passing null for entities tests that the function handles null gracefully.
    let added = unsafe {
        goud_component_add_batch(
            context_id,
            std::ptr::null(),
            3,
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert_eq!(added, 0);
}

#[test]
fn test_component_add_batch_null_data() {
    let context_id = setup_test_context();
    let entities = [1u64, 2, 3];

    // SAFETY: Passing null for data tests that the function handles null gracefully.
    let added = unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            3,
            TEST_TYPE_ID,
            std::ptr::null(),
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert_eq!(added, 0);
}

#[test]
fn test_component_add_batch_unregistered_type() {
    let context_id = setup_test_context();
    let entities = [1u64, 2, 3];
    let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

    // SAFETY: entities and components are valid slices; the function handles unregistered types gracefully.
    let added = unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            3,
            99999, // Unregistered type
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };

    assert_eq!(added, 0);
}

#[test]
fn test_component_add_batch_size_mismatch() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID);

    let entities = [1u64, 2, 3];
    let components = [TestComponent { x: 0.0, y: 0.0 }; 3];

    // SAFETY: entities and components are valid slices; the function handles size mismatch gracefully.
    let added = unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            3,
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            16, // Wrong size
        )
    };

    assert_eq!(added, 0);
}

// ============================================================================
// Batch Remove Tests
// ============================================================================

#[test]
fn test_component_remove_batch_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID);

    let mut entities = [0u64; 5];
    // SAFETY: entities has capacity for 5 u64 values.
    unsafe {
        crate::ffi::entity::goud_entity_spawn_batch(context_id, 5, entities.as_mut_ptr());
    }

    let components = [
        TestComponent { x: 1.0, y: 2.0 },
        TestComponent { x: 3.0, y: 4.0 },
        TestComponent { x: 5.0, y: 6.0 },
        TestComponent { x: 7.0, y: 8.0 },
        TestComponent { x: 9.0, y: 10.0 },
    ];

    // SAFETY: entities and components are valid slices of length 5; size matches the registered type.
    unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            5,
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        );
    }

    for &entity_bits in &entities {
        assert!(goud_component_has(
            context_id,
            GoudEntityId::new(entity_bits),
            TEST_TYPE_ID
        ));
    }

    // SAFETY: entities is a valid slice of 5 u64 values.
    let removed =
        unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 5, TEST_TYPE_ID) };

    assert_eq!(removed, 5);

    for &entity_bits in &entities {
        assert!(!goud_component_has(
            context_id,
            GoudEntityId::new(entity_bits),
            TEST_TYPE_ID
        ));
    }
}

#[test]
fn test_component_remove_batch_invalid_context() {
    let entities = [1u64, 2, 3];

    // SAFETY: entities is a valid slice; the function handles invalid context gracefully.
    let removed = unsafe {
        goud_component_remove_batch(GOUD_INVALID_CONTEXT_ID, entities.as_ptr(), 3, TEST_TYPE_ID)
    };

    assert_eq!(removed, 0);
}

#[test]
fn test_component_remove_batch_unregistered_type() {
    let context_id = setup_test_context();
    let entities = [1u64, 2, 3];

    // SAFETY: entities is a valid slice; the function handles unregistered types gracefully.
    let removed = unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 3, 99999) };

    assert_eq!(removed, 0);
}

// ============================================================================
// Batch Has Tests
// ============================================================================

#[test]
fn test_component_has_batch_basic() {
    let context_id = setup_test_context();
    register_test_type(TEST_TYPE_ID);

    let mut entities = [0u64; 5];
    // SAFETY: entities has capacity for 5 u64 values.
    unsafe {
        crate::ffi::entity::goud_entity_spawn_batch(context_id, 5, entities.as_mut_ptr());
    }

    let components = [
        TestComponent { x: 1.0, y: 2.0 },
        TestComponent { x: 3.0, y: 4.0 },
        TestComponent { x: 5.0, y: 6.0 },
    ];

    // SAFETY: entities and components are valid slices; size matches the registered type.
    unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            3, // Only first 3
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        );
    }

    let mut results = [0u8; 5];

    // SAFETY: entities and results are valid slices of length 5.
    let count = unsafe {
        goud_component_has_batch(
            context_id,
            entities.as_ptr(),
            5,
            TEST_TYPE_ID,
            results.as_mut_ptr(),
        )
    };

    assert_eq!(count, 5);
    assert_eq!(results[0], 1);
    assert_eq!(results[1], 1);
    assert_eq!(results[2], 1);
    assert_eq!(results[3], 0);
    assert_eq!(results[4], 0);
}

#[test]
fn test_component_has_batch_invalid_context() {
    let entities = [1u64, 2, 3];
    let mut results = [0u8; 3];

    // SAFETY: entities and results are valid slices; the function handles invalid context gracefully.
    let count = unsafe {
        goud_component_has_batch(
            GOUD_INVALID_CONTEXT_ID,
            entities.as_ptr(),
            3,
            TEST_TYPE_ID,
            results.as_mut_ptr(),
        )
    };

    assert_eq!(count, 0);
}

#[test]
fn test_component_has_batch_null_results() {
    let context_id = setup_test_context();
    let entities = [1u64, 2, 3];

    // SAFETY: Passing null for results tests that the function handles null gracefully.
    let count = unsafe {
        goud_component_has_batch(
            context_id,
            entities.as_ptr(),
            3,
            TEST_TYPE_ID,
            std::ptr::null_mut(),
        )
    };

    assert_eq!(count, 0);
}

#[test]
fn test_component_has_batch_unregistered_type() {
    let context_id = setup_test_context();
    let entities = [1u64, 2, 3];
    let mut results = [0u8; 3];

    // SAFETY: entities and results are valid slices; the function handles unregistered types gracefully.
    let count = unsafe {
        goud_component_has_batch(
            context_id,
            entities.as_ptr(),
            3,
            99999, // Unregistered type
            results.as_mut_ptr(),
        )
    };

    assert_eq!(count, 0);
}

// ============================================================================
// Zero-Count Edge Case
// ============================================================================

#[test]
fn test_component_batch_zero_count() {
    let context_id = setup_test_context();
    let entities = [1u64];
    let components = [TestComponent { x: 0.0, y: 0.0 }];
    let mut results = [0u8; 1];

    // SAFETY: entities and components are valid slices; count 0 means no elements are accessed.
    let added = unsafe {
        goud_component_add_batch(
            context_id,
            entities.as_ptr(),
            0,
            TEST_TYPE_ID,
            components.as_ptr() as *const u8,
            std::mem::size_of::<TestComponent>(),
        )
    };
    assert_eq!(added, 0);

    // SAFETY: entities is a valid slice; count 0 means no elements are accessed.
    let removed =
        unsafe { goud_component_remove_batch(context_id, entities.as_ptr(), 0, TEST_TYPE_ID) };
    assert_eq!(removed, 0);

    // SAFETY: entities and results are valid slices; count 0 means no elements are accessed.
    let count = unsafe {
        goud_component_has_batch(
            context_id,
            entities.as_ptr(),
            0,
            TEST_TYPE_ID,
            results.as_mut_ptr(),
        )
    };
    assert_eq!(count, 0);
}
