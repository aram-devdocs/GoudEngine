//! Stage positioning and ordering types.

use std::cmp::Ordering;

use super::core_stage::CoreStage;
use super::stage_label::StageLabelId;

/// Specifies where a custom stage should run relative to a reference stage.
///
/// This is used when inserting custom stages into the schedule.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::{CoreStage, StagePosition};
///
/// let position = StagePosition::After(CoreStage::PreUpdate.into());
/// ```
#[derive(Clone, Debug)]
pub enum StagePosition {
    /// Run before the specified stage.
    Before(StageLabelId),
    /// Run after the specified stage.
    After(StageLabelId),
    /// Replace the specified stage.
    Replace(StageLabelId),
    /// Run at the very beginning, before all other stages.
    AtStart,
    /// Run at the very end, after all other stages.
    AtEnd,
}

impl StagePosition {
    /// Creates a position before a core stage.
    pub fn before_core(stage: CoreStage) -> Self {
        StagePosition::Before(stage.into())
    }

    /// Creates a position after a core stage.
    pub fn after_core(stage: CoreStage) -> Self {
        StagePosition::After(stage.into())
    }

    /// Creates a position that replaces a core stage.
    pub fn replace_core(stage: CoreStage) -> Self {
        StagePosition::Replace(stage.into())
    }
}

/// Result of comparing two stages in the execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageOrder {
    /// First stage runs before the second.
    Before,
    /// Stages run at the same time (same stage).
    Same,
    /// First stage runs after the second.
    After,
    /// Stages are unrelated (custom stages without explicit ordering).
    Unordered,
}

impl StageOrder {
    /// Converts from std::cmp::Ordering.
    pub fn from_ordering(ordering: Ordering) -> Self {
        match ordering {
            Ordering::Less => StageOrder::Before,
            Ordering::Equal => StageOrder::Same,
            Ordering::Greater => StageOrder::After,
        }
    }

    /// Converts to std::cmp::Ordering if ordered, None otherwise.
    pub fn to_ordering(self) -> Option<Ordering> {
        match self {
            StageOrder::Before => Some(Ordering::Less),
            StageOrder::Same => Some(Ordering::Equal),
            StageOrder::After => Some(Ordering::Greater),
            StageOrder::Unordered => None,
        }
    }

    /// Returns true if this represents a defined order (not Unordered).
    pub fn is_ordered(self) -> bool {
        !matches!(self, StageOrder::Unordered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod stage_position_tests {
        use super::*;

        #[test]
        fn test_before() {
            let pos = StagePosition::Before(CoreStage::Update.into());
            if let StagePosition::Before(id) = pos {
                assert_eq!(id.name(), "Update");
            } else {
                panic!("Expected Before variant");
            }
        }

        #[test]
        fn test_after() {
            let pos = StagePosition::After(CoreStage::PreUpdate.into());
            if let StagePosition::After(id) = pos {
                assert_eq!(id.name(), "PreUpdate");
            } else {
                panic!("Expected After variant");
            }
        }

        #[test]
        fn test_replace() {
            let pos = StagePosition::Replace(CoreStage::Render.into());
            if let StagePosition::Replace(id) = pos {
                assert_eq!(id.name(), "Render");
            } else {
                panic!("Expected Replace variant");
            }
        }

        #[test]
        fn test_at_start() {
            let pos = StagePosition::AtStart;
            assert!(matches!(pos, StagePosition::AtStart));
        }

        #[test]
        fn test_at_end() {
            let pos = StagePosition::AtEnd;
            assert!(matches!(pos, StagePosition::AtEnd));
        }

        #[test]
        fn test_before_core() {
            let pos = StagePosition::before_core(CoreStage::Update);
            if let StagePosition::Before(id) = pos {
                assert_eq!(id.name(), "Update");
            } else {
                panic!("Expected Before variant");
            }
        }

        #[test]
        fn test_after_core() {
            let pos = StagePosition::after_core(CoreStage::PreUpdate);
            if let StagePosition::After(id) = pos {
                assert_eq!(id.name(), "PreUpdate");
            } else {
                panic!("Expected After variant");
            }
        }

        #[test]
        fn test_replace_core() {
            let pos = StagePosition::replace_core(CoreStage::Render);
            if let StagePosition::Replace(id) = pos {
                assert_eq!(id.name(), "Render");
            } else {
                panic!("Expected Replace variant");
            }
        }

        #[test]
        fn test_clone() {
            let pos = StagePosition::After(CoreStage::Update.into());
            let cloned = pos.clone();
            if let (StagePosition::After(a), StagePosition::After(b)) = (&pos, &cloned) {
                assert_eq!(a.name(), b.name());
            } else {
                panic!("Clone should preserve variant");
            }
        }

        #[test]
        fn test_debug() {
            let pos = StagePosition::Before(CoreStage::Update.into());
            let debug_str = format!("{:?}", pos);
            assert!(debug_str.contains("Before"));
        }
    }

    mod stage_order_tests {
        use super::*;

        #[test]
        fn test_from_ordering() {
            assert_eq!(
                StageOrder::from_ordering(Ordering::Less),
                StageOrder::Before
            );
            assert_eq!(StageOrder::from_ordering(Ordering::Equal), StageOrder::Same);
            assert_eq!(
                StageOrder::from_ordering(Ordering::Greater),
                StageOrder::After
            );
        }

        #[test]
        fn test_to_ordering() {
            assert_eq!(StageOrder::Before.to_ordering(), Some(Ordering::Less));
            assert_eq!(StageOrder::Same.to_ordering(), Some(Ordering::Equal));
            assert_eq!(StageOrder::After.to_ordering(), Some(Ordering::Greater));
            assert_eq!(StageOrder::Unordered.to_ordering(), None);
        }

        #[test]
        fn test_is_ordered() {
            assert!(StageOrder::Before.is_ordered());
            assert!(StageOrder::Same.is_ordered());
            assert!(StageOrder::After.is_ordered());
            assert!(!StageOrder::Unordered.is_ordered());
        }

        #[test]
        fn test_clone() {
            let order = StageOrder::Before;
            let cloned = order;
            assert_eq!(order, cloned);
        }

        #[test]
        fn test_eq() {
            assert_eq!(StageOrder::Before, StageOrder::Before);
            assert_ne!(StageOrder::Before, StageOrder::After);
        }

        #[test]
        fn test_debug() {
            assert_eq!(format!("{:?}", StageOrder::Before), "Before");
            assert_eq!(format!("{:?}", StageOrder::Same), "Same");
            assert_eq!(format!("{:?}", StageOrder::After), "After");
            assert_eq!(format!("{:?}", StageOrder::Unordered), "Unordered");
        }
    }
}
