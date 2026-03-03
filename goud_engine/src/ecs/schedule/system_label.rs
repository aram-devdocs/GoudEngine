//! System label traits and type-erased identifiers.

use std::any::TypeId;
use std::fmt;
use std::hash::{Hash, Hasher};

use super::stage_label::DynHasherWrapper;

/// Trait for types that can be used as system labels.
///
/// System labels provide a way to reference systems by name rather than ID,
/// enabling more flexible ordering constraints.
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::schedule::SystemLabel;
/// use std::any::TypeId;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct PhysicsSystem;
///
/// impl SystemLabel for PhysicsSystem {
///     fn label_id(&self) -> TypeId { TypeId::of::<Self>() }
///     fn label_name(&self) -> &'static str { "PhysicsSystem" }
/// }
/// ```
pub trait SystemLabel: Send + Sync + 'static {
    /// Returns a unique identifier for this label type.
    fn label_id(&self) -> TypeId;

    /// Returns a human-readable name for this label.
    fn label_name(&self) -> &'static str;

    /// Clones this label into a boxed trait object.
    fn dyn_clone(&self) -> Box<dyn SystemLabel>;

    /// Compares this label with another for equality.
    fn dyn_eq(&self, other: &dyn SystemLabel) -> bool {
        self.label_id() == other.label_id()
    }

    /// Computes a hash of this label.
    fn dyn_hash(&self, state: &mut dyn Hasher) {
        self.label_id().hash(&mut DynHasherWrapper(state));
    }
}

impl Clone for Box<dyn SystemLabel> {
    fn clone(&self) -> Self {
        self.dyn_clone()
    }
}

impl PartialEq for dyn SystemLabel {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

impl Eq for dyn SystemLabel {}

impl Hash for dyn SystemLabel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label_id().hash(state);
    }
}

impl fmt::Debug for dyn SystemLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SystemLabel({})", self.label_name())
    }
}

// ============================================================================
// SystemLabelId - Type-erased System Label
// ============================================================================

/// A type-erased system label identifier.
///
/// This wraps a `Box<dyn SystemLabel>` and provides the necessary trait
/// implementations for use as a map key.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::{SystemLabelId, SystemLabel, CoreSystemLabel};
///
/// let id = SystemLabelId::of(CoreSystemLabel::Input);
/// assert_eq!(id.name(), "Input");
/// ```
#[derive(Clone)]
pub struct SystemLabelId(Box<dyn SystemLabel>);

impl SystemLabelId {
    /// Creates a new `SystemLabelId` from a label.
    pub fn of<L: SystemLabel + Clone>(label: L) -> Self {
        Self(Box::new(label))
    }

    /// Returns the label's human-readable name.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.0.label_name()
    }

    /// Returns the label's type ID.
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.0.label_id()
    }

    /// Returns a reference to the inner label.
    #[inline]
    pub fn inner(&self) -> &dyn SystemLabel {
        &*self.0
    }
}

impl PartialEq for SystemLabelId {
    fn eq(&self, other: &Self) -> bool {
        self.0.label_id() == other.0.label_id()
    }
}

impl Eq for SystemLabelId {}

impl Hash for SystemLabelId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.label_id().hash(state);
    }
}

impl fmt::Debug for SystemLabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SystemLabelId({})", self.0.label_name())
    }
}

impl fmt::Display for SystemLabelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.label_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::schedule::CoreSystemLabel;
    use std::collections::HashMap;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct CustomLabel;

    impl SystemLabel for CustomLabel {
        fn label_id(&self) -> TypeId {
            TypeId::of::<Self>()
        }
        fn label_name(&self) -> &'static str {
            "CustomLabel"
        }
        fn dyn_clone(&self) -> Box<dyn SystemLabel> {
            Box::new(*self)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct AnotherLabel;

    impl SystemLabel for AnotherLabel {
        fn label_id(&self) -> TypeId {
            TypeId::of::<Self>()
        }
        fn label_name(&self) -> &'static str {
            "AnotherLabel"
        }
        fn dyn_clone(&self) -> Box<dyn SystemLabel> {
            Box::new(*self)
        }
    }

    // SystemLabel trait tests

    #[test]
    fn test_custom_label_name() {
        assert_eq!(CustomLabel.label_name(), "CustomLabel");
    }

    #[test]
    fn test_custom_label_id() {
        assert_eq!(CustomLabel.label_id(), TypeId::of::<CustomLabel>());
    }

    #[test]
    fn test_custom_label_dyn_clone() {
        let cloned = CustomLabel.dyn_clone();
        assert_eq!(cloned.label_name(), "CustomLabel");
    }

    #[test]
    fn test_dyn_eq_same() {
        assert!(CustomLabel.dyn_eq(&CustomLabel));
    }

    #[test]
    fn test_dyn_eq_different() {
        assert!(!CustomLabel.dyn_eq(&AnotherLabel));
    }

    #[test]
    fn test_box_clone() {
        let label: Box<dyn SystemLabel> = Box::new(CustomLabel);
        let cloned = label.clone();
        assert_eq!(label.label_name(), cloned.label_name());
    }

    #[test]
    fn test_dyn_partial_eq() {
        let a: &dyn SystemLabel = &CustomLabel;
        let b: &dyn SystemLabel = &CustomLabel;
        assert!(a == b);
    }

    #[test]
    fn test_dyn_hash() {
        use std::collections::hash_map::DefaultHasher;
        let a: &dyn SystemLabel = &CustomLabel;
        let b: &dyn SystemLabel = &CustomLabel;
        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();
        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);
        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_dyn_debug() {
        let label: &dyn SystemLabel = &CustomLabel;
        let debug = format!("{:?}", label);
        assert!(debug.contains("CustomLabel"));
    }

    #[test]
    fn test_system_label_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CustomLabel>();
        assert_sync::<CustomLabel>();
    }

    // SystemLabelId tests

    #[test]
    fn test_of() {
        let id = SystemLabelId::of(CustomLabel);
        assert_eq!(id.name(), "CustomLabel");
    }

    #[test]
    fn test_name() {
        let id = SystemLabelId::of(CoreSystemLabel::Physics);
        assert_eq!(id.name(), "Physics");
    }

    #[test]
    fn test_type_id() {
        let id = SystemLabelId::of(CustomLabel);
        assert_eq!(id.type_id(), TypeId::of::<CustomLabel>());
    }

    #[test]
    fn test_inner() {
        let id = SystemLabelId::of(CoreSystemLabel::Input);
        assert_eq!(id.inner().label_name(), "Input");
    }

    #[test]
    fn test_equality_same() {
        let a = SystemLabelId::of(CustomLabel);
        let b = SystemLabelId::of(CustomLabel);
        assert_eq!(a, b);
    }

    #[test]
    fn test_equality_different() {
        let a = SystemLabelId::of(CustomLabel);
        let b = SystemLabelId::of(CoreSystemLabel::Physics);
        assert_ne!(a, b);
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;
        let a = SystemLabelId::of(CustomLabel);
        let b = SystemLabelId::of(CustomLabel);
        let mut hasher_a = DefaultHasher::new();
        let mut hasher_b = DefaultHasher::new();
        a.hash(&mut hasher_a);
        b.hash(&mut hasher_b);
        assert_eq!(hasher_a.finish(), hasher_b.finish());
    }

    #[test]
    fn test_debug() {
        let id = SystemLabelId::of(CustomLabel);
        assert!(format!("{:?}", id).contains("CustomLabel"));
    }

    #[test]
    fn test_display() {
        let id = SystemLabelId::of(CoreSystemLabel::Audio);
        assert_eq!(format!("{}", id), "Audio");
    }

    #[test]
    fn test_clone() {
        let id = SystemLabelId::of(CustomLabel);
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_in_hashmap() {
        let mut map = HashMap::new();
        let id = SystemLabelId::of(CustomLabel);
        map.insert(id.clone(), "value");
        assert_eq!(map.get(&id), Some(&"value"));
    }
}
