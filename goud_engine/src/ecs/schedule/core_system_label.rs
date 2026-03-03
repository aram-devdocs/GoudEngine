//! Built-in system labels for common system phases.

use std::any::TypeId;
use std::fmt;
use std::hash::{Hash, Hasher};

use super::stage_label::DynHasherWrapper;
use super::system_label::SystemLabel;

/// Built-in system labels for common system phases.
///
/// These labels represent standard system phases that many games use.
/// They can be used to order custom systems relative to engine systems.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::{CoreSystemLabel, SystemLabel};
///
/// let input = CoreSystemLabel::Input;
/// let physics = CoreSystemLabel::Physics;
///
/// assert_eq!(input.label_name(), "Input");
/// assert_eq!(physics.label_name(), "Physics");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CoreSystemLabel {
    /// Input processing systems (keyboard, mouse, gamepad).
    #[default]
    Input,
    /// Physics simulation systems.
    Physics,
    /// Animation update systems.
    Animation,
    /// AI/behavior systems.
    AI,
    /// Audio playback systems.
    Audio,
    /// Transform propagation systems.
    TransformPropagate,
    /// Collision detection systems.
    Collision,
    /// Event processing systems.
    Events,
    /// UI layout systems.
    UILayout,
    /// UI rendering systems.
    UIRender,
}

impl CoreSystemLabel {
    /// Returns all core system labels in recommended execution order.
    pub fn all() -> &'static [CoreSystemLabel] {
        &[
            CoreSystemLabel::Input,
            CoreSystemLabel::Events,
            CoreSystemLabel::AI,
            CoreSystemLabel::Physics,
            CoreSystemLabel::Collision,
            CoreSystemLabel::Animation,
            CoreSystemLabel::TransformPropagate,
            CoreSystemLabel::Audio,
            CoreSystemLabel::UILayout,
            CoreSystemLabel::UIRender,
        ]
    }

    /// Returns the count of core system labels.
    #[inline]
    pub const fn count() -> usize {
        10
    }
}

impl SystemLabel for CoreSystemLabel {
    fn label_id(&self) -> TypeId {
        TypeId::of::<(CoreSystemLabel, u8)>()
    }

    fn label_name(&self) -> &'static str {
        match self {
            CoreSystemLabel::Input => "Input",
            CoreSystemLabel::Physics => "Physics",
            CoreSystemLabel::Animation => "Animation",
            CoreSystemLabel::AI => "AI",
            CoreSystemLabel::Audio => "Audio",
            CoreSystemLabel::TransformPropagate => "TransformPropagate",
            CoreSystemLabel::Collision => "Collision",
            CoreSystemLabel::Events => "Events",
            CoreSystemLabel::UILayout => "UILayout",
            CoreSystemLabel::UIRender => "UIRender",
        }
    }

    fn dyn_clone(&self) -> Box<dyn SystemLabel> {
        Box::new(*self)
    }

    fn dyn_eq(&self, other: &dyn SystemLabel) -> bool {
        if other.label_id() == TypeId::of::<(CoreSystemLabel, u8)>() {
            self.label_name() == other.label_name()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        TypeId::of::<CoreSystemLabel>().hash(&mut DynHasherWrapper(state));
        (*self as u8).hash(&mut DynHasherWrapper(state));
    }
}

impl fmt::Display for CoreSystemLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_labels() {
        let all = CoreSystemLabel::all();
        assert_eq!(all.len(), 10);
        assert_eq!(all[0], CoreSystemLabel::Input);
        assert_eq!(all[1], CoreSystemLabel::Events);
        assert_eq!(all[2], CoreSystemLabel::AI);
        assert_eq!(all[3], CoreSystemLabel::Physics);
    }

    #[test]
    fn test_count() {
        assert_eq!(CoreSystemLabel::count(), 10);
    }

    #[test]
    fn test_label_names() {
        assert_eq!(CoreSystemLabel::Input.label_name(), "Input");
        assert_eq!(CoreSystemLabel::Physics.label_name(), "Physics");
        assert_eq!(CoreSystemLabel::Animation.label_name(), "Animation");
        assert_eq!(CoreSystemLabel::AI.label_name(), "AI");
        assert_eq!(CoreSystemLabel::Audio.label_name(), "Audio");
        assert_eq!(
            CoreSystemLabel::TransformPropagate.label_name(),
            "TransformPropagate"
        );
        assert_eq!(CoreSystemLabel::Collision.label_name(), "Collision");
        assert_eq!(CoreSystemLabel::Events.label_name(), "Events");
        assert_eq!(CoreSystemLabel::UILayout.label_name(), "UILayout");
        assert_eq!(CoreSystemLabel::UIRender.label_name(), "UIRender");
    }

    #[test]
    fn test_default() {
        assert_eq!(CoreSystemLabel::default(), CoreSystemLabel::Input);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CoreSystemLabel::Physics), "Physics");
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", CoreSystemLabel::Physics), "Physics");
    }

    #[test]
    fn test_clone() {
        let label = CoreSystemLabel::Physics;
        let cloned = label.clone();
        assert_eq!(label, cloned);
    }

    #[test]
    fn test_dyn_clone() {
        let label = CoreSystemLabel::Physics;
        let boxed = label.dyn_clone();
        assert_eq!(boxed.label_name(), "Physics");
    }

    #[test]
    fn test_dyn_eq_same_variant() {
        let a = CoreSystemLabel::Physics;
        let b = CoreSystemLabel::Physics;
        assert!(a.dyn_eq(&b));
    }

    #[test]
    fn test_dyn_eq_different_variant() {
        let a = CoreSystemLabel::Physics;
        let b = CoreSystemLabel::Audio;
        assert!(!a.dyn_eq(&b));
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<CoreSystemLabel>();
        assert_sync::<CoreSystemLabel>();
    }
}
