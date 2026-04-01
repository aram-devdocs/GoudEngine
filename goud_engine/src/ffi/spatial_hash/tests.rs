//! Unit tests for spatial hash FFI functions.

use crate::core::error::ERR_INVALID_STATE;

use super::*;

/// Helper: creates a hash, runs the test, then destroys it.
fn with_hash(cell_size: f32, f: impl FnOnce(u32)) {
    let handle = goud_spatial_hash_create(cell_size);
    assert_ne!(handle, GOUD_INVALID_SPATIAL_HASH_HANDLE);
    f(handle);
    let rc = goud_spatial_hash_destroy(handle);
    assert_eq!(rc, 0);
}

#[test]
fn test_create_and_destroy() {
    with_hash(64.0, |_| {});
}

#[test]
fn test_create_with_capacity() {
    let handle = goud_spatial_hash_create_with_capacity(32.0, 1000);
    assert_ne!(handle, GOUD_INVALID_SPATIAL_HASH_HANDLE);
    assert_eq!(goud_spatial_hash_entity_count(handle), 0);
    assert_eq!(goud_spatial_hash_destroy(handle), 0);
}

#[test]
fn test_create_invalid_cell_size() {
    assert_eq!(
        goud_spatial_hash_create(0.0),
        GOUD_INVALID_SPATIAL_HASH_HANDLE
    );
    assert_eq!(
        goud_spatial_hash_create(-1.0),
        GOUD_INVALID_SPATIAL_HASH_HANDLE
    );
    assert_eq!(
        goud_spatial_hash_create(f32::NAN),
        GOUD_INVALID_SPATIAL_HASH_HANDLE
    );
    assert_eq!(
        goud_spatial_hash_create(f32::INFINITY),
        GOUD_INVALID_SPATIAL_HASH_HANDLE
    );
}

#[test]
fn test_destroy_invalid_handle() {
    let rc = goud_spatial_hash_destroy(GOUD_INVALID_SPATIAL_HASH_HANDLE);
    assert_eq!(rc, ERR_INVALID_STATE);
}

#[test]
fn test_destroy_nonexistent_handle() {
    let rc = goud_spatial_hash_destroy(999_999);
    assert_eq!(rc, ERR_INVALID_STATE);
}

#[test]
fn test_insert_and_count() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_entity_count(h), 0);
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_entity_count(h), 1);
        assert_eq!(goud_spatial_hash_insert(h, 2, 20.0, 20.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_entity_count(h), 2);
    });
}

#[test]
fn test_insert_overwrites() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 1, 50.0, 50.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_entity_count(h), 1);
    });
}

#[test]
fn test_remove() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 2, 20.0, 20.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_remove(h, 1), 0);
        assert_eq!(goud_spatial_hash_entity_count(h), 1);
    });
}

#[test]
fn test_remove_nonexistent_is_idempotent() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_remove(h, 999), 0);
    });
}

#[test]
fn test_clear() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 2, 20.0, 20.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_clear(h), 0);
        assert_eq!(goud_spatial_hash_entity_count(h), 0);
    });
}

#[test]
fn test_update() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_update(h, 1, 50.0, 50.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_entity_count(h), 1);
    });
}

#[test]
fn test_update_nonexistent_fails() {
    with_hash(64.0, |h| {
        let rc = goud_spatial_hash_update(h, 999, 10.0, 10.0, 5.0, 5.0);
        assert_eq!(rc, ERR_INVALID_STATE);
    });
}

#[test]
fn test_query_range() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 2, 15.0, 15.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 3, 1000.0, 1000.0, 5.0, 5.0), 0);

        let mut buf = [0u64; 16];
        // SAFETY: buf is a valid buffer of 16 u64 values.
        let count =
            unsafe { goud_spatial_hash_query_range(h, 12.0, 12.0, 50.0, buf.as_mut_ptr(), 16) };

        assert!(count >= 2, "Expected at least 2 results, got {count}");
    });
}

#[test]
fn test_query_rect() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 2, 15.0, 15.0, 5.0, 5.0), 0);
        assert_eq!(goud_spatial_hash_insert(h, 3, 1000.0, 1000.0, 5.0, 5.0), 0);

        let mut buf = [0u64; 16];
        // SAFETY: buf is a valid buffer of 16 u64 values.
        let count =
            unsafe { goud_spatial_hash_query_rect(h, 0.0, 0.0, 30.0, 30.0, buf.as_mut_ptr(), 16) };

        assert!(count >= 2, "Expected at least 2 results, got {count}");
    });
}

#[test]
fn test_query_buffer_overflow_returns_total() {
    with_hash(64.0, |h| {
        for i in 0..10u64 {
            assert_eq!(goud_spatial_hash_insert(h, i + 1, 10.0, 10.0, 5.0, 5.0), 0);
        }

        let mut buf = [0u64; 2];
        // SAFETY: buf is a valid buffer of 2 u64 values.
        let count =
            unsafe { goud_spatial_hash_query_range(h, 10.0, 10.0, 50.0, buf.as_mut_ptr(), 2) };

        assert!(count >= 10, "Expected total >= 10, got {count}");
        // Verify buffer was written to
        assert!(buf.iter().any(|&v| v != 0), "Buffer should have entries");
    });
}

#[test]
fn test_query_null_buffer_zero_capacity() {
    with_hash(64.0, |h| {
        assert_eq!(goud_spatial_hash_insert(h, 1, 10.0, 10.0, 5.0, 5.0), 0);

        // SAFETY: null buffer with 0 capacity is explicitly allowed.
        let count =
            unsafe { goud_spatial_hash_query_range(h, 10.0, 10.0, 50.0, std::ptr::null_mut(), 0) };
        assert!(count >= 1);
    });
}

#[test]
fn test_query_null_buffer_nonzero_capacity_errors() {
    with_hash(64.0, |h| {
        // SAFETY: Testing error path — null buffer with nonzero capacity.
        let count =
            unsafe { goud_spatial_hash_query_range(h, 10.0, 10.0, 50.0, std::ptr::null_mut(), 10) };
        assert_eq!(count, ERR_INVALID_STATE);
    });
}

#[test]
fn test_invalid_handle_errors() {
    assert_ne!(
        goud_spatial_hash_entity_count(GOUD_INVALID_SPATIAL_HASH_HANDLE),
        0
    );
    assert_ne!(
        goud_spatial_hash_insert(GOUD_INVALID_SPATIAL_HASH_HANDLE, 1, 0.0, 0.0, 1.0, 1.0),
        0
    );
    assert_ne!(
        goud_spatial_hash_remove(GOUD_INVALID_SPATIAL_HASH_HANDLE, 1),
        0
    );
    assert_ne!(goud_spatial_hash_clear(GOUD_INVALID_SPATIAL_HASH_HANDLE), 0);
    assert_ne!(
        goud_spatial_hash_update(GOUD_INVALID_SPATIAL_HASH_HANDLE, 1, 0.0, 0.0, 1.0, 1.0),
        0
    );
}
