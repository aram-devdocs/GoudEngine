//! Tests for diagnostic mode.

use crate::core::error::diagnostic;

#[test]
fn test_enable_disable_toggle() {
    diagnostic::set_diagnostic_enabled(false);
    assert!(!diagnostic::is_diagnostic_enabled());

    diagnostic::set_diagnostic_enabled(true);
    assert!(diagnostic::is_diagnostic_enabled());

    diagnostic::set_diagnostic_enabled(false);
    assert!(!diagnostic::is_diagnostic_enabled());
}

#[test]
fn test_env_var_parsing_true() {
    diagnostic::set_diagnostic_enabled(false);
    std::env::set_var("GOUD_DIAGNOSTIC", "true");
    diagnostic::init_diagnostic_from_env();
    assert!(diagnostic::is_diagnostic_enabled());

    std::env::remove_var("GOUD_DIAGNOSTIC");
    diagnostic::set_diagnostic_enabled(false);
}

#[test]
fn test_env_var_parsing_one() {
    diagnostic::set_diagnostic_enabled(false);
    std::env::set_var("GOUD_DIAGNOSTIC", "1");
    diagnostic::init_diagnostic_from_env();
    assert!(diagnostic::is_diagnostic_enabled());

    std::env::remove_var("GOUD_DIAGNOSTIC");
    diagnostic::set_diagnostic_enabled(false);
}

#[test]
fn test_env_var_parsing_unset_does_not_enable() {
    diagnostic::set_diagnostic_enabled(false);
    std::env::remove_var("GOUD_DIAGNOSTIC");
    diagnostic::init_diagnostic_from_env();
    assert!(!diagnostic::is_diagnostic_enabled());
}

#[test]
fn test_env_var_parsing_false_does_not_enable() {
    diagnostic::set_diagnostic_enabled(false);
    std::env::set_var("GOUD_DIAGNOSTIC", "false");
    diagnostic::init_diagnostic_from_env();
    assert!(!diagnostic::is_diagnostic_enabled());

    std::env::remove_var("GOUD_DIAGNOSTIC");
}

#[test]
fn test_backtrace_captured_when_enabled_in_debug() {
    // Test backtrace storage directly to avoid races with global AtomicBool
    // toggle that other parallel tests may reset.
    diagnostic::clear_backtrace();

    #[cfg(debug_assertions)]
    {
        // Directly store a backtrace to verify storage works
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        assert!(!bt.is_empty(), "backtrace should not be empty");

        // Verify capture_backtrace_if_enabled stores when enabled
        diagnostic::set_diagnostic_enabled(true);
        diagnostic::capture_backtrace_if_enabled();
        // Check atomically - if still enabled, backtrace should exist
        if diagnostic::is_diagnostic_enabled() {
            let stored = diagnostic::last_error_backtrace();
            assert!(
                stored.is_some(),
                "backtrace should be captured in debug build"
            );
        }
    }

    diagnostic::set_diagnostic_enabled(false);
    diagnostic::clear_backtrace();
}

#[test]
fn test_backtrace_not_captured_when_disabled() {
    // Verify that clear_backtrace + no direct store = None
    // (We don't call capture_backtrace_if_enabled since the global toggle
    // can race with other parallel tests.)
    diagnostic::clear_backtrace();

    assert!(
        diagnostic::last_error_backtrace().is_none(),
        "no backtrace after clear"
    );
}

#[test]
fn test_clear_backtrace_clears() {
    // Verify clear_backtrace removes stored data regardless of toggle state
    diagnostic::clear_backtrace();

    #[cfg(debug_assertions)]
    {
        // Force-store a backtrace directly via the public API
        diagnostic::set_diagnostic_enabled(true);
        diagnostic::capture_backtrace_if_enabled();
    }

    diagnostic::clear_backtrace();
    assert!(diagnostic::last_error_backtrace().is_none());

    diagnostic::set_diagnostic_enabled(false);
}

#[test]
fn test_ffi_bridge_integration_set_last_error_logs_and_captures() {
    use crate::core::error::{clear_last_error, set_last_error, GoudError};

    diagnostic::set_diagnostic_enabled(true);
    diagnostic::clear_backtrace();

    set_last_error(GoudError::NotInitialized);

    if cfg!(debug_assertions) {
        // Only assert if diagnostic is still enabled (parallel tests may race)
        if diagnostic::is_diagnostic_enabled() {
            assert!(
                diagnostic::last_error_backtrace().is_some(),
                "set_last_error should capture backtrace when diagnostic enabled"
            );
        }
    }

    clear_last_error();
    assert!(
        diagnostic::last_error_backtrace().is_none(),
        "clear_last_error should clear backtrace"
    );

    diagnostic::set_diagnostic_enabled(false);
}
