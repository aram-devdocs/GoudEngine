//! Application lifecycle events.
//!
//! Contains events emitted during the engine's startup and shutdown phases.

/// Emitted when the application/engine has fully initialized.
///
/// This event fires once after all core systems are ready but before the
/// first frame begins. Use this for one-time initialization that depends
/// on the engine being fully set up.
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::AppStarted;
/// use goud_engine::core::event::Events;
///
/// fn on_app_start(events: &Events<AppStarted>) {
///     let mut reader = events.reader();
///     for _ in reader.read() {
///         println!("Application started - performing one-time setup");
///         // Load initial assets, connect to services, etc.
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AppStarted;

/// Emitted when the application is about to exit.
///
/// This event fires before shutdown begins, giving systems a chance to
/// perform cleanup, save state, or release resources gracefully.
///
/// # Exit Reasons
///
/// The `reason` field indicates why the application is exiting:
/// - `User`: User requested exit (close button, quit command)
/// - `Error`: Unrecoverable error occurred
/// - `Programmatic`: Code explicitly requested shutdown
///
/// # Example
///
/// ```rust
/// use goud_engine::core::events::{AppExiting, ExitReason};
/// use goud_engine::core::event::Events;
///
/// fn on_app_exit(events: &Events<AppExiting>) {
///     let mut reader = events.reader();
///     for event in reader.read() {
///         match event.reason {
///             ExitReason::User => println!("User requested exit"),
///             ExitReason::Error => println!("Exiting due to error"),
///             ExitReason::Programmatic => println!("Programmatic shutdown"),
///         }
///         // Save game state, cleanup resources, etc.
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AppExiting {
    /// The reason the application is exiting.
    pub reason: ExitReason,
}

impl AppExiting {
    /// Creates an `AppExiting` event with the specified reason.
    #[must_use]
    pub fn new(reason: ExitReason) -> Self {
        Self { reason }
    }

    /// Creates an `AppExiting` event for user-initiated exit.
    #[must_use]
    pub fn user() -> Self {
        Self::new(ExitReason::User)
    }

    /// Creates an `AppExiting` event for error-initiated exit.
    #[must_use]
    pub fn error() -> Self {
        Self::new(ExitReason::Error)
    }

    /// Creates an `AppExiting` event for programmatic exit.
    #[must_use]
    pub fn programmatic() -> Self {
        Self::new(ExitReason::Programmatic)
    }
}

impl Default for AppExiting {
    fn default() -> Self {
        Self::new(ExitReason::User)
    }
}

/// Reason for application exit.
///
/// Used with [`AppExiting`] to indicate why the application is shutting down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ExitReason {
    /// User requested exit (close button, Alt+F4, quit menu, etc.).
    #[default]
    User,
    /// An unrecoverable error occurred.
    Error,
    /// Code explicitly requested shutdown.
    Programmatic,
}
