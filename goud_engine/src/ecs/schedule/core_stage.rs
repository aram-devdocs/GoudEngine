//! Built-in stages for the standard game loop.

use std::any::TypeId;
use std::fmt;

use super::stage_label::{StageLabel, StageLabelId};

/// Built-in stages for the standard game loop.
///
/// These stages define the order of execution for systems within each frame.
/// The stages are executed in the order they are defined in this enum.
///
/// # Stage Order
///
/// 1. **PreUpdate**: Input processing, event polling, time updates
/// 2. **Update**: Main game logic, AI, user systems
/// 3. **PostUpdate**: State synchronization, hierarchy propagation, cleanup
/// 4. **PreRender**: Visibility culling, LOD selection, batch preparation
/// 5. **Render**: Actual draw calls and GPU command submission
/// 6. **PostRender**: Frame statistics, debug drawing, post-processing
///
/// # Usage
///
/// Systems are typically added to the `Update` stage:
///
/// ```
/// use goud_engine::ecs::schedule::CoreStage;
///
/// let stage = CoreStage::Update;
/// let physics_input = CoreStage::PreUpdate;
/// let physics_output = CoreStage::PostUpdate;
/// let cull = CoreStage::PreRender;
/// let draw = CoreStage::Render;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum CoreStage {
    /// Input processing, event polling, time updates.
    PreUpdate = 0,
    /// Main game logic, AI, physics step.
    Update = 1,
    /// State synchronization, hierarchy propagation, cleanup.
    PostUpdate = 2,
    /// Visibility culling, LOD selection, batch preparation.
    PreRender = 3,
    /// Actual draw calls and GPU command submission.
    Render = 4,
    /// Frame statistics, debug drawing, post-processing finalization.
    PostRender = 5,
}

impl CoreStage {
    /// Returns all core stages in execution order.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::schedule::CoreStage;
    ///
    /// for stage in CoreStage::all() {
    ///     println!("Stage: {:?}", stage);
    /// }
    /// ```
    pub const fn all() -> [CoreStage; 6] {
        [
            CoreStage::PreUpdate,
            CoreStage::Update,
            CoreStage::PostUpdate,
            CoreStage::PreRender,
            CoreStage::Render,
            CoreStage::PostRender,
        ]
    }

    /// Returns the number of core stages.
    pub const fn count() -> usize {
        6
    }

    /// Returns the index of this stage in the execution order.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::schedule::CoreStage;
    ///
    /// assert_eq!(CoreStage::PreUpdate.index(), 0);
    /// assert_eq!(CoreStage::Update.index(), 1);
    /// assert_eq!(CoreStage::PostRender.index(), 5);
    /// ```
    pub const fn index(&self) -> usize {
        *self as usize
    }

    /// Creates a CoreStage from an index, if valid.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::schedule::CoreStage;
    ///
    /// assert_eq!(CoreStage::from_index(0), Some(CoreStage::PreUpdate));
    /// assert_eq!(CoreStage::from_index(6), None);
    /// ```
    pub const fn from_index(index: usize) -> Option<CoreStage> {
        match index {
            0 => Some(CoreStage::PreUpdate),
            1 => Some(CoreStage::Update),
            2 => Some(CoreStage::PostUpdate),
            3 => Some(CoreStage::PreRender),
            4 => Some(CoreStage::Render),
            5 => Some(CoreStage::PostRender),
            _ => None,
        }
    }

    /// Returns the next stage in the execution order, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::schedule::CoreStage;
    ///
    /// assert_eq!(CoreStage::PreUpdate.next(), Some(CoreStage::Update));
    /// assert_eq!(CoreStage::PostRender.next(), None);
    /// ```
    pub const fn next(&self) -> Option<CoreStage> {
        CoreStage::from_index(self.index() + 1)
    }

    /// Returns the previous stage in the execution order, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::schedule::CoreStage;
    ///
    /// assert_eq!(CoreStage::Update.previous(), Some(CoreStage::PreUpdate));
    /// assert_eq!(CoreStage::PreUpdate.previous(), None);
    /// ```
    pub const fn previous(&self) -> Option<CoreStage> {
        if self.index() == 0 {
            None
        } else {
            CoreStage::from_index(self.index() - 1)
        }
    }

    /// Returns whether this stage is a pre-stage (PreUpdate or PreRender).
    pub const fn is_pre(&self) -> bool {
        matches!(self, CoreStage::PreUpdate | CoreStage::PreRender)
    }

    /// Returns whether this stage is a post-stage (PostUpdate or PostRender).
    pub const fn is_post(&self) -> bool {
        matches!(self, CoreStage::PostUpdate | CoreStage::PostRender)
    }

    /// Returns whether this stage is a rendering stage.
    pub const fn is_render(&self) -> bool {
        matches!(
            self,
            CoreStage::PreRender | CoreStage::Render | CoreStage::PostRender
        )
    }

    /// Returns whether this stage is a logic stage.
    pub const fn is_logic(&self) -> bool {
        matches!(
            self,
            CoreStage::PreUpdate | CoreStage::Update | CoreStage::PostUpdate
        )
    }
}

impl StageLabel for CoreStage {
    fn label_id(&self) -> TypeId {
        match self {
            CoreStage::PreUpdate => TypeId::of::<CoreStagePreUpdate>(),
            CoreStage::Update => TypeId::of::<CoreStageUpdate>(),
            CoreStage::PostUpdate => TypeId::of::<CoreStagePostUpdate>(),
            CoreStage::PreRender => TypeId::of::<CoreStagePreRender>(),
            CoreStage::Render => TypeId::of::<CoreStageRender>(),
            CoreStage::PostRender => TypeId::of::<CoreStagePostRender>(),
        }
    }

    fn label_name(&self) -> &'static str {
        match self {
            CoreStage::PreUpdate => "PreUpdate",
            CoreStage::Update => "Update",
            CoreStage::PostUpdate => "PostUpdate",
            CoreStage::PreRender => "PreRender",
            CoreStage::Render => "Render",
            CoreStage::PostRender => "PostRender",
        }
    }

    fn dyn_clone(&self) -> Box<dyn StageLabel> {
        Box::new(*self)
    }
}

// Marker types for unique TypeIds per CoreStage variant
struct CoreStagePreUpdate;
struct CoreStageUpdate;
struct CoreStagePostUpdate;
struct CoreStagePreRender;
struct CoreStageRender;
struct CoreStagePostRender;

impl Default for CoreStage {
    /// The default stage is `Update`, as this is where most game logic runs.
    fn default() -> Self {
        CoreStage::Update
    }
}

impl fmt::Display for CoreStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label_name())
    }
}

impl From<CoreStage> for StageLabelId {
    fn from(stage: CoreStage) -> Self {
        StageLabelId::of(stage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_all_stages() {
        let all = CoreStage::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], CoreStage::PreUpdate);
        assert_eq!(all[1], CoreStage::Update);
        assert_eq!(all[2], CoreStage::PostUpdate);
        assert_eq!(all[3], CoreStage::PreRender);
        assert_eq!(all[4], CoreStage::Render);
        assert_eq!(all[5], CoreStage::PostRender);
    }

    #[test]
    fn test_count() {
        assert_eq!(CoreStage::count(), 6);
        assert_eq!(CoreStage::all().len(), CoreStage::count());
    }

    #[test]
    fn test_index() {
        assert_eq!(CoreStage::PreUpdate.index(), 0);
        assert_eq!(CoreStage::Update.index(), 1);
        assert_eq!(CoreStage::PostUpdate.index(), 2);
        assert_eq!(CoreStage::PreRender.index(), 3);
        assert_eq!(CoreStage::Render.index(), 4);
        assert_eq!(CoreStage::PostRender.index(), 5);
    }

    #[test]
    fn test_from_index() {
        assert_eq!(CoreStage::from_index(0), Some(CoreStage::PreUpdate));
        assert_eq!(CoreStage::from_index(1), Some(CoreStage::Update));
        assert_eq!(CoreStage::from_index(2), Some(CoreStage::PostUpdate));
        assert_eq!(CoreStage::from_index(3), Some(CoreStage::PreRender));
        assert_eq!(CoreStage::from_index(4), Some(CoreStage::Render));
        assert_eq!(CoreStage::from_index(5), Some(CoreStage::PostRender));
        assert_eq!(CoreStage::from_index(6), None);
        assert_eq!(CoreStage::from_index(100), None);
    }

    #[test]
    fn test_next() {
        assert_eq!(CoreStage::PreUpdate.next(), Some(CoreStage::Update));
        assert_eq!(CoreStage::Update.next(), Some(CoreStage::PostUpdate));
        assert_eq!(CoreStage::PostUpdate.next(), Some(CoreStage::PreRender));
        assert_eq!(CoreStage::PreRender.next(), Some(CoreStage::Render));
        assert_eq!(CoreStage::Render.next(), Some(CoreStage::PostRender));
        assert_eq!(CoreStage::PostRender.next(), None);
    }

    #[test]
    fn test_previous() {
        assert_eq!(CoreStage::PreUpdate.previous(), None);
        assert_eq!(CoreStage::Update.previous(), Some(CoreStage::PreUpdate));
        assert_eq!(CoreStage::PostUpdate.previous(), Some(CoreStage::Update));
        assert_eq!(CoreStage::PreRender.previous(), Some(CoreStage::PostUpdate));
        assert_eq!(CoreStage::Render.previous(), Some(CoreStage::PreRender));
        assert_eq!(CoreStage::PostRender.previous(), Some(CoreStage::Render));
    }

    #[test]
    fn test_is_pre() {
        assert!(CoreStage::PreUpdate.is_pre());
        assert!(!CoreStage::Update.is_pre());
        assert!(!CoreStage::PostUpdate.is_pre());
        assert!(CoreStage::PreRender.is_pre());
        assert!(!CoreStage::Render.is_pre());
        assert!(!CoreStage::PostRender.is_pre());
    }

    #[test]
    fn test_is_post() {
        assert!(!CoreStage::PreUpdate.is_post());
        assert!(!CoreStage::Update.is_post());
        assert!(CoreStage::PostUpdate.is_post());
        assert!(!CoreStage::PreRender.is_post());
        assert!(!CoreStage::Render.is_post());
        assert!(CoreStage::PostRender.is_post());
    }

    #[test]
    fn test_is_render() {
        assert!(!CoreStage::PreUpdate.is_render());
        assert!(!CoreStage::Update.is_render());
        assert!(!CoreStage::PostUpdate.is_render());
        assert!(CoreStage::PreRender.is_render());
        assert!(CoreStage::Render.is_render());
        assert!(CoreStage::PostRender.is_render());
    }

    #[test]
    fn test_is_logic() {
        assert!(CoreStage::PreUpdate.is_logic());
        assert!(CoreStage::Update.is_logic());
        assert!(CoreStage::PostUpdate.is_logic());
        assert!(!CoreStage::PreRender.is_logic());
        assert!(!CoreStage::Render.is_logic());
        assert!(!CoreStage::PostRender.is_logic());
    }

    #[test]
    fn test_default() {
        assert_eq!(CoreStage::default(), CoreStage::Update);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CoreStage::PreUpdate), "PreUpdate");
        assert_eq!(format!("{}", CoreStage::Update), "Update");
        assert_eq!(format!("{}", CoreStage::PostUpdate), "PostUpdate");
        assert_eq!(format!("{}", CoreStage::PreRender), "PreRender");
        assert_eq!(format!("{}", CoreStage::Render), "Render");
        assert_eq!(format!("{}", CoreStage::PostRender), "PostRender");
    }

    #[test]
    fn test_debug() {
        assert_eq!(format!("{:?}", CoreStage::PreUpdate), "PreUpdate");
        assert_eq!(format!("{:?}", CoreStage::Update), "Update");
    }

    #[test]
    fn test_clone_and_copy() {
        let stage = CoreStage::Update;
        let cloned = stage;
        let copied: CoreStage = stage;
        assert_eq!(stage, cloned);
        assert_eq!(stage, copied);
    }

    #[test]
    fn test_eq() {
        assert_eq!(CoreStage::Update, CoreStage::Update);
        assert_ne!(CoreStage::Update, CoreStage::PreUpdate);
    }

    #[test]
    fn test_ord() {
        assert!(CoreStage::PreUpdate < CoreStage::Update);
        assert!(CoreStage::Update < CoreStage::PostUpdate);
        assert!(CoreStage::PostUpdate < CoreStage::PreRender);
        assert!(CoreStage::PreRender < CoreStage::Render);
        assert!(CoreStage::Render < CoreStage::PostRender);

        let mut stages = vec![
            CoreStage::PostRender,
            CoreStage::PreUpdate,
            CoreStage::Render,
            CoreStage::Update,
        ];
        stages.sort();
        assert_eq!(
            stages,
            vec![
                CoreStage::PreUpdate,
                CoreStage::Update,
                CoreStage::Render,
                CoreStage::PostRender
            ]
        );
    }

    #[test]
    fn test_hash() {
        let mut set = HashSet::new();
        set.insert(CoreStage::Update);
        set.insert(CoreStage::PreUpdate);
        set.insert(CoreStage::Update);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&CoreStage::Update));
        assert!(set.contains(&CoreStage::PreUpdate));
    }

    #[test]
    fn test_roundtrip_index() {
        for stage in CoreStage::all() {
            let index = stage.index();
            let recovered = CoreStage::from_index(index);
            assert_eq!(recovered, Some(stage));
        }
    }

    #[test]
    fn test_navigation_chain() {
        let mut current = Some(CoreStage::PreUpdate);
        let mut visited = Vec::new();
        while let Some(stage) = current {
            visited.push(stage);
            current = stage.next();
        }
        assert_eq!(visited, CoreStage::all().to_vec());
    }

    #[test]
    fn test_reverse_navigation() {
        let mut current = Some(CoreStage::PostRender);
        let mut visited = Vec::new();
        while let Some(stage) = current {
            visited.push(stage);
            current = stage.previous();
        }
        visited.reverse();
        assert_eq!(visited, CoreStage::all().to_vec());
    }
}
