//! Stage label traits and type-erased identifiers.

use std::any::TypeId;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Trait for types that can be used as stage labels.
///
/// Stage labels identify stages in the schedule. They must be:
/// - Hashable and comparable for use as map keys
/// - Cloneable for storage
/// - Send + Sync for thread safety
///
/// # Implementation
///
/// The trait provides two methods:
/// - `label_id()`: Returns a unique TypeId for the label type
/// - `label_name()`: Returns a human-readable name for debugging
///
/// Most implementations should use `TypeId::of::<Self>()` for `label_id()`.
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::schedule::StageLabel;
/// use std::any::TypeId;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct MyCustomStage;
///
/// impl StageLabel for MyCustomStage {
///     fn label_id(&self) -> TypeId {
///         TypeId::of::<Self>()
///     }
///
///     fn label_name(&self) -> &'static str {
///         "MyCustomStage"
///     }
/// }
/// ```
pub trait StageLabel: Send + Sync + 'static {
    /// Returns a unique identifier for this label type.
    ///
    /// This is used for equality comparison and hashing when labels
    /// are stored in collections. Most implementations should return
    /// `TypeId::of::<Self>()`.
    fn label_id(&self) -> TypeId;

    /// Returns a human-readable name for this stage.
    ///
    /// This is used for debugging, logging, and error messages.
    fn label_name(&self) -> &'static str;

    /// Clones this label into a boxed trait object.
    ///
    /// This enables storing labels in collections while maintaining
    /// their concrete type identity through `label_id()`.
    fn dyn_clone(&self) -> Box<dyn StageLabel>;

    /// Compares this label with another for equality.
    ///
    /// Two labels are equal if they have the same `label_id()`.
    fn dyn_eq(&self, other: &dyn StageLabel) -> bool {
        self.label_id() == other.label_id()
    }

    /// Computes a hash of this label.
    ///
    /// Uses the `label_id()` for hashing to ensure consistency
    /// with `dyn_eq()`.
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        self.label_id().hash(&mut DynHasherWrapper(state));
    }
}

/// Wrapper to allow using `&mut dyn Hasher` with `Hash::hash`.
pub(crate) struct DynHasherWrapper<'a>(pub(crate) &'a mut dyn Hasher);

impl Hasher for DynHasherWrapper<'_> {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes);
    }
}

impl Clone for Box<dyn StageLabel> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

impl PartialEq for dyn StageLabel {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl Eq for dyn StageLabel {}

impl Hash for dyn StageLabel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label_id().hash(state);
    }
}

impl fmt::Debug for dyn StageLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StageLabel({})", self.label_name())
    }
}

// ============================================================================
// StageLabelId - Wrapper for type-erased stage labels
// ============================================================================

/// A type-erased stage label identifier.
///
/// This wraps a `Box<dyn StageLabel>` and provides the necessary trait
/// implementations for use as a HashMap key.
///
/// # Usage
///
/// ```
/// use goud_engine::ecs::schedule::{CoreStage, StageLabelId};
///
/// let id = StageLabelId::of::<CoreStage>(CoreStage::Update);
/// assert_eq!(id.name(), "Update");
/// ```
#[derive(Clone)]
pub struct StageLabelId(Box<dyn StageLabel>);

impl StageLabelId {
    /// Creates a new `StageLabelId` from a stage label.
    pub fn of<L: StageLabel + Clone>(label: L) -> Self {
        Self(Box::new(label))
    }

    /// Returns the name of the stage label.
    pub fn name(&self) -> &'static str {
        self.0.label_name()
    }

    /// Returns the TypeId of the underlying label.
    pub fn type_id(&self) -> TypeId {
        self.0.label_id()
    }

    /// Returns a reference to the inner label.
    pub fn inner(&self) -> &dyn StageLabel {
        &*self.0
    }
}

impl PartialEq for StageLabelId {
    fn eq(&self, other: &Self) -> bool {
        self.0.label_id() == other.0.label_id()
    }
}

impl Eq for StageLabelId {}

impl Hash for StageLabelId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.label_id().hash(state);
    }
}

impl fmt::Debug for StageLabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StageLabelId({})", self.0.label_name())
    }
}

impl fmt::Display for StageLabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.label_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::schedule::CoreStage;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_core_stage_label_id() {
        let ids: Vec<_> = CoreStage::all().iter().map(|s| s.label_id()).collect();
        let unique: HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
    }

    #[test]
    fn test_core_stage_label_name() {
        assert_eq!(CoreStage::PreUpdate.label_name(), "PreUpdate");
        assert_eq!(CoreStage::Update.label_name(), "Update");
        assert_eq!(CoreStage::PostUpdate.label_name(), "PostUpdate");
        assert_eq!(CoreStage::PreRender.label_name(), "PreRender");
        assert_eq!(CoreStage::Render.label_name(), "Render");
        assert_eq!(CoreStage::PostRender.label_name(), "PostRender");
    }

    #[test]
    fn test_core_stage_dyn_clone() {
        let stage = CoreStage::Update;
        let boxed: Box<dyn StageLabel> = stage.dyn_clone();
        assert_eq!(boxed.label_name(), "Update");
        assert_eq!(boxed.label_id(), stage.label_id());
    }

    #[test]
    fn test_core_stage_dyn_eq() {
        let a: &dyn StageLabel = &CoreStage::Update;
        let b: &dyn StageLabel = &CoreStage::Update;
        let c: &dyn StageLabel = &CoreStage::PreUpdate;
        assert!(a.dyn_eq(b));
        assert!(!a.dyn_eq(c));
    }

    #[test]
    fn test_custom_stage_label() {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct PhysicsStage;

        impl StageLabel for PhysicsStage {
            fn label_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }
            fn label_name(&self) -> &'static str {
                "PhysicsStage"
            }
            fn dyn_clone(&self) -> Box<dyn StageLabel> {
                Box::new(*self)
            }
        }

        let physics = PhysicsStage;
        assert_eq!(physics.label_name(), "PhysicsStage");
        assert_eq!(physics.label_id(), TypeId::of::<PhysicsStage>());
        assert_ne!(physics.label_id(), CoreStage::Update.label_id());
    }

    #[test]
    fn test_boxed_stage_label_clone() {
        let boxed: Box<dyn StageLabel> = Box::new(CoreStage::Update);
        let cloned = boxed.clone();
        assert_eq!(boxed.label_id(), cloned.label_id());
        assert_eq!(boxed.label_name(), cloned.label_name());
    }

    #[test]
    fn test_dyn_stage_label_eq() {
        let a: Box<dyn StageLabel> = Box::new(CoreStage::Update);
        let b: Box<dyn StageLabel> = Box::new(CoreStage::Update);
        let c: Box<dyn StageLabel> = Box::new(CoreStage::PreUpdate);
        assert_eq!(&*a, &*b);
        assert_ne!(&*a, &*c);
    }

    #[test]
    fn test_dyn_stage_label_debug() {
        let label: &dyn StageLabel = &CoreStage::Update;
        let debug_str = format!("{:?}", label);
        assert!(debug_str.contains("Update"));
    }

    // StageLabelId tests

    #[test]
    fn test_of() {
        let id = StageLabelId::of(CoreStage::Update);
        assert_eq!(id.name(), "Update");
    }

    #[test]
    fn test_name() {
        for stage in CoreStage::all() {
            let id = StageLabelId::of(stage);
            assert_eq!(id.name(), stage.label_name());
        }
    }

    #[test]
    fn test_type_id() {
        let id = StageLabelId::of(CoreStage::Update);
        assert_eq!(id.type_id(), CoreStage::Update.label_id());
    }

    #[test]
    fn test_inner() {
        let id = StageLabelId::of(CoreStage::Update);
        assert_eq!(id.inner().label_name(), "Update");
    }

    #[test]
    fn test_eq() {
        let a = StageLabelId::of(CoreStage::Update);
        let b = StageLabelId::of(CoreStage::Update);
        let c = StageLabelId::of(CoreStage::PreUpdate);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_hash() {
        let mut map = HashMap::new();
        map.insert(StageLabelId::of(CoreStage::Update), "update_systems");
        map.insert(StageLabelId::of(CoreStage::PreUpdate), "pre_update_systems");
        assert_eq!(
            map.get(&StageLabelId::of(CoreStage::Update)),
            Some(&"update_systems")
        );
        assert_eq!(
            map.get(&StageLabelId::of(CoreStage::PreUpdate)),
            Some(&"pre_update_systems")
        );
    }

    #[test]
    fn test_clone() {
        let id = StageLabelId::of(CoreStage::Update);
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_debug() {
        let id = StageLabelId::of(CoreStage::Update);
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("Update"));
    }

    #[test]
    fn test_display() {
        let id = StageLabelId::of(CoreStage::Update);
        assert_eq!(format!("{}", id), "Update");
    }

    #[test]
    fn test_from_core_stage() {
        let id: StageLabelId = CoreStage::Update.into();
        assert_eq!(id.name(), "Update");
    }
}
