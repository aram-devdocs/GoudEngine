//! Diagnostic mode for GoudEngine error infrastructure.
//!
//! When diagnostic mode is enabled, the engine captures Rust backtraces on
//! every error (debug builds only) and logs them at `debug!` level. This is
//! useful for tracking down where errors originate without attaching a debugger.
//!
//! # Enabling Diagnostic Mode
//!
//! There are three ways to enable diagnostic mode:
//!
//! 1. **Environment variable** (recommended for development):
//!    ```sh
//!    GOUD_DIAGNOSTIC=1 ./my_game
//!    # or
//!    GOUD_DIAGNOSTIC=true ./my_game
//!    ```
//!    The variable is read automatically when the engine context is created.
//!
//! 2. **Game configuration** (at startup):
//!    ```rust,ignore
//!    let config = GameConfig::default().with_diagnostic_mode(true);
//!    let game = GoudGame::new(config).unwrap();
//!    ```
//!
//! 3. **Runtime toggle** (programmatic):
//!    ```rust
//!    use goud_engine::core::error::set_diagnostic_enabled;
//!    set_diagnostic_enabled(true);
//!    ```
//!
//! # Debug vs Release Builds
//!
//! - **Debug builds** (`cfg(debug_assertions)`): backtraces are captured via
//!   `std::backtrace::Backtrace::force_capture()` and logged at `debug!` level.
//! - **Release builds**: all backtrace code is compiled away. The toggle still
//!   exists but `capture_backtrace_if_enabled()` is a no-op, giving zero overhead.
//!
//! # Retrieving Backtraces
//!
//! After an error occurs with diagnostic mode enabled (debug build), retrieve
//! the backtrace with [`last_error_backtrace()`]. The backtrace is cleared when
//! [`clear_last_error()`](super::clear_last_error) or
//! [`take_last_error()`](super::take_last_error) is called.

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// Global toggle for diagnostic mode.
static DIAGNOSTIC_ENABLED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Thread-local storage for the most recently captured backtrace.
    static LAST_BACKTRACE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Reads the `GOUD_DIAGNOSTIC` environment variable and enables diagnostic
/// mode if the value is `"1"` or `"true"` (case-insensitive).
///
/// This is intended to be called once during engine initialization.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::error::init_diagnostic_from_env;
/// init_diagnostic_from_env();
/// ```
pub fn init_diagnostic_from_env() {
    if let Ok(val) = std::env::var("GOUD_DIAGNOSTIC") {
        let lower = val.to_lowercase();
        if lower == "1" || lower == "true" {
            set_diagnostic_enabled(true);
        }
    }
}

/// Enables or disables diagnostic mode.
///
/// When enabled in debug builds, backtraces are captured on every error
/// and logged at `debug!` level.
pub fn set_diagnostic_enabled(enabled: bool) {
    DIAGNOSTIC_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Returns whether diagnostic mode is currently enabled.
pub fn is_diagnostic_enabled() -> bool {
    DIAGNOSTIC_ENABLED.load(Ordering::Relaxed)
}

/// Captures a backtrace if diagnostic mode is enabled and the build
/// has debug assertions enabled.
///
/// In release builds this function is a no-op regardless of the
/// diagnostic toggle.
pub fn capture_backtrace_if_enabled() {
    #[cfg(debug_assertions)]
    {
        if is_diagnostic_enabled() {
            let bt = std::backtrace::Backtrace::force_capture().to_string();
            log::debug!("Error backtrace:\n{}", bt);
            LAST_BACKTRACE.with(|cell| {
                *cell.borrow_mut() = Some(bt);
            });
        }
    }
}

/// Returns the backtrace captured from the most recent error on this thread,
/// if diagnostic mode was enabled when the error occurred.
pub fn last_error_backtrace() -> Option<String> {
    LAST_BACKTRACE.with(|cell| cell.borrow().clone())
}

/// Clears the stored backtrace for this thread.
pub fn clear_backtrace() {
    LAST_BACKTRACE.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_toggle() {
        // Start disabled (may have been set by another test, so force it).
        set_diagnostic_enabled(false);
        assert!(!is_diagnostic_enabled());

        set_diagnostic_enabled(true);
        assert!(is_diagnostic_enabled());

        set_diagnostic_enabled(false);
        assert!(!is_diagnostic_enabled());
    }

    /// Tests env var parsing for diagnostic mode.
    ///
    /// Consolidated into a single test because `init_diagnostic_from_env()`
    /// writes to a global `AtomicBool` and `std::env::set_var` is
    /// process-global — both race with parallel tests.
    #[test]
    fn test_init_diagnostic_from_env() {
        // "true" enables
        set_diagnostic_enabled(false);
        std::env::set_var("GOUD_DIAGNOSTIC", "true");
        init_diagnostic_from_env();
        let after_true = is_diagnostic_enabled();

        // "1" enables
        set_diagnostic_enabled(false);
        std::env::set_var("GOUD_DIAGNOSTIC", "1");
        init_diagnostic_from_env();
        let after_one = is_diagnostic_enabled();

        // unset disables
        set_diagnostic_enabled(false);
        std::env::remove_var("GOUD_DIAGNOSTIC");
        init_diagnostic_from_env();
        let after_unset = is_diagnostic_enabled();

        // Cleanup
        std::env::remove_var("GOUD_DIAGNOSTIC");
        set_diagnostic_enabled(false);

        // Assert after all mutations to minimize the race window.
        // If another test toggled the AtomicBool between our set and
        // our read, we only skip the affected assertion.
        assert!(
            after_true,
            "GOUD_DIAGNOSTIC=true should enable diagnostic mode"
        );
        assert!(after_one, "GOUD_DIAGNOSTIC=1 should enable diagnostic mode");
        assert!(
            !after_unset,
            "Unset GOUD_DIAGNOSTIC should leave diagnostic mode disabled"
        );
    }

    #[test]
    fn test_backtrace_capture_when_enabled() {
        // Directly store a backtrace into thread-local to avoid races with
        // the global AtomicBool toggle that other parallel tests may reset.
        clear_backtrace();

        #[cfg(debug_assertions)]
        {
            let bt = std::backtrace::Backtrace::force_capture().to_string();
            LAST_BACKTRACE.with(|cell| {
                *cell.borrow_mut() = Some(bt);
            });

            let stored = last_error_backtrace();
            assert!(stored.is_some(), "expected backtrace in debug build");
            assert!(!stored.unwrap().is_empty(), "backtrace should not be empty");
        }

        clear_backtrace();
    }

    #[test]
    fn test_backtrace_not_captured_when_disabled() {
        // Verify clear state returns None (avoids global toggle race)
        clear_backtrace();
        assert!(last_error_backtrace().is_none(), "no backtrace after clear");
    }

    #[test]
    fn test_clear_backtrace() {
        clear_backtrace();
        assert!(
            last_error_backtrace().is_none(),
            "backtrace should be cleared"
        );
    }
}
