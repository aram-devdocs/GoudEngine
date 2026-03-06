//! Tests for the FFI error bridge: thread-local storage and GoudFFIResult.

use std::sync::{Arc, Barrier};
use std::thread;

use crate::core::error::{
    clear_last_error, get_last_error, last_error_code, last_error_message, set_last_error,
    take_last_error, GoudError, GoudFFIResult, GoudResult,
};
use crate::core::error::{
    ERR_ENTITY_NOT_FOUND, ERR_INITIALIZATION_FAILED, ERR_INVALID_HANDLE, ERR_NOT_INITIALIZED,
    ERR_RESOURCE_NOT_FOUND, ERR_SHADER_COMPILATION_FAILED, SUCCESS,
};

/// Helper to ensure clean state for each test.
fn with_clean_error_state<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    clear_last_error();
    let result = f();
    clear_last_error();
    result
}

#[test]
fn test_set_and_get_last_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);
        let error = get_last_error();
        assert!(error.is_some());
        assert_eq!(error.unwrap(), GoudError::NotInitialized);
    });
}

#[test]
fn test_get_does_not_clear_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);

        let error1 = get_last_error();
        let error2 = get_last_error();
        let error3 = get_last_error();

        assert_eq!(error1, error2);
        assert_eq!(error2, error3);
        assert!(error1.is_some());
    });
}

#[test]
fn test_take_clears_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);

        let error1 = take_last_error();
        assert!(error1.is_some());
        assert_eq!(error1.unwrap(), GoudError::NotInitialized);

        let error2 = take_last_error();
        assert!(error2.is_none());

        let error3 = take_last_error();
        assert!(error3.is_none());
    });
}

#[test]
fn test_last_error_code_no_error() {
    with_clean_error_state(|| {
        assert_eq!(last_error_code(), SUCCESS);
    });
}

#[test]
fn test_last_error_code_with_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);
        assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

        set_last_error(GoudError::ResourceNotFound("test".to_string()));
        assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);

        set_last_error(GoudError::ShaderCompilationFailed("error".to_string()));
        assert_eq!(last_error_code(), ERR_SHADER_COMPILATION_FAILED);
    });
}

#[test]
fn test_last_error_message_no_error() {
    with_clean_error_state(|| {
        assert!(last_error_message().is_none());
    });
}

#[test]
fn test_last_error_message_with_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);
        let msg = last_error_message();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap(), "Engine has not been initialized");

        set_last_error(GoudError::InitializationFailed("GPU not found".to_string()));
        let msg = last_error_message();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap(), "GPU not found");
    });
}

#[test]
fn test_clear_last_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);
        assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

        clear_last_error();

        assert_eq!(last_error_code(), SUCCESS);
        assert!(last_error_message().is_none());
        assert!(get_last_error().is_none());
    });
}

#[test]
fn test_overwrite_error() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);
        assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

        set_last_error(GoudError::ResourceNotFound("file.txt".to_string()));
        assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);

        let error = take_last_error();
        assert!(matches!(error, Some(GoudError::ResourceNotFound(_))));
    });
}

#[test]
fn test_thread_isolation() {
    let barrier = Arc::new(Barrier::new(2));
    let barrier_clone = Arc::clone(&barrier);

    let handle = thread::spawn(move || {
        clear_last_error();
        barrier_clone.wait();

        assert_eq!(
            last_error_code(),
            SUCCESS,
            "Thread should have no error from main thread"
        );

        set_last_error(GoudError::ResourceNotFound("thread_file.txt".to_string()));
        assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);

        barrier_clone.wait();
    });

    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);
        assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);

        barrier.wait();
        barrier.wait();

        assert_eq!(
            last_error_code(),
            ERR_NOT_INITIALIZED,
            "Main thread error should not be affected by spawned thread"
        );
    });

    handle.join().unwrap();
}

#[test]
fn test_multiple_threads_independent_errors() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    const THREAD_COUNT: usize = 4;
    let success_count = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..THREAD_COUNT)
        .map(|i| {
            let success_count = Arc::clone(&success_count);
            thread::spawn(move || {
                clear_last_error();

                let error = match i {
                    0 => GoudError::NotInitialized,
                    1 => GoudError::AlreadyInitialized,
                    2 => GoudError::InvalidContext,
                    _ => GoudError::ContextDestroyed,
                };
                let expected_code = error.error_code();
                set_last_error(error);

                if last_error_code() == expected_code {
                    success_count.fetch_add(1, Ordering::SeqCst);
                }

                clear_last_error();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(
        success_count.load(std::sync::atomic::Ordering::SeqCst),
        THREAD_COUNT,
        "All threads should have set and verified their own errors"
    );
}

// =============================================================================
// GoudFFIResult Tests
// =============================================================================

#[test]
fn test_ffi_result_success() {
    let result = GoudFFIResult::success();
    assert!(result.success);
    assert!(result.is_success());
    assert!(!result.is_error());
    assert_eq!(result.code, SUCCESS);
}

#[test]
fn test_ffi_result_from_code_success() {
    let result = GoudFFIResult::from_code(SUCCESS);
    assert!(result.success);
    assert_eq!(result.code, SUCCESS);
}

#[test]
fn test_ffi_result_from_code_error() {
    let result = GoudFFIResult::from_code(ERR_NOT_INITIALIZED);
    assert!(!result.success);
    assert!(result.is_error());
    assert!(!result.is_success());
    assert_eq!(result.code, ERR_NOT_INITIALIZED);
}

#[test]
fn test_ffi_result_from_error() {
    with_clean_error_state(|| {
        let result = GoudFFIResult::from_error(GoudError::NotInitialized);
        assert!(!result.success);
        assert_eq!(result.code, ERR_NOT_INITIALIZED);
        assert_eq!(last_error_code(), ERR_NOT_INITIALIZED);
    });
}

#[test]
fn test_ffi_result_from_error_with_message() {
    with_clean_error_state(|| {
        let error = GoudError::InitializationFailed("Custom error message".to_string());
        let result = GoudFFIResult::from_error(error);

        assert!(!result.success);
        assert_eq!(result.code, ERR_INITIALIZATION_FAILED);

        let msg = last_error_message();
        assert_eq!(msg, Some("Custom error message".to_string()));
    });
}

#[test]
fn test_ffi_result_from_result_ok() {
    with_clean_error_state(|| {
        set_last_error(GoudError::NotInitialized);

        let result: GoudResult<i32> = Ok(42);
        let ffi_result = GoudFFIResult::from_result(result);

        assert!(ffi_result.success);
        assert_eq!(ffi_result.code, SUCCESS);
        assert_eq!(last_error_code(), SUCCESS);
    });
}

#[test]
fn test_ffi_result_from_result_err() {
    with_clean_error_state(|| {
        let result: GoudResult<i32> = Err(GoudError::ResourceNotFound("test.png".to_string()));
        let ffi_result = GoudFFIResult::from_result(result);

        assert!(!ffi_result.success);
        assert_eq!(ffi_result.code, ERR_RESOURCE_NOT_FOUND);
        assert_eq!(last_error_code(), ERR_RESOURCE_NOT_FOUND);
        assert_eq!(last_error_message(), Some("test.png".to_string()));
    });
}

#[test]
fn test_ffi_result_default() {
    let result = GoudFFIResult::default();
    assert!(result.success);
    assert_eq!(result.code, SUCCESS);
}

#[test]
fn test_ffi_result_from_goud_error() {
    with_clean_error_state(|| {
        let ffi_result: GoudFFIResult = GoudError::EntityNotFound.into();
        assert!(!ffi_result.success);
        assert_eq!(ffi_result.code, ERR_ENTITY_NOT_FOUND);
    });
}

#[test]
fn test_ffi_result_from_goud_result() {
    with_clean_error_state(|| {
        let ok: GoudResult<String> = Ok("hello".to_string());
        let ffi_result: GoudFFIResult = ok.into();
        assert!(ffi_result.success);

        let err: GoudResult<String> = Err(GoudError::InvalidHandle);
        let ffi_result: GoudFFIResult = err.into();
        assert!(!ffi_result.success);
        assert_eq!(ffi_result.code, ERR_INVALID_HANDLE);
    });
}

#[test]
fn test_ffi_result_derive_traits() {
    let result1 = GoudFFIResult::from_code(ERR_NOT_INITIALIZED);
    let result2 = result1;
    assert_eq!(result1, result2);

    let debug_str = format!("{:?}", result1);
    assert!(debug_str.contains("GoudFFIResult"));
    assert!(debug_str.contains("code"));
    assert!(debug_str.contains("success"));

    assert_eq!(GoudFFIResult::success(), GoudFFIResult::success());
    assert_ne!(
        GoudFFIResult::success(),
        GoudFFIResult::from_code(ERR_NOT_INITIALIZED)
    );
}

#[test]
fn test_ffi_result_repr_c() {
    use std::mem::{align_of, size_of};

    let size = size_of::<GoudFFIResult>();
    assert!(size >= 5, "GoudFFIResult should be at least 5 bytes");
    assert!(size <= 8, "GoudFFIResult should be at most 8 bytes");

    let align = align_of::<GoudFFIResult>();
    assert!(
        align >= 4,
        "GoudFFIResult should have at least 4-byte alignment"
    );
}

#[test]
fn test_ffi_workflow_simulation() {
    with_clean_error_state(|| {
        fn rust_ffi_function() -> GoudFFIResult {
            let result: GoudResult<()> = Err(GoudError::ShaderCompilationFailed(
                "ERROR: 0:15: 'vec3' : undeclared identifier".to_string(),
            ));
            GoudFFIResult::from_result(result)
        }

        let ffi_result = rust_ffi_function();

        assert!(!ffi_result.success);
        assert_eq!(ffi_result.code, ERR_SHADER_COMPILATION_FAILED);

        let code = last_error_code();
        let message = last_error_message();

        assert_eq!(code, ERR_SHADER_COMPILATION_FAILED);
        assert!(message.is_some());
        assert!(message.unwrap().contains("vec3"));

        clear_last_error();

        assert_eq!(last_error_code(), SUCCESS);
        assert!(last_error_message().is_none());
    });
}

#[test]
fn test_ffi_success_workflow_simulation() {
    with_clean_error_state(|| {
        fn rust_ffi_function() -> GoudFFIResult {
            let result: GoudResult<()> = Ok(());
            GoudFFIResult::from_result(result)
        }

        let ffi_result = rust_ffi_function();

        assert!(ffi_result.success);
        assert_eq!(ffi_result.code, SUCCESS);
        assert_eq!(last_error_code(), SUCCESS);
        assert!(last_error_message().is_none());
    });
}
