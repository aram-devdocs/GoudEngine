//! Asset change event types for the hot reload system.

use std::fmt;
use std::path::{Path, PathBuf};

// =============================================================================
// AssetChangeEvent
// =============================================================================

/// Event representing a change to an asset file.
///
/// Emitted by the hot reload system when a file is modified, created, or deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetChangeEvent {
    /// Asset file was modified.
    Modified {
        /// The path to the modified asset.
        path: PathBuf,
    },

    /// Asset file was created.
    Created {
        /// The path to the newly created asset.
        path: PathBuf,
    },

    /// Asset file was deleted.
    Deleted {
        /// The path to the deleted asset.
        path: PathBuf,
    },

    /// Asset file was renamed.
    Renamed {
        /// The old path of the asset.
        from: PathBuf,
        /// The new path of the asset.
        to: PathBuf,
    },
}

impl AssetChangeEvent {
    /// Returns the primary path affected by this event.
    pub fn path(&self) -> &Path {
        match self {
            Self::Modified { path } | Self::Created { path } | Self::Deleted { path } => path,
            Self::Renamed { to, .. } => to,
        }
    }

    /// Returns the event kind as a string.
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Modified { .. } => "Modified",
            Self::Created { .. } => "Created",
            Self::Deleted { .. } => "Deleted",
            Self::Renamed { .. } => "Renamed",
        }
    }

    /// Returns true if this is a modification event.
    pub fn is_modified(&self) -> bool {
        matches!(self, Self::Modified { .. })
    }

    /// Returns true if this is a creation event.
    pub fn is_created(&self) -> bool {
        matches!(self, Self::Created { .. })
    }

    /// Returns true if this is a deletion event.
    pub fn is_deleted(&self) -> bool {
        matches!(self, Self::Deleted { .. })
    }

    /// Returns true if this is a rename event.
    pub fn is_renamed(&self) -> bool {
        matches!(self, Self::Renamed { .. })
    }
}

impl fmt::Display for AssetChangeEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Modified { path } => write!(f, "Modified: {}", path.display()),
            Self::Created { path } => write!(f, "Created: {}", path.display()),
            Self::Deleted { path } => write!(f, "Deleted: {}", path.display()),
            Self::Renamed { from, to } => {
                write!(f, "Renamed: {} -> {}", from.display(), to.display())
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modified() {
        let event = AssetChangeEvent::Modified {
            path: PathBuf::from("test.png"),
        };

        assert_eq!(event.path(), Path::new("test.png"));
        assert_eq!(event.kind_str(), "Modified");
        assert!(event.is_modified());
        assert!(!event.is_created());
        assert!(!event.is_deleted());
        assert!(!event.is_renamed());
    }

    #[test]
    fn test_created() {
        let event = AssetChangeEvent::Created {
            path: PathBuf::from("new.png"),
        };

        assert_eq!(event.path(), Path::new("new.png"));
        assert_eq!(event.kind_str(), "Created");
        assert!(!event.is_modified());
        assert!(event.is_created());
    }

    #[test]
    fn test_deleted() {
        let event = AssetChangeEvent::Deleted {
            path: PathBuf::from("old.png"),
        };

        assert_eq!(event.path(), Path::new("old.png"));
        assert_eq!(event.kind_str(), "Deleted");
        assert!(event.is_deleted());
    }

    #[test]
    fn test_renamed() {
        let event = AssetChangeEvent::Renamed {
            from: PathBuf::from("old.png"),
            to: PathBuf::from("new.png"),
        };

        assert_eq!(event.path(), Path::new("new.png")); // Returns "to" path
        assert_eq!(event.kind_str(), "Renamed");
        assert!(event.is_renamed());
    }

    #[test]
    fn test_display() {
        let event = AssetChangeEvent::Modified {
            path: PathBuf::from("test.png"),
        };
        let display = format!("{}", event);
        assert!(display.contains("Modified"));
        assert!(display.contains("test.png"));
    }

    #[test]
    fn test_clone() {
        let event = AssetChangeEvent::Modified {
            path: PathBuf::from("test.png"),
        };
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
