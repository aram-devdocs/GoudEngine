//! Scheduling module for system execution ordering.
//!
//! This module defines stages and labels for organizing system execution order.
//! Stages are discrete phases of the game loop where systems are executed in groups.
//!
//! # Architecture
//!
//! The scheduling system is built around these key concepts:
//!
//! - **StageLabel**: Trait for types that can be used as stage identifiers
//! - **CoreStage**: Built-in stages for the standard game loop
//! - **Stage**: Trait for stage implementations that contain and run systems
//!
//! # Game Loop Stages
//!
//! The engine provides a standard set of stages for the game loop:
//!
//! ```text
//! ┌─────────────┐
//! │  PreUpdate  │  Input processing, event handling
//! ├─────────────┤
//! │   Update    │  Game logic, AI, physics step
//! ├─────────────┤
//! │ PostUpdate  │  Cleanup, state synchronization
//! ├─────────────┤
//! │  PreRender  │  Culling, LOD, batching
//! ├─────────────┤
//! │   Render    │  Draw calls, GPU submission
//! ├─────────────┤
//! │ PostRender  │  Frame cleanup, stats collection
//! └─────────────┘
//! ```
//!
//! # Usage
//!
//! ```
//! use goud_engine::ecs::schedule::{CoreStage, StageLabel};
//!
//! // CoreStage variants can be used directly
//! let stage = CoreStage::Update;
//! assert_eq!(stage.label_name(), "Update");
//!
//! // Stages implement StageLabel for use in scheduling
//! fn add_system_to_stage<L: StageLabel>(_label: L) {
//!     // Systems would be added to the stage identified by this label
//! }
//!
//! add_system_to_stage(CoreStage::Update);
//! ```
//!
//! # Custom Stages
//!
//! Games can define custom stages that interleave with core stages:
//!
//! ```ignore
//! use goud_engine::ecs::schedule::StageLabel;
//! use std::any::TypeId;
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! struct PhysicsStage;
//!
//! impl StageLabel for PhysicsStage {
//!     fn label_id(&self) -> TypeId {
//!         TypeId::of::<Self>()
//!     }
//!
//!     fn label_name(&self) -> &'static str {
//!         "PhysicsStage"
//!     }
//! }
//! ```

use std::any::TypeId;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::ecs::system::{BoxedSystem, IntoSystem, SystemId};
use crate::ecs::World;

// ============================================================================
// StageLabel Trait
// ============================================================================

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
struct DynHasherWrapper<'a>(&'a mut dyn Hasher);

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

// ============================================================================
// CoreStage Enum
// ============================================================================

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
/// 6. **PostRender**: Frame statistics, debug drawing, post-processing finalization
///
/// # Usage
///
/// Systems are typically added to the `Update` stage:
///
/// ```
/// use goud_engine::ecs::schedule::CoreStage;
///
/// // Most game logic goes in Update
/// let stage = CoreStage::Update;
///
/// // Physics might use PreUpdate and PostUpdate
/// let physics_input = CoreStage::PreUpdate;   // Sync bodies to physics world
/// let physics_output = CoreStage::PostUpdate; // Sync physics results back
///
/// // Rendering uses the render stages
/// let cull = CoreStage::PreRender;  // Determine what to draw
/// let draw = CoreStage::Render;      // Actually draw it
/// ```
///
/// # Custom Stages
///
/// For more granular control, games can define custom stages that run
/// between core stages. See [`StageLabel`] for details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum CoreStage {
    /// Input processing, event polling, time updates.
    ///
    /// Systems in this stage should:
    /// - Poll input devices
    /// - Process window events
    /// - Update time resources
    /// - Handle external events
    PreUpdate = 0,

    /// Main game logic, AI, physics step.
    ///
    /// This is where most game systems should run:
    /// - Player input handling
    /// - AI decision making
    /// - Physics simulation
    /// - Game state updates
    Update = 1,

    /// State synchronization, hierarchy propagation, cleanup.
    ///
    /// Systems in this stage should:
    /// - Propagate transform hierarchies
    /// - Synchronize state between systems
    /// - Clean up despawned entities
    /// - Validate game state
    PostUpdate = 2,

    /// Visibility culling, LOD selection, batch preparation.
    ///
    /// Systems in this stage prepare for rendering:
    /// - Frustum culling
    /// - LOD selection
    /// - Sprite batching
    /// - Sort render commands
    PreRender = 3,

    /// Actual draw calls and GPU command submission.
    ///
    /// This stage contains the actual rendering:
    /// - Background rendering
    /// - Sprite/mesh rendering
    /// - UI rendering
    /// - Post-processing
    Render = 4,

    /// Frame statistics, debug drawing, post-processing finalization.
    ///
    /// Systems in this stage:
    /// - Collect frame statistics
    /// - Debug/editor overlays
    /// - Present frame to display
    /// - Clean up temporary render data
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

    /// Returns whether this stage is a rendering stage (PreRender, Render, or PostRender).
    pub const fn is_render(&self) -> bool {
        matches!(
            self,
            CoreStage::PreRender | CoreStage::Render | CoreStage::PostRender
        )
    }

    /// Returns whether this stage is a logic stage (PreUpdate, Update, or PostUpdate).
    pub const fn is_logic(&self) -> bool {
        matches!(
            self,
            CoreStage::PreUpdate | CoreStage::Update | CoreStage::PostUpdate
        )
    }
}

impl StageLabel for CoreStage {
    fn label_id(&self) -> TypeId {
        // Use a discriminated TypeId based on the variant
        // This ensures different variants have different IDs
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

// ============================================================================
// StagePosition - For ordering custom stages relative to core stages
// ============================================================================

/// Specifies where a custom stage should run relative to a reference stage.
///
/// This is used when inserting custom stages into the schedule.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::{CoreStage, StagePosition};
///
/// // Run a physics stage after PreUpdate but before Update
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

// ============================================================================
// StageOrder - Comparison result for stage ordering
// ============================================================================

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

// ============================================================================
// System Ordering Types
// ============================================================================

/// Specifies an ordering constraint between two systems.
///
/// Ordering constraints are used to ensure systems run in a specific order,
/// regardless of the order they were added to the stage.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::SystemOrdering;
/// use goud_engine::ecs::system::SystemId;
///
/// // System A must run before System B
/// let ordering = SystemOrdering::Before {
///     system: SystemId::from_raw(1),
///     before: SystemId::from_raw(2),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemOrdering {
    /// The `system` must run before `before`.
    Before {
        /// The system that must run first.
        system: SystemId,
        /// The system that must run after.
        before: SystemId,
    },
    /// The `system` must run after `after`.
    After {
        /// The system that must run second.
        system: SystemId,
        /// The system that must run first.
        after: SystemId,
    },
}

impl SystemOrdering {
    /// Creates a constraint where `system` runs before `other`.
    #[inline]
    pub fn before(system: SystemId, other: SystemId) -> Self {
        SystemOrdering::Before {
            system,
            before: other,
        }
    }

    /// Creates a constraint where `system` runs after `other`.
    #[inline]
    pub fn after(system: SystemId, other: SystemId) -> Self {
        SystemOrdering::After {
            system,
            after: other,
        }
    }

    /// Returns the system that must run first according to this constraint.
    #[inline]
    pub fn first(&self) -> SystemId {
        match self {
            SystemOrdering::Before { system, .. } => *system,
            SystemOrdering::After { after, .. } => *after,
        }
    }

    /// Returns the system that must run second according to this constraint.
    #[inline]
    pub fn second(&self) -> SystemId {
        match self {
            SystemOrdering::Before { before, .. } => *before,
            SystemOrdering::After { system, .. } => *system,
        }
    }

    /// Returns the edge (from, to) for the dependency graph.
    ///
    /// The edge represents "from must run before to".
    #[inline]
    pub fn as_edge(&self) -> (SystemId, SystemId) {
        (self.first(), self.second())
    }

    /// Returns true if this ordering involves the given system.
    #[inline]
    pub fn involves(&self, id: SystemId) -> bool {
        self.first() == id || self.second() == id
    }
}

impl fmt::Display for SystemOrdering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemOrdering::Before { system, before } => {
                write!(f, "System {} before {}", system.raw(), before.raw())
            }
            SystemOrdering::After { system, after } => {
                write!(f, "System {} after {}", system.raw(), after.raw())
            }
        }
    }
}

// ============================================================================
// SystemLabel Trait - Named Labels for System Ordering
// ============================================================================

/// Trait for types that can be used as system labels.
///
/// System labels provide a way to reference systems by name rather than ID,
/// enabling more flexible ordering constraints. Labels can be applied to
/// systems and then used in `before()` and `after()` ordering constraints.
///
/// # Implementation
///
/// Similar to [`StageLabel`], the trait provides:
/// - `label_id()`: Returns a unique TypeId for the label type
/// - `label_name()`: Returns a human-readable name for debugging
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::schedule::SystemLabel;
/// use std::any::TypeId;
///
/// // Define a label type
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// struct PhysicsSystem;
///
/// impl SystemLabel for PhysicsSystem {
///     fn label_id(&self) -> TypeId {
///         TypeId::of::<Self>()
///     }
///
///     fn label_name(&self) -> &'static str {
///         "PhysicsSystem"
///     }
/// }
///
/// // Use in ordering
/// let label = PhysicsSystem;
/// assert_eq!(label.label_name(), "PhysicsSystem");
/// ```
///
/// # Pre-defined Labels
///
/// Common labels are provided for standard patterns:
/// - [`CoreSystemLabel`]: Built-in system phases
pub trait SystemLabel: Send + Sync + 'static {
    /// Returns a unique identifier for this label type.
    ///
    /// This is used for equality comparison and hashing when labels
    /// are stored in collections. Most implementations should return
    /// `TypeId::of::<Self>()`.
    fn label_id(&self) -> TypeId;

    /// Returns a human-readable name for this label.
    ///
    /// Used for debugging, logging, and error messages.
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

// ============================================================================
// CoreSystemLabel - Built-in System Labels
// ============================================================================

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
/// // Order custom systems relative to engine phases
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
        // Use a unique TypeId based on variant
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
        // Check if other is also a CoreSystemLabel by comparing label names
        // We can't downcast trait objects, so we compare by identity
        if other.label_id() == TypeId::of::<(CoreSystemLabel, u8)>() {
            // Same type family, compare by name
            self.label_name() == other.label_name()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        // Hash both the type and the variant
        TypeId::of::<CoreSystemLabel>().hash(&mut DynHasherWrapper(state));
        (*self as u8).hash(&mut DynHasherWrapper(state));
    }
}

impl fmt::Display for CoreSystemLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label_name())
    }
}

// ============================================================================
// SystemSet - Group Systems for Collective Ordering
// ============================================================================

/// A set of systems that can be ordered as a group.
///
/// System sets allow you to:
/// - Group related systems together
/// - Apply ordering constraints to the entire group
/// - Run conditions that affect all systems in the set
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::{SystemSet, SystemSetConfig};
/// use goud_engine::ecs::system::SystemId;
///
/// // Create a set for physics systems
/// let mut physics_set = SystemSet::new("PhysicsSystems");
///
/// // Add systems to the set (by ID)
/// let id1 = SystemId::from_raw(1);
/// let id2 = SystemId::from_raw(2);
/// physics_set.add(id1);
/// physics_set.add(id2);
///
/// assert_eq!(physics_set.len(), 2);
/// assert!(physics_set.contains(id1));
/// ```
#[derive(Debug, Clone)]
pub struct SystemSet {
    /// Human-readable name for the set.
    name: String,
    /// Systems in this set.
    systems: Vec<SystemId>,
    /// Unique set for fast contains checks.
    system_set: std::collections::HashSet<SystemId>,
}

impl SystemSet {
    /// Creates a new empty system set.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            system_set: std::collections::HashSet::new(),
        }
    }

    /// Creates a new system set with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
            system_set: std::collections::HashSet::with_capacity(capacity),
        }
    }

    /// Returns the name of this set.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a system to this set.
    ///
    /// Returns `true` if the system was added, `false` if already present.
    pub fn add(&mut self, system: SystemId) -> bool {
        if self.system_set.insert(system) {
            self.systems.push(system);
            true
        } else {
            false
        }
    }

    /// Removes a system from this set.
    ///
    /// Returns `true` if the system was removed, `false` if not present.
    pub fn remove(&mut self, system: SystemId) -> bool {
        if self.system_set.remove(&system) {
            self.systems.retain(|&id| id != system);
            true
        } else {
            false
        }
    }

    /// Returns `true` if this set contains the given system.
    #[inline]
    pub fn contains(&self, system: SystemId) -> bool {
        self.system_set.contains(&system)
    }

    /// Returns the number of systems in this set.
    #[inline]
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Returns `true` if this set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Returns an iterator over systems in this set.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().copied()
    }

    /// Clears all systems from this set.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_set.clear();
    }
}

impl Default for SystemSet {
    fn default() -> Self {
        Self::new("DefaultSet")
    }
}

// ============================================================================
// SystemSetConfig - Configuration for System Sets
// ============================================================================

/// Configuration for a system set's execution behavior.
///
/// This allows setting ordering constraints and run conditions
/// that apply to all systems in the set.
#[derive(Debug, Clone)]
pub struct SystemSetConfig {
    /// Labels that this set should run before.
    pub before_labels: Vec<SystemLabelId>,
    /// Labels that this set should run after.
    pub after_labels: Vec<SystemLabelId>,
    /// Whether the set is enabled.
    pub enabled: bool,
}

impl Default for SystemSetConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemSetConfig {
    /// Creates a new default configuration.
    pub fn new() -> Self {
        Self {
            before_labels: Vec::new(),
            after_labels: Vec::new(),
            enabled: true,
        }
    }

    /// Adds a "run before" constraint.
    pub fn before<L: SystemLabel + Clone>(mut self, label: L) -> Self {
        self.before_labels.push(SystemLabelId::of(label));
        self
    }

    /// Adds a "run after" constraint.
    pub fn after<L: SystemLabel + Clone>(mut self, label: L) -> Self {
        self.after_labels.push(SystemLabelId::of(label));
        self
    }

    /// Sets whether the set is enabled.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

// ============================================================================
// ChainedSystems - Strict Sequential Ordering
// ============================================================================

/// A chain of systems that must run in strict sequential order.
///
/// Chained systems are guaranteed to run one after another with no
/// interleaving. This is useful when systems have implicit dependencies
/// that cannot be expressed through component access patterns.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::ChainedSystems;
/// use goud_engine::ecs::system::SystemId;
///
/// let mut chain = ChainedSystems::new("InputChain");
///
/// let a = SystemId::from_raw(1);
/// let b = SystemId::from_raw(2);
/// let c = SystemId::from_raw(3);
///
/// chain.add(a);
/// chain.add(b);
/// chain.add(c);
///
/// // Get ordering constraints for the chain
/// let orderings = chain.to_orderings();
/// assert_eq!(orderings.len(), 2); // a->b, b->c
/// ```
#[derive(Debug, Clone)]
pub struct ChainedSystems {
    /// Name of the chain for debugging.
    name: String,
    /// Systems in chain order.
    systems: Vec<SystemId>,
}

impl ChainedSystems {
    /// Creates a new empty chain.
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
        }
    }

    /// Creates a chain with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
        }
    }

    /// Returns the name of this chain.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a system to the end of the chain.
    pub fn add(&mut self, system: SystemId) {
        self.systems.push(system);
    }

    /// Adds a system that must run immediately after another.
    ///
    /// If `after` is not in the chain, the system is added at the end.
    pub fn add_after(&mut self, system: SystemId, after: SystemId) {
        if let Some(pos) = self.systems.iter().position(|&id| id == after) {
            self.systems.insert(pos + 1, system);
        } else {
            self.systems.push(system);
        }
    }

    /// Returns the number of systems in the chain.
    #[inline]
    pub fn len(&self) -> usize {
        self.systems.len()
    }

    /// Returns `true` if the chain is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Returns an iterator over systems in chain order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().copied()
    }

    /// Converts this chain to a vector of ordering constraints.
    ///
    /// Each consecutive pair becomes a "before" constraint.
    pub fn to_orderings(&self) -> Vec<SystemOrdering> {
        self.systems
            .windows(2)
            .map(|pair| SystemOrdering::before(pair[0], pair[1]))
            .collect()
    }

    /// Clears the chain.
    pub fn clear(&mut self) {
        self.systems.clear();
    }
}

impl Default for ChainedSystems {
    fn default() -> Self {
        Self::new("Chain")
    }
}

/// Creates a chain of systems from the given system IDs.
///
/// This is a convenience function for creating strict sequential ordering.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::chain;
/// use goud_engine::ecs::system::SystemId;
///
/// let a = SystemId::from_raw(1);
/// let b = SystemId::from_raw(2);
/// let c = SystemId::from_raw(3);
///
/// let orderings = chain([a, b, c]);
/// assert_eq!(orderings.len(), 2); // a->b, b->c
/// ```
pub fn chain<I>(systems: I) -> Vec<SystemOrdering>
where
    I: IntoIterator<Item = SystemId>,
{
    let systems: Vec<_> = systems.into_iter().collect();
    systems
        .windows(2)
        .map(|pair| SystemOrdering::before(pair[0], pair[1]))
        .collect()
}

// ============================================================================
// LabeledOrderingConstraint - Label-based Ordering
// ============================================================================

/// An ordering constraint that uses labels instead of system IDs.
///
/// This allows defining ordering relationships before systems are
/// registered, using labels as placeholders.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::{LabeledOrderingConstraint, CoreSystemLabel};
///
/// // Define that custom systems run after input
/// let constraint = LabeledOrderingConstraint::after_label(CoreSystemLabel::Input);
/// ```
#[derive(Debug, Clone)]
pub enum LabeledOrderingConstraint {
    /// Run before systems with this label.
    BeforeLabel(SystemLabelId),
    /// Run after systems with this label.
    AfterLabel(SystemLabelId),
    /// Run before a specific system.
    BeforeSystem(SystemId),
    /// Run after a specific system.
    AfterSystem(SystemId),
}

impl LabeledOrderingConstraint {
    /// Creates a "before label" constraint.
    pub fn before_label<L: SystemLabel + Clone>(label: L) -> Self {
        LabeledOrderingConstraint::BeforeLabel(SystemLabelId::of(label))
    }

    /// Creates an "after label" constraint.
    pub fn after_label<L: SystemLabel + Clone>(label: L) -> Self {
        LabeledOrderingConstraint::AfterLabel(SystemLabelId::of(label))
    }

    /// Creates a "before system" constraint.
    #[inline]
    pub fn before_system(id: SystemId) -> Self {
        LabeledOrderingConstraint::BeforeSystem(id)
    }

    /// Creates an "after system" constraint.
    #[inline]
    pub fn after_system(id: SystemId) -> Self {
        LabeledOrderingConstraint::AfterSystem(id)
    }

    /// Returns `true` if this is a label-based constraint.
    #[inline]
    pub fn is_label_based(&self) -> bool {
        matches!(
            self,
            LabeledOrderingConstraint::BeforeLabel(_) | LabeledOrderingConstraint::AfterLabel(_)
        )
    }

    /// Returns `true` if this is a system-based constraint.
    #[inline]
    pub fn is_system_based(&self) -> bool {
        matches!(
            self,
            LabeledOrderingConstraint::BeforeSystem(_) | LabeledOrderingConstraint::AfterSystem(_)
        )
    }
}

impl fmt::Display for LabeledOrderingConstraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LabeledOrderingConstraint::BeforeLabel(label) => {
                write!(f, "before label '{}'", label.name())
            }
            LabeledOrderingConstraint::AfterLabel(label) => {
                write!(f, "after label '{}'", label.name())
            }
            LabeledOrderingConstraint::BeforeSystem(id) => {
                write!(f, "before system {}", id.raw())
            }
            LabeledOrderingConstraint::AfterSystem(id) => {
                write!(f, "after system {}", id.raw())
            }
        }
    }
}

// ============================================================================
// Cycle Detection Error
// ============================================================================

/// Error returned when a cycle is detected in system orderings.
///
/// A cycle means the ordering constraints are impossible to satisfy.
/// For example: A before B, B before C, C before A is a cycle.
#[derive(Debug, Clone)]
pub struct OrderingCycleError {
    /// Systems involved in the cycle (in cycle order).
    pub cycle: Vec<SystemId>,
    /// Human-readable system names for debugging.
    pub names: Vec<&'static str>,
}

impl OrderingCycleError {
    /// Creates a new cycle error.
    pub fn new(cycle: Vec<SystemId>, names: Vec<&'static str>) -> Self {
        Self { cycle, names }
    }

    /// Returns a human-readable description of the cycle.
    pub fn describe(&self) -> String {
        if self.names.is_empty() {
            return "Empty cycle detected".to_string();
        }

        let mut desc = String::new();
        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                desc.push_str(" -> ");
            }
            desc.push_str(name);
        }
        // Show it cycles back
        if !self.names.is_empty() {
            desc.push_str(" -> ");
            desc.push_str(self.names[0]);
        }
        desc
    }
}

impl fmt::Display for OrderingCycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ordering cycle detected: {}", self.describe())
    }
}

impl std::error::Error for OrderingCycleError {}

// ============================================================================
// Topological Sorter
// ============================================================================

/// Performs topological sorting of systems based on ordering constraints.
///
/// Uses Kahn's algorithm for topological sorting with cycle detection.
///
/// # Algorithm
///
/// 1. Build a dependency graph from ordering constraints
/// 2. Find all nodes with no incoming edges (no dependencies)
/// 3. Process those nodes, removing their outgoing edges
/// 4. Repeat until all nodes are processed or a cycle is detected
///
/// # Example
///
/// ```
/// use goud_engine::ecs::schedule::TopologicalSorter;
/// use goud_engine::ecs::system::SystemId;
///
/// let mut sorter = TopologicalSorter::new();
///
/// // Add systems
/// let a = SystemId::from_raw(1);
/// let b = SystemId::from_raw(2);
/// let c = SystemId::from_raw(3);
///
/// sorter.add_system(a, "SystemA");
/// sorter.add_system(b, "SystemB");
/// sorter.add_system(c, "SystemC");
///
/// // A runs before B, B runs before C
/// sorter.add_ordering(a, b);
/// sorter.add_ordering(b, c);
///
/// let sorted = sorter.sort().expect("Should not have cycles");
/// assert_eq!(sorted, vec![a, b, c]);
/// ```
#[derive(Debug, Default)]
pub struct TopologicalSorter {
    /// All systems to sort.
    systems: Vec<(SystemId, &'static str)>,
    /// Map from system ID to index in systems vec.
    system_indices: HashMap<SystemId, usize>,
    /// Edges: (from, to) means "from must run before to".
    edges: Vec<(SystemId, SystemId)>,
}

impl TopologicalSorter {
    /// Creates a new empty sorter.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sorter with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(systems: usize, edges: usize) -> Self {
        Self {
            systems: Vec::with_capacity(systems),
            system_indices: HashMap::with_capacity(systems),
            edges: Vec::with_capacity(edges),
        }
    }

    /// Adds a system to be sorted.
    ///
    /// If the system was already added, this is a no-op.
    pub fn add_system(&mut self, id: SystemId, name: &'static str) {
        if self.system_indices.contains_key(&id) {
            return;
        }
        let index = self.systems.len();
        self.systems.push((id, name));
        self.system_indices.insert(id, index);
    }

    /// Adds an ordering constraint: `first` must run before `second`.
    ///
    /// Both systems must have been added via `add_system` first.
    pub fn add_ordering(&mut self, first: SystemId, second: SystemId) {
        // Only add if both systems exist and edge doesn't already exist
        if self.system_indices.contains_key(&first)
            && self.system_indices.contains_key(&second)
            && first != second
            && !self.edges.contains(&(first, second))
        {
            self.edges.push((first, second));
        }
    }

    /// Adds a `SystemOrdering` constraint.
    pub fn add_system_ordering(&mut self, ordering: SystemOrdering) {
        let (first, second) = ordering.as_edge();
        self.add_ordering(first, second);
    }

    /// Returns the number of systems.
    #[inline]
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Returns the number of ordering constraints.
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Returns true if there are no systems.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.systems.is_empty()
    }

    /// Clears all systems and orderings.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_indices.clear();
        self.edges.clear();
    }

    /// Performs topological sort using Kahn's algorithm.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<SystemId>)` - Systems in valid execution order
    /// - `Err(OrderingCycleError)` - If a cycle was detected
    ///
    /// # Algorithm Complexity
    ///
    /// Time: O(V + E) where V = systems, E = ordering constraints
    /// Space: O(V + E)
    pub fn sort(&self) -> Result<Vec<SystemId>, OrderingCycleError> {
        if self.systems.is_empty() {
            return Ok(Vec::new());
        }

        let n = self.systems.len();

        // Build adjacency list and in-degree count
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut in_degree = vec![0usize; n];

        for &(from, to) in &self.edges {
            if let (Some(&from_idx), Some(&to_idx)) =
                (self.system_indices.get(&from), self.system_indices.get(&to))
            {
                adj[from_idx].push(to_idx);
                in_degree[to_idx] += 1;
            }
        }

        // Initialize queue with nodes that have no dependencies
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i);
            }
        }

        // Process nodes
        let mut result = Vec::with_capacity(n);
        while let Some(idx) = queue.pop_front() {
            result.push(self.systems[idx].0);

            // Remove this node's outgoing edges
            for &neighbor in &adj[idx] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        // Check for cycle
        if result.len() != n {
            // Find cycle for error reporting
            let cycle = self.find_cycle(&adj, &in_degree);
            return Err(cycle);
        }

        Ok(result)
    }

    /// Finds a cycle in the graph for error reporting.
    fn find_cycle(&self, adj: &[Vec<usize>], in_degree: &[usize]) -> OrderingCycleError {
        // Find a node still in the cycle (has remaining in-degree)
        let start = in_degree.iter().position(|&d| d > 0).unwrap_or(0);

        // DFS to find cycle
        let n = self.systems.len();
        let mut visited = vec![false; n];
        let mut rec_stack = vec![false; n];
        let mut path = Vec::new();

        fn dfs(
            node: usize,
            adj: &[Vec<usize>],
            visited: &mut [bool],
            rec_stack: &mut [bool],
            path: &mut Vec<usize>,
        ) -> Option<Vec<usize>> {
            visited[node] = true;
            rec_stack[node] = true;
            path.push(node);

            for &neighbor in &adj[node] {
                if !visited[neighbor] {
                    if let Some(cycle) = dfs(neighbor, adj, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack[neighbor] {
                    // Found cycle - extract it
                    if let Some(pos) = path.iter().position(|&x| x == neighbor) {
                        return Some(path[pos..].to_vec());
                    }
                }
            }

            path.pop();
            rec_stack[node] = false;
            None
        }

        let cycle_indices =
            dfs(start, adj, &mut visited, &mut rec_stack, &mut path).unwrap_or_else(|| vec![start]);

        let cycle: Vec<SystemId> = cycle_indices.iter().map(|&i| self.systems[i].0).collect();

        let names: Vec<&'static str> = cycle_indices.iter().map(|&i| self.systems[i].1).collect();

        OrderingCycleError::new(cycle, names)
    }

    /// Checks if sorting would produce a cycle without performing the full sort.
    ///
    /// This is useful for validating orderings before committing them.
    pub fn would_cycle(&self) -> bool {
        self.sort().is_err()
    }
}

impl Clone for TopologicalSorter {
    fn clone(&self) -> Self {
        Self {
            systems: self.systems.clone(),
            system_indices: self.system_indices.clone(),
            edges: self.edges.clone(),
        }
    }
}

// ============================================================================
// Stage Trait
// ============================================================================

/// Trait for stage implementations that can contain and run systems.
///
/// A stage is a container for systems that run together in a specific order.
/// Different stage implementations can provide different execution strategies
/// (sequential, parallel, etc.).
///
/// # Implementation
///
/// Stages must be `Send + Sync` for thread safety when stored in schedules.
///
/// # Example
///
/// ```
/// use goud_engine::ecs::{World};
/// use goud_engine::ecs::schedule::{Stage, SystemStage};
/// use goud_engine::ecs::system::System;
///
/// // Create a stage
/// let mut stage = SystemStage::new("GameLogic");
///
/// // Add systems to the stage
/// // stage.add_system(my_system);
///
/// // Run all systems
/// let mut world = World::new();
/// stage.run(&mut world);
/// ```
pub trait Stage: Send + Sync {
    /// Returns the name of this stage.
    fn name(&self) -> &str;

    /// Runs all systems in this stage on the given world.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to run systems on
    fn run(&mut self, world: &mut World);

    /// Initializes the stage and all its systems.
    ///
    /// Called once when the stage is first added to a schedule.
    fn initialize(&mut self, _world: &mut World) {
        // Default: no initialization needed
    }

    /// Returns the number of systems in this stage.
    fn system_count(&self) -> usize;

    /// Returns true if this stage has no systems.
    fn is_empty(&self) -> bool {
        self.system_count() == 0
    }
}

// ============================================================================
// SystemStage - Sequential System Container
// ============================================================================

/// A container that holds and runs systems sequentially.
///
/// `SystemStage` is the primary implementation of [`Stage`]. It stores systems
/// in a vector and runs them in the order they were added.
///
/// # Features
///
/// - **Sequential Execution**: Systems run one after another
/// - **Initialization**: Systems are initialized on first run
/// - **Conditional Execution**: Systems can skip execution via `should_run()`
/// - **Named**: Stages have names for debugging and logging
///
/// # Execution Order
///
/// Systems run in the order they are added. For explicit ordering control,
/// use the future ordering API (to be implemented in Step 3.3.6).
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::{World};
/// use goud_engine::ecs::schedule::SystemStage;
/// use goud_engine::ecs::system::{System, IntoSystem, BoxedSystem};
///
/// // Create a stage
/// let mut stage = SystemStage::new("Update");
///
/// // Define systems
/// struct SystemA;
/// impl System for SystemA {
///     fn name(&self) -> &'static str { "SystemA" }
///     fn run(&mut self, _world: &mut World) {
///         println!("System A running");
///     }
/// }
///
/// struct SystemB;
/// impl System for SystemB {
///     fn name(&self) -> &'static str { "SystemB" }
///     fn run(&mut self, _world: &mut World) {
///         println!("System B running");
///     }
/// }
///
/// // Add systems
/// stage.add_system(SystemA);
/// stage.add_system(SystemB);
///
/// // Run the stage
/// let mut world = World::new();
/// stage.run(&mut world);
/// // Output: System A running
/// //         System B running
/// ```
///
/// # Thread Safety
///
/// `SystemStage` is `Send + Sync`, allowing it to be stored in thread-safe
/// collections and shared between threads. However, the `run()` method
/// requires exclusive access (`&mut self`).
pub struct SystemStage {
    /// Human-readable name of this stage.
    name: String,
    /// Systems to run in this stage, in order.
    systems: Vec<BoxedSystem>,
    /// Map from system ID to index for fast lookup.
    system_indices: HashMap<SystemId, usize>,
    /// Whether the stage has been initialized.
    initialized: bool,
    /// Ordering constraints between systems.
    orderings: Vec<SystemOrdering>,
    /// Whether the system order needs to be rebuilt.
    order_dirty: bool,
}

impl SystemStage {
    /// Creates a new empty stage with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for debugging
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::schedule::SystemStage;
    ///
    /// let stage = SystemStage::new("Update");
    /// assert_eq!(stage.name(), "Update");
    /// assert!(stage.is_empty());
    /// ```
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            system_indices: HashMap::new(),
            initialized: false,
            orderings: Vec::new(),
            order_dirty: false,
        }
    }

    /// Creates a new stage with pre-allocated capacity.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for debugging
    /// * `capacity` - Initial capacity for systems
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::schedule::SystemStage;
    ///
    /// let stage = SystemStage::with_capacity("Physics", 10);
    /// assert_eq!(stage.system_count(), 0);
    /// ```
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
            system_indices: HashMap::with_capacity(capacity),
            initialized: false,
            orderings: Vec::with_capacity(capacity),
            order_dirty: false,
        }
    }

    /// Creates a new stage from a `CoreStage` variant.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::schedule::{SystemStage, CoreStage};
    ///
    /// let stage = SystemStage::from_core(CoreStage::Update);
    /// assert_eq!(stage.name(), "Update");
    /// ```
    #[inline]
    pub fn from_core(core_stage: CoreStage) -> Self {
        Self::new(core_stage.label_name())
    }

    /// Adds a system to this stage.
    ///
    /// The system will run after all previously added systems.
    /// Returns the `SystemId` assigned to the system.
    ///
    /// # Arguments
    ///
    /// * `system` - Any type that implements `IntoSystem`
    ///
    /// # Returns
    ///
    /// The `SystemId` of the added system, which can be used to reference
    /// it later (e.g., for removal).
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::{World};
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::System;
    ///
    /// struct MySystem;
    /// impl System for MySystem {
    ///     fn name(&self) -> &'static str { "MySystem" }
    ///     fn run(&mut self, _world: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// let id = stage.add_system(MySystem);
    /// assert!(id.is_valid());
    /// assert_eq!(stage.system_count(), 1);
    /// ```
    pub fn add_system<S, Marker>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<Marker>,
    {
        let boxed = system.into_system();
        let id = boxed.id();
        let index = self.systems.len();
        self.system_indices.insert(id, index);
        self.systems.push(boxed);
        id
    }

    /// Removes a system from this stage by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The `SystemId` of the system to remove
    ///
    /// # Returns
    ///
    /// `true` if the system was found and removed, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use goud_engine::ecs::{World};
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::System;
    ///
    /// struct MySystem;
    /// impl System for MySystem {
    ///     fn name(&self) -> &'static str { "MySystem" }
    ///     fn run(&mut self, _world: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// let id = stage.add_system(MySystem);
    /// assert_eq!(stage.system_count(), 1);
    ///
    /// assert!(stage.remove_system(id));
    /// assert_eq!(stage.system_count(), 0);
    ///
    /// // Removing again returns false
    /// assert!(!stage.remove_system(id));
    /// ```
    pub fn remove_system(&mut self, id: SystemId) -> bool {
        if let Some(index) = self.system_indices.remove(&id) {
            self.systems.remove(index);
            // Rebuild indices for all systems after the removed one
            self.system_indices.clear();
            for (i, system) in self.systems.iter().enumerate() {
                self.system_indices.insert(system.id(), i);
            }
            true
        } else {
            false
        }
    }

    /// Returns a reference to a system by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The `SystemId` of the system to get
    ///
    /// # Returns
    ///
    /// `Some(&BoxedSystem)` if found, `None` otherwise.
    #[inline]
    pub fn get_system(&self, id: SystemId) -> Option<&BoxedSystem> {
        self.system_indices.get(&id).map(|&i| &self.systems[i])
    }

    /// Returns a mutable reference to a system by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The `SystemId` of the system to get
    ///
    /// # Returns
    ///
    /// `Some(&mut BoxedSystem)` if found, `None` otherwise.
    #[inline]
    pub fn get_system_mut(&mut self, id: SystemId) -> Option<&mut BoxedSystem> {
        self.system_indices
            .get(&id)
            .copied()
            .map(|i| &mut self.systems[i])
    }

    /// Returns whether the stage contains a system with the given ID.
    #[inline]
    pub fn contains_system(&self, id: SystemId) -> bool {
        self.system_indices.contains_key(&id)
    }

    /// Returns an iterator over all system IDs in this stage.
    #[inline]
    pub fn system_ids(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().map(|s| s.id())
    }

    /// Returns an iterator over all systems in this stage.
    #[inline]
    pub fn systems(&self) -> impl Iterator<Item = &BoxedSystem> {
        self.systems.iter()
    }

    /// Returns a mutable iterator over all systems in this stage.
    #[inline]
    pub fn systems_mut(&mut self) -> impl Iterator<Item = &mut BoxedSystem> {
        self.systems.iter_mut()
    }

    /// Returns whether this stage has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Marks the stage as uninitialized, forcing re-initialization on next run.
    ///
    /// This is useful when systems are added after the stage has already run.
    #[inline]
    pub fn reset_initialized(&mut self) {
        self.initialized = false;
    }

    /// Clears all systems from this stage.
    ///
    /// After calling this, `system_count()` will return 0.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_indices.clear();
        self.orderings.clear();
        self.initialized = false;
        self.order_dirty = false;
    }

    /// Returns system names for debugging.
    pub fn system_names(&self) -> Vec<&'static str> {
        self.systems.iter().map(|s| s.name()).collect()
    }

    /// Runs a single system by ID on the given world.
    ///
    /// This respects the system's `should_run()` check.
    ///
    /// # Returns
    ///
    /// - `Some(true)` if the system was found and ran
    /// - `Some(false)` if the system was found but skipped due to `should_run()`
    /// - `None` if the system was not found
    pub fn run_system(&mut self, id: SystemId, world: &mut World) -> Option<bool> {
        if let Some(&index) = self.system_indices.get(&id) {
            let system = &mut self.systems[index];
            if system.should_run(world) {
                system.run(world);
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        }
    }

    // =========================================================================
    // System Ordering API
    // =========================================================================

    /// Adds an ordering constraint: `first` must run before `second`.
    ///
    /// Both systems must exist in this stage. The order will be enforced
    /// when `rebuild_order()` is called (either explicitly or during `run()`).
    ///
    /// # Arguments
    ///
    /// * `first` - System that must run first
    /// * `second` - System that must run after `first`
    ///
    /// # Returns
    ///
    /// `true` if both systems exist and the constraint was added,
    /// `false` if either system doesn't exist.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World};
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::System;
    ///
    /// struct SystemA;
    /// impl System for SystemA {
    ///     fn name(&self) -> &'static str { "SystemA" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// struct SystemB;
    /// impl System for SystemB {
    ///     fn name(&self) -> &'static str { "SystemB" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// let id_a = stage.add_system(SystemA);
    /// let id_b = stage.add_system(SystemB);
    ///
    /// // Ensure A runs before B
    /// assert!(stage.add_ordering(id_a, id_b));
    /// ```
    pub fn add_ordering(&mut self, first: SystemId, second: SystemId) -> bool {
        // Validate both systems exist
        if !self.contains_system(first) || !self.contains_system(second) {
            return false;
        }

        // Don't add self-ordering
        if first == second {
            return false;
        }

        // Don't add duplicate orderings
        let ordering = SystemOrdering::before(first, second);
        if self.orderings.contains(&ordering) {
            return true; // Already exists, success
        }

        self.orderings.push(ordering);
        self.order_dirty = true;
        true
    }

    /// Adds a constraint that `system` must run before `other`.
    ///
    /// Convenience wrapper around [`add_ordering`](Self::add_ordering).
    #[inline]
    pub fn set_before(&mut self, system: SystemId, other: SystemId) -> bool {
        self.add_ordering(system, other)
    }

    /// Adds a constraint that `system` must run after `other`.
    ///
    /// Convenience wrapper around [`add_ordering`](Self::add_ordering).
    #[inline]
    pub fn set_after(&mut self, system: SystemId, other: SystemId) -> bool {
        self.add_ordering(other, system)
    }

    /// Removes all ordering constraints involving the given system.
    ///
    /// Returns the number of constraints removed.
    pub fn remove_orderings_for(&mut self, system: SystemId) -> usize {
        let before_len = self.orderings.len();
        self.orderings.retain(|o| !o.involves(system));
        let removed = before_len - self.orderings.len();
        if removed > 0 {
            self.order_dirty = true;
        }
        removed
    }

    /// Clears all ordering constraints.
    pub fn clear_orderings(&mut self) {
        if !self.orderings.is_empty() {
            self.orderings.clear();
            self.order_dirty = true;
        }
    }

    /// Chains multiple systems for strict sequential execution.
    ///
    /// This ensures the systems run in exactly the order specified,
    /// with no interleaving. Equivalent to calling `add_ordering` for
    /// each consecutive pair.
    ///
    /// # Arguments
    ///
    /// * `systems` - System IDs in the order they should run
    ///
    /// # Returns
    ///
    /// The number of ordering constraints added (len - 1).
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::{System, SystemId};
    /// use goud_engine::ecs::World;
    ///
    /// struct SysA;
    /// impl System for SysA {
    ///     fn name(&self) -> &'static str { "A" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    /// struct SysB;
    /// impl System for SysB {
    ///     fn name(&self) -> &'static str { "B" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    /// struct SysC;
    /// impl System for SysC {
    ///     fn name(&self) -> &'static str { "C" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// let a = stage.add_system(SysA);
    /// let b = stage.add_system(SysB);
    /// let c = stage.add_system(SysC);
    ///
    /// // Chain them: A -> B -> C
    /// let count = stage.chain_systems([a, b, c]);
    /// assert_eq!(count, 2); // Two orderings: A->B and B->C
    /// ```
    pub fn chain_systems<I>(&mut self, systems: I) -> usize
    where
        I: IntoIterator<Item = SystemId>,
    {
        let orderings = chain(systems);
        let count = orderings.len();
        for ordering in orderings {
            let (first, second) = ordering.as_edge();
            self.add_ordering(first, second);
        }
        count
    }

    /// Adds all orderings from a ChainedSystems.
    ///
    /// # Arguments
    ///
    /// * `chained` - A ChainedSystems instance with systems in order
    ///
    /// # Returns
    ///
    /// The number of ordering constraints added.
    pub fn add_chain(&mut self, chained: &ChainedSystems) -> usize {
        let orderings = chained.to_orderings();
        let count = orderings.len();
        for ordering in orderings {
            let (first, second) = ordering.as_edge();
            self.add_ordering(first, second);
        }
        count
    }

    /// Returns an iterator over all ordering constraints.
    #[inline]
    pub fn orderings(&self) -> impl Iterator<Item = &SystemOrdering> {
        self.orderings.iter()
    }

    /// Returns the number of ordering constraints.
    #[inline]
    pub fn ordering_count(&self) -> usize {
        self.orderings.len()
    }

    /// Returns whether the system order needs to be rebuilt.
    #[inline]
    pub fn is_order_dirty(&self) -> bool {
        self.order_dirty
    }

    /// Rebuilds the system execution order based on ordering constraints.
    ///
    /// Uses topological sorting to determine a valid order that respects
    /// all constraints. If no constraints exist, the order remains unchanged.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the order was successfully rebuilt
    /// - `Err(OrderingCycleError)` if the constraints form a cycle
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World};
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::System;
    ///
    /// struct SystemA;
    /// impl System for SystemA {
    ///     fn name(&self) -> &'static str { "SystemA" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// struct SystemB;
    /// impl System for SystemB {
    ///     fn name(&self) -> &'static str { "SystemB" }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// let id_b = stage.add_system(SystemB);  // Added first
    /// let id_a = stage.add_system(SystemA);  // Added second
    ///
    /// // Ensure A runs before B (reversing the add order)
    /// stage.add_ordering(id_a, id_b);
    /// stage.rebuild_order().expect("No cycles");
    ///
    /// // Now A will run before B
    /// let names = stage.system_names();
    /// assert_eq!(names[0], "SystemA");
    /// assert_eq!(names[1], "SystemB");
    /// ```
    pub fn rebuild_order(&mut self) -> Result<(), OrderingCycleError> {
        // If no orderings, nothing to do
        if self.orderings.is_empty() {
            self.order_dirty = false;
            return Ok(());
        }

        // Build topological sorter
        let mut sorter = TopologicalSorter::with_capacity(self.systems.len(), self.orderings.len());

        // Add all systems
        for system in &self.systems {
            sorter.add_system(system.id(), system.name());
        }

        // Add all ordering constraints
        for ordering in &self.orderings {
            sorter.add_system_ordering(*ordering);
        }

        // Perform topological sort
        let sorted_ids = sorter.sort()?;

        // Reorder systems according to sorted order
        let mut new_systems = Vec::with_capacity(self.systems.len());
        let mut new_indices = HashMap::with_capacity(self.systems.len());

        // Build a map of id -> system for quick lookup
        let mut system_map: HashMap<SystemId, BoxedSystem> =
            self.systems.drain(..).map(|s| (s.id(), s)).collect();

        for id in sorted_ids {
            if let Some(system) = system_map.remove(&id) {
                new_indices.insert(id, new_systems.len());
                new_systems.push(system);
            }
        }

        // Add any remaining systems (shouldn't happen, but be safe)
        for (id, system) in system_map {
            new_indices.insert(id, new_systems.len());
            new_systems.push(system);
        }

        self.systems = new_systems;
        self.system_indices = new_indices;
        self.order_dirty = false;

        Ok(())
    }

    /// Checks if adding an ordering would create a cycle.
    ///
    /// This does not modify the stage; it only checks.
    pub fn would_ordering_cycle(&self, first: SystemId, second: SystemId) -> bool {
        let mut sorter =
            TopologicalSorter::with_capacity(self.systems.len(), self.orderings.len() + 1);

        // Add all systems
        for system in &self.systems {
            sorter.add_system(system.id(), system.name());
        }

        // Add existing orderings
        for ordering in &self.orderings {
            sorter.add_system_ordering(*ordering);
        }

        // Add the proposed ordering
        sorter.add_ordering(first, second);

        // Check for cycle
        sorter.would_cycle()
    }

    /// Returns the orderings involving a specific system.
    pub fn orderings_for(&self, system: SystemId) -> Vec<&SystemOrdering> {
        self.orderings
            .iter()
            .filter(|o| o.involves(system))
            .collect()
    }

    // =========================================================================
    // Conflict Detection API
    // =========================================================================

    /// Checks if any systems in this stage have conflicting access patterns.
    ///
    /// Returns `true` if there are any access conflicts between systems.
    /// Use [`find_conflicts`](Self::find_conflicts) to get detailed information.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World};
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::System;
    /// use goud_engine::ecs::query::Access;
    /// use goud_engine::ecs::component::ComponentId;
    /// use goud_engine::ecs::Component;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Position { x: f32, y: f32 }
    /// impl Component for Position {}
    ///
    /// struct WriterSystem;
    /// impl System for WriterSystem {
    ///     fn name(&self) -> &'static str { "WriterSystem" }
    ///     fn component_access(&self) -> Access {
    ///         let mut access = Access::new();
    ///         access.add_write(ComponentId::of::<Position>());
    ///         access
    ///     }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// struct ReaderSystem;
    /// impl System for ReaderSystem {
    ///     fn name(&self) -> &'static str { "ReaderSystem" }
    ///     fn component_access(&self) -> Access {
    ///         let mut access = Access::new();
    ///         access.add_read(ComponentId::of::<Position>());
    ///         access
    ///     }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// stage.add_system(WriterSystem);
    /// stage.add_system(ReaderSystem);
    ///
    /// assert!(stage.has_conflicts());
    /// ```
    pub fn has_conflicts(&self) -> bool {
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                if self.systems[i].conflicts_with(&self.systems[j]) {
                    return true;
                }
            }
        }
        false
    }

    /// Finds all conflicting system pairs in this stage.
    ///
    /// Returns a vector of `SystemConflict` structs describing each conflict.
    /// If there are no conflicts, returns an empty vector.
    ///
    /// # Performance
    ///
    /// This is O(n²) in the number of systems. For stages with many systems,
    /// consider caching the result if called frequently.
    ///
    /// # Example
    ///
    /// ```
    /// use goud_engine::ecs::{World};
    /// use goud_engine::ecs::schedule::SystemStage;
    /// use goud_engine::ecs::system::System;
    /// use goud_engine::ecs::query::Access;
    /// use goud_engine::ecs::component::ComponentId;
    /// use goud_engine::ecs::Component;
    ///
    /// #[derive(Clone, Copy)]
    /// struct Health(f32);
    /// impl Component for Health {}
    ///
    /// struct DamageSystem;
    /// impl System for DamageSystem {
    ///     fn name(&self) -> &'static str { "DamageSystem" }
    ///     fn component_access(&self) -> Access {
    ///         let mut access = Access::new();
    ///         access.add_write(ComponentId::of::<Health>());
    ///         access
    ///     }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// struct HealSystem;
    /// impl System for HealSystem {
    ///     fn name(&self) -> &'static str { "HealSystem" }
    ///     fn component_access(&self) -> Access {
    ///         let mut access = Access::new();
    ///         access.add_write(ComponentId::of::<Health>());
    ///         access
    ///     }
    ///     fn run(&mut self, _: &mut World) {}
    /// }
    ///
    /// let mut stage = SystemStage::new("Update");
    /// stage.add_system(DamageSystem);
    /// stage.add_system(HealSystem);
    ///
    /// let conflicts = stage.find_conflicts();
    /// assert_eq!(conflicts.len(), 1);
    /// assert_eq!(conflicts[0].first_system_name, "DamageSystem");
    /// assert_eq!(conflicts[0].second_system_name, "HealSystem");
    /// ```
    pub fn find_conflicts(&self) -> Vec<SystemConflict> {
        let mut conflicts = Vec::new();

        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                let access_i = self.systems[i].component_access();
                let access_j = self.systems[j].component_access();

                if let Some(access_conflict) = access_i.get_conflicts(&access_j) {
                    conflicts.push(SystemConflict {
                        first_system_id: self.systems[i].id(),
                        first_system_name: self.systems[i].name(),
                        second_system_id: self.systems[j].id(),
                        second_system_name: self.systems[j].name(),
                        conflict: access_conflict,
                    });
                }
            }
        }

        conflicts
    }

    /// Checks if a specific system conflicts with any other system in this stage.
    ///
    /// # Arguments
    ///
    /// * `id` - The system ID to check
    ///
    /// # Returns
    ///
    /// A vector of conflicts involving the specified system.
    pub fn find_conflicts_for_system(&self, id: SystemId) -> Vec<SystemConflict> {
        let index = match self.system_indices.get(&id) {
            Some(&i) => i,
            None => return Vec::new(),
        };

        let mut conflicts = Vec::new();
        let access_target = self.systems[index].component_access();

        for (i, system) in self.systems.iter().enumerate() {
            if i == index {
                continue;
            }

            let access = system.component_access();
            if let Some(access_conflict) = access_target.get_conflicts(&access) {
                conflicts.push(SystemConflict {
                    first_system_id: self.systems[index].id(),
                    first_system_name: self.systems[index].name(),
                    second_system_id: system.id(),
                    second_system_name: system.name(),
                    conflict: access_conflict,
                });
            }
        }

        conflicts
    }

    /// Returns all systems that are read-only (don't write any components).
    ///
    /// Read-only systems can potentially run in parallel with each other.
    pub fn read_only_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| s.is_read_only())
            .map(|s| s.id())
            .collect()
    }

    /// Returns all systems that have write access to components.
    pub fn writing_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| !s.is_read_only())
            .map(|s| s.id())
            .collect()
    }

    /// Groups systems by whether they conflict with each other.
    ///
    /// Returns groups of system IDs where systems within a group don't conflict.
    /// Systems can potentially run in parallel within their group.
    ///
    /// # Algorithm
    ///
    /// Uses a greedy coloring algorithm. Not guaranteed to find the optimal
    /// grouping, but provides a reasonable parallel execution plan.
    pub fn compute_parallel_groups(&self) -> Vec<Vec<SystemId>> {
        if self.systems.is_empty() {
            return Vec::new();
        }

        let n = self.systems.len();
        let mut groups: Vec<Vec<SystemId>> = Vec::new();

        // Track which group each system is assigned to (None = unassigned)
        let mut assigned = vec![None::<usize>; n];

        // Using index loop to avoid borrow checker issues with self.systems and assigned
        #[allow(clippy::needless_range_loop)]
        for i in 0..n {
            // Try to find an existing group this system can join
            let mut found_group = false;
            for (group_idx, group) in groups.iter().enumerate() {
                // Check if system i conflicts with any system in this group
                let conflicts_with_group = group.iter().any(|&other_id| {
                    let other_idx = self.system_indices[&other_id];
                    self.systems[i].conflicts_with(&self.systems[other_idx])
                });

                if !conflicts_with_group {
                    // Can add to this group
                    assigned[i] = Some(group_idx);
                    found_group = true;
                    break;
                }
            }

            if !found_group {
                // Create new group
                assigned[i] = Some(groups.len());
                groups.push(Vec::new());
            }

            // Add system to its group
            let group_idx = assigned[i].unwrap();
            groups[group_idx].push(self.systems[i].id());
        }

        groups
    }

    /// Returns the number of conflicts in this stage.
    pub fn conflict_count(&self) -> usize {
        let mut count = 0;
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                if self.systems[i].conflicts_with(&self.systems[j]) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Returns combined access pattern for all systems in this stage.
    ///
    /// This represents the total component and resource access of the entire stage.
    pub fn combined_access(&self) -> crate::ecs::query::Access {
        let mut combined = crate::ecs::query::Access::new();
        for system in &self.systems {
            combined.extend(&system.component_access());
        }
        combined
    }
}

// =============================================================================
// System Conflict Info
// =============================================================================

/// Information about a conflict between two systems.
///
/// This struct provides detailed information about which systems conflict
/// and what specific accesses cause the conflict.
#[derive(Debug, Clone)]
pub struct SystemConflict {
    /// ID of the first conflicting system.
    pub first_system_id: SystemId,
    /// Name of the first conflicting system.
    pub first_system_name: &'static str,
    /// ID of the second conflicting system.
    pub second_system_id: SystemId,
    /// Name of the second conflicting system.
    pub second_system_name: &'static str,
    /// Detailed access conflict information.
    pub conflict: crate::ecs::query::AccessConflict,
}

impl SystemConflict {
    /// Returns the pair of system IDs involved in this conflict.
    #[inline]
    pub fn system_ids(&self) -> (SystemId, SystemId) {
        (self.first_system_id, self.second_system_id)
    }

    /// Returns the pair of system names involved in this conflict.
    #[inline]
    pub fn system_names(&self) -> (&'static str, &'static str) {
        (self.first_system_name, self.second_system_name)
    }

    /// Returns true if this is a write-write conflict.
    #[inline]
    pub fn is_write_write(&self) -> bool {
        self.conflict.has_write_write()
    }

    /// Returns the number of conflicting components.
    #[inline]
    pub fn component_conflict_count(&self) -> usize {
        self.conflict.component_count()
    }

    /// Returns the number of conflicting resources.
    #[inline]
    pub fn resource_conflict_count(&self) -> usize {
        self.conflict.resource_count()
    }

    /// Returns the total number of conflicts.
    #[inline]
    pub fn total_conflict_count(&self) -> usize {
        self.conflict.total_count()
    }
}

impl fmt::Display for SystemConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Conflict: '{}' vs '{}' - {}",
            self.first_system_name, self.second_system_name, self.conflict
        )
    }
}

impl Stage for SystemStage {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&mut self, world: &mut World) {
        // Rebuild order if dirty (silently ignore cycle errors during run)
        if self.order_dirty {
            let _ = self.rebuild_order();
        }

        // Initialize if needed
        if !self.initialized {
            self.initialize(world);
        }

        // Run each system in order
        for system in &mut self.systems {
            if system.should_run(world) {
                system.run(world);
            }
        }
    }

    fn initialize(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.initialize(world);
        }
        self.initialized = true;
    }

    #[inline]
    fn system_count(&self) -> usize {
        self.systems.len()
    }
}

impl Default for SystemStage {
    fn default() -> Self {
        Self::new("DefaultStage")
    }
}

impl fmt::Debug for SystemStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemStage")
            .field("name", &self.name)
            .field("system_count", &self.systems.len())
            .field("initialized", &self.initialized)
            .field("ordering_count", &self.orderings.len())
            .field("order_dirty", &self.order_dirty)
            .field("systems", &self.system_names())
            .finish()
    }
}

// SystemStage is Send + Sync because:
// - String is Send + Sync
// - Vec<BoxedSystem> is Send because BoxedSystem contains Box<dyn System> where System: Send
// - HashMap<SystemId, usize> is Send + Sync
// - bool is Send + Sync
// SAFETY: BoxedSystem contains Box<dyn System> where System: Send.
// The stage requires &mut self to run, so there's no concurrent access.
unsafe impl Send for SystemStage {}
unsafe impl Sync for SystemStage {}

// ============================================================================
// ParallelSystemStage - Parallel System Container
// ============================================================================

/// A wrapper for raw pointers that implements Send + Sync.
///
/// # Safety
///
/// This is only safe to use when the caller guarantees that:
/// 1. The pointed-to data is not accessed concurrently in conflicting ways
/// 2. The pointer remains valid for the duration of use
///
/// This type is used internally for parallel system execution where access
/// patterns are verified at batch computation time.
struct UnsafePtr<T>(*mut T);

impl<T> UnsafePtr<T> {
    #[inline]
    fn get(&self) -> *mut T {
        self.0
    }
}

impl<T> Clone for UnsafePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UnsafePtr<T> {}

// SAFETY: UnsafePtr is only used in contexts where we've verified
// that concurrent access is safe (non-conflicting component access).
unsafe impl<T> Send for UnsafePtr<T> {}
unsafe impl<T> Sync for UnsafePtr<T> {}

/// Configuration for parallel system execution.
///
/// Controls how systems are grouped and executed in parallel.
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    /// Maximum number of threads to use. 0 means use Rayon's default.
    pub max_threads: usize,
    /// Whether to automatically rebuild parallel groups when systems change.
    pub auto_rebuild: bool,
    /// Whether to respect ordering constraints (may reduce parallelism).
    pub respect_ordering: bool,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_threads: 0, // Use Rayon's default
            auto_rebuild: true,
            respect_ordering: true,
        }
    }
}

impl ParallelExecutionConfig {
    /// Creates a new configuration with the specified maximum threads.
    #[inline]
    pub fn with_max_threads(max_threads: usize) -> Self {
        Self {
            max_threads,
            ..Default::default()
        }
    }

    /// Creates a configuration that ignores ordering constraints for maximum parallelism.
    #[inline]
    pub fn ignore_ordering() -> Self {
        Self {
            respect_ordering: false,
            ..Default::default()
        }
    }
}

/// A batched execution group containing systems that can run in parallel.
///
/// All systems within a batch have no access conflicts and can safely
/// execute concurrently. Batches are executed sequentially.
#[derive(Debug, Clone)]
pub struct ParallelBatch {
    /// System IDs in this batch.
    pub system_ids: Vec<SystemId>,
    /// Whether all systems in this batch are read-only.
    pub all_read_only: bool,
}

impl ParallelBatch {
    /// Creates a new empty batch.
    #[inline]
    pub fn new() -> Self {
        Self {
            system_ids: Vec::new(),
            all_read_only: true,
        }
    }

    /// Creates a batch with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            system_ids: Vec::with_capacity(capacity),
            all_read_only: true,
        }
    }

    /// Adds a system ID to the batch.
    #[inline]
    pub fn add(&mut self, id: SystemId, is_read_only: bool) {
        self.system_ids.push(id);
        if !is_read_only {
            self.all_read_only = false;
        }
    }

    /// Returns the number of systems in this batch.
    #[inline]
    pub fn len(&self) -> usize {
        self.system_ids.len()
    }

    /// Returns true if the batch is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.system_ids.is_empty()
    }

    /// Returns true if the batch can run in parallel (more than 1 system).
    #[inline]
    pub fn can_parallelize(&self) -> bool {
        self.system_ids.len() > 1
    }
}

impl Default for ParallelBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about parallel execution performance.
#[derive(Debug, Clone, Default)]
pub struct ParallelExecutionStats {
    /// Number of batches executed.
    pub batch_count: usize,
    /// Total systems executed.
    pub system_count: usize,
    /// Systems that ran in parallel (batch size > 1).
    pub parallel_systems: usize,
    /// Systems that ran sequentially (batch size = 1).
    pub sequential_systems: usize,
    /// Maximum parallelism achieved (largest batch).
    pub max_parallelism: usize,
}

impl ParallelExecutionStats {
    /// Returns the parallelism ratio (0.0-1.0).
    ///
    /// Higher values indicate more parallel execution.
    #[inline]
    pub fn parallelism_ratio(&self) -> f32 {
        if self.system_count == 0 {
            return 0.0;
        }
        self.parallel_systems as f32 / self.system_count as f32
    }
}

/// A stage that executes non-conflicting systems in parallel.
///
/// `ParallelSystemStage` analyzes system access patterns to determine which
/// systems can safely run concurrently. It groups non-conflicting systems
/// into batches that execute in parallel using Rayon's thread pool.
///
/// # Execution Model
///
/// 1. Systems are analyzed for access conflicts
/// 2. Non-conflicting systems are grouped into parallel batches
/// 3. Batches are executed sequentially (to respect dependencies)
/// 4. Systems within each batch execute in parallel
///
/// # Ordering Constraints
///
/// If `respect_ordering` is enabled (default), ordering constraints are
/// respected by ensuring dependent systems are in later batches. This may
/// reduce parallelism but guarantees correct execution order.
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::{World};
/// use goud_engine::ecs::schedule::ParallelSystemStage;
/// use goud_engine::ecs::system::System;
/// use goud_engine::ecs::query::Access;
/// use goud_engine::ecs::component::ComponentId;
/// use goud_engine::ecs::Component;
///
/// #[derive(Clone, Copy)]
/// struct Position { x: f32, y: f32 }
/// impl Component for Position {}
///
/// #[derive(Clone, Copy)]
/// struct Velocity { x: f32, y: f32 }
/// impl Component for Velocity {}
///
/// // Two systems that don't conflict (different components)
/// struct PositionSystem;
/// impl System for PositionSystem {
///     fn name(&self) -> &'static str { "PositionSystem" }
///     fn component_access(&self) -> Access {
///         let mut access = Access::new();
///         access.add_write(ComponentId::of::<Position>());
///         access
///     }
///     fn run(&mut self, _: &mut World) {}
/// }
///
/// struct VelocitySystem;
/// impl System for VelocitySystem {
///     fn name(&self) -> &'static str { "VelocitySystem" }
///     fn component_access(&self) -> Access {
///         let mut access = Access::new();
///         access.add_write(ComponentId::of::<Velocity>());
///         access
///     }
///     fn run(&mut self, _: &mut World) {}
/// }
///
/// // Create parallel stage
/// let mut stage = ParallelSystemStage::new("Physics");
/// stage.add_system(PositionSystem);
/// stage.add_system(VelocitySystem);
///
/// // These systems can run in parallel since they access different components
/// let mut world = World::new();
/// stage.run(&mut world);
/// ```
///
/// # Thread Safety
///
/// Despite running systems in parallel, `ParallelSystemStage` maintains safety
/// by ensuring no two concurrent systems have conflicting access. The stage
/// uses interior mutability patterns that are safe due to the non-overlapping
/// access guarantees.
pub struct ParallelSystemStage {
    /// Human-readable name of this stage.
    name: String,
    /// Systems to run in this stage.
    systems: Vec<BoxedSystem>,
    /// Map from system ID to index for fast lookup.
    system_indices: HashMap<SystemId, usize>,
    /// Whether the stage has been initialized.
    initialized: bool,
    /// Ordering constraints between systems.
    orderings: Vec<SystemOrdering>,
    /// Whether system order/batches need rebuild.
    dirty: bool,
    /// Parallel execution configuration.
    config: ParallelExecutionConfig,
    /// Pre-computed parallel batches.
    batches: Vec<ParallelBatch>,
    /// Execution statistics from last run.
    last_stats: ParallelExecutionStats,
}

impl ParallelSystemStage {
    /// Creates a new empty parallel stage with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - Human-readable name for debugging
    #[inline]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            systems: Vec::new(),
            system_indices: HashMap::new(),
            initialized: false,
            orderings: Vec::new(),
            dirty: false,
            config: ParallelExecutionConfig::default(),
            batches: Vec::new(),
            last_stats: ParallelExecutionStats::default(),
        }
    }

    /// Creates a new parallel stage with pre-allocated capacity.
    #[inline]
    pub fn with_capacity(name: impl Into<String>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            systems: Vec::with_capacity(capacity),
            system_indices: HashMap::with_capacity(capacity),
            initialized: false,
            orderings: Vec::with_capacity(capacity),
            dirty: false,
            config: ParallelExecutionConfig::default(),
            batches: Vec::new(),
            last_stats: ParallelExecutionStats::default(),
        }
    }

    /// Creates a new parallel stage from a `CoreStage` variant.
    #[inline]
    pub fn from_core(core_stage: CoreStage) -> Self {
        Self::new(core_stage.label_name())
    }

    /// Creates a new parallel stage with custom configuration.
    #[inline]
    pub fn with_config(name: impl Into<String>, config: ParallelExecutionConfig) -> Self {
        let mut stage = Self::new(name);
        stage.config = config;
        stage
    }

    /// Returns a reference to the current configuration.
    #[inline]
    pub fn config(&self) -> &ParallelExecutionConfig {
        &self.config
    }

    /// Returns a mutable reference to the configuration.
    #[inline]
    pub fn config_mut(&mut self) -> &mut ParallelExecutionConfig {
        self.dirty = true; // Config change requires rebuild
        &mut self.config
    }

    /// Sets the parallel execution configuration.
    #[inline]
    pub fn set_config(&mut self, config: ParallelExecutionConfig) {
        self.config = config;
        self.dirty = true;
    }

    /// Returns execution statistics from the last run.
    #[inline]
    pub fn last_stats(&self) -> &ParallelExecutionStats {
        &self.last_stats
    }

    /// Returns the pre-computed batches.
    #[inline]
    pub fn batches(&self) -> &[ParallelBatch] {
        &self.batches
    }

    /// Returns the number of batches.
    #[inline]
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }

    /// Adds a system to this stage.
    ///
    /// Returns the `SystemId` assigned to the system.
    pub fn add_system<S, Marker>(&mut self, system: S) -> SystemId
    where
        S: IntoSystem<Marker>,
    {
        let boxed = system.into_system();
        let id = boxed.id();
        let index = self.systems.len();
        self.system_indices.insert(id, index);
        self.systems.push(boxed);
        self.dirty = true;
        id
    }

    /// Removes a system from this stage by its ID.
    pub fn remove_system(&mut self, id: SystemId) -> bool {
        if let Some(index) = self.system_indices.remove(&id) {
            self.systems.remove(index);
            // Rebuild indices for all systems after the removed one
            self.system_indices.clear();
            for (i, system) in self.systems.iter().enumerate() {
                self.system_indices.insert(system.id(), i);
            }
            self.dirty = true;
            true
        } else {
            false
        }
    }

    /// Returns a reference to a system by its ID.
    #[inline]
    pub fn get_system(&self, id: SystemId) -> Option<&BoxedSystem> {
        self.system_indices.get(&id).map(|&i| &self.systems[i])
    }

    /// Returns a mutable reference to a system by its ID.
    #[inline]
    pub fn get_system_mut(&mut self, id: SystemId) -> Option<&mut BoxedSystem> {
        self.system_indices
            .get(&id)
            .copied()
            .map(|i| &mut self.systems[i])
    }

    /// Returns whether the stage contains a system with the given ID.
    #[inline]
    pub fn contains_system(&self, id: SystemId) -> bool {
        self.system_indices.contains_key(&id)
    }

    /// Returns an iterator over all system IDs in this stage.
    #[inline]
    pub fn system_ids(&self) -> impl Iterator<Item = SystemId> + '_ {
        self.systems.iter().map(|s| s.id())
    }

    /// Returns an iterator over all systems in this stage.
    #[inline]
    pub fn systems(&self) -> impl Iterator<Item = &BoxedSystem> {
        self.systems.iter()
    }

    /// Returns a mutable iterator over all systems in this stage.
    #[inline]
    pub fn systems_mut(&mut self) -> impl Iterator<Item = &mut BoxedSystem> {
        self.systems.iter_mut()
    }

    /// Returns whether this stage has been initialized.
    #[inline]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Marks the stage as uninitialized, forcing re-initialization on next run.
    #[inline]
    pub fn reset_initialized(&mut self) {
        self.initialized = false;
    }

    /// Clears all systems from this stage.
    pub fn clear(&mut self) {
        self.systems.clear();
        self.system_indices.clear();
        self.orderings.clear();
        self.batches.clear();
        self.initialized = false;
        self.dirty = false;
    }

    /// Returns system names for debugging.
    pub fn system_names(&self) -> Vec<&'static str> {
        self.systems.iter().map(|s| s.name()).collect()
    }

    // =========================================================================
    // System Ordering API
    // =========================================================================

    /// Adds an ordering constraint: `first` must run before `second`.
    pub fn add_ordering(&mut self, first: SystemId, second: SystemId) -> bool {
        if !self.contains_system(first) || !self.contains_system(second) {
            return false;
        }
        if first == second {
            return false;
        }
        let ordering = SystemOrdering::before(first, second);
        if self.orderings.contains(&ordering) {
            return true;
        }
        self.orderings.push(ordering);
        self.dirty = true;
        true
    }

    /// Adds a constraint that `system` must run before `other`.
    #[inline]
    pub fn set_before(&mut self, system: SystemId, other: SystemId) -> bool {
        self.add_ordering(system, other)
    }

    /// Adds a constraint that `system` must run after `other`.
    #[inline]
    pub fn set_after(&mut self, system: SystemId, other: SystemId) -> bool {
        self.add_ordering(other, system)
    }

    /// Removes all ordering constraints involving the given system.
    pub fn remove_orderings_for(&mut self, system: SystemId) -> usize {
        let before_len = self.orderings.len();
        self.orderings.retain(|o| !o.involves(system));
        let removed = before_len - self.orderings.len();
        if removed > 0 {
            self.dirty = true;
        }
        removed
    }

    /// Clears all ordering constraints.
    pub fn clear_orderings(&mut self) {
        if !self.orderings.is_empty() {
            self.orderings.clear();
            self.dirty = true;
        }
    }

    /// Returns an iterator over all ordering constraints.
    #[inline]
    pub fn orderings(&self) -> impl Iterator<Item = &SystemOrdering> {
        self.orderings.iter()
    }

    /// Returns the number of ordering constraints.
    #[inline]
    pub fn ordering_count(&self) -> usize {
        self.orderings.len()
    }

    // =========================================================================
    // Batch Computation
    // =========================================================================

    /// Rebuilds the parallel execution batches.
    ///
    /// This analyzes system access patterns and ordering constraints to
    /// determine optimal parallel groupings.
    ///
    /// # Algorithm
    ///
    /// 1. If ordering constraints exist and `respect_ordering` is enabled,
    ///    first perform topological sort
    /// 2. Process systems in sorted order, assigning each to a batch
    /// 3. A system can join a batch if:
    ///    - It doesn't conflict with any system already in the batch
    ///    - No ordering constraint requires it to run after a system in the batch
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(OrderingCycleError)` if ordering constraints form a cycle
    pub fn rebuild_batches(&mut self) -> Result<(), OrderingCycleError> {
        self.batches.clear();

        if self.systems.is_empty() {
            self.dirty = false;
            return Ok(());
        }

        // Get the execution order (may be sorted by dependencies)
        let execution_order = if self.config.respect_ordering && !self.orderings.is_empty() {
            // Use topological sort
            let mut sorter =
                TopologicalSorter::with_capacity(self.systems.len(), self.orderings.len());
            for system in &self.systems {
                sorter.add_system(system.id(), system.name());
            }
            for ordering in &self.orderings {
                sorter.add_system_ordering(*ordering);
            }
            sorter.sort()?
        } else {
            // Use natural order
            self.systems.iter().map(|s| s.id()).collect()
        };

        // Build ordering edges for quick lookup (system -> its direct predecessors)
        let mut direct_predecessors: HashMap<SystemId, Vec<SystemId>> = HashMap::new();
        if self.config.respect_ordering {
            for ordering in &self.orderings {
                direct_predecessors
                    .entry(ordering.second())
                    .or_default()
                    .push(ordering.first());
            }
        }

        // Track which batch each system is assigned to
        let mut system_batch_index: HashMap<SystemId, usize> = HashMap::new();

        // Build batches
        for system_id in execution_order {
            let system_idx = self.system_indices[&system_id];
            let system = &self.systems[system_idx];
            let system_access = system.component_access();
            let system_read_only = system.is_read_only();

            // Find the minimum batch index this system can be placed in.
            // It must be placed AFTER all its predecessors (direct and transitive).
            // Since we process in topological order, we only need to check direct
            // predecessors - their batch indices already account for their predecessors.
            let min_batch_idx = if self.config.respect_ordering {
                direct_predecessors
                    .get(&system_id)
                    .map(|preds| {
                        preds
                            .iter()
                            .filter_map(|pred| system_batch_index.get(pred))
                            .max()
                            .map(|&max_pred_batch| max_pred_batch + 1)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0)
            } else {
                0
            };

            // Try to find an existing batch (starting from min_batch_idx) this system can join
            let mut assigned = false;
            for batch_idx in min_batch_idx..self.batches.len() {
                let batch = &self.batches[batch_idx];

                // Check for access conflicts with all systems in the batch
                let has_conflict = batch.system_ids.iter().any(|&batch_id| {
                    let batch_idx = self.system_indices[&batch_id];
                    system_access.conflicts_with(&self.systems[batch_idx].component_access())
                });

                if !has_conflict {
                    // Can add to this batch
                    self.batches[batch_idx].add(system_id, system_read_only);
                    system_batch_index.insert(system_id, batch_idx);
                    assigned = true;
                    break;
                }
            }

            if !assigned {
                // Create new batch (it will be at index self.batches.len())
                let new_batch_idx = self.batches.len();
                let mut batch = ParallelBatch::with_capacity(4);
                batch.add(system_id, system_read_only);
                self.batches.push(batch);
                system_batch_index.insert(system_id, new_batch_idx);
            }
        }

        self.dirty = false;
        Ok(())
    }

    /// Forces a rebuild of parallel batches on next run.
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Returns whether batches need to be rebuilt.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    // =========================================================================
    // Parallel Execution
    // =========================================================================

    /// Runs all systems using parallel execution.
    ///
    /// Non-conflicting systems execute in parallel within their batch.
    /// Batches are executed sequentially to respect ordering constraints.
    ///
    /// # Safety
    ///
    /// This method uses unsafe code to enable parallel execution while
    /// maintaining memory safety. Safety is guaranteed because:
    ///
    /// 1. Batch computation ensures no two concurrent systems have conflicting access
    /// 2. Each system has exclusive access to its accessed components/resources
    /// 3. The World is not mutably borrowed during parallel execution
    pub fn run_parallel(&mut self, world: &mut World) {
        // Rebuild batches if needed
        if (self.dirty || self.batches.is_empty()) && self.config.auto_rebuild {
            let _ = self.rebuild_batches();
        }

        // Initialize if needed
        if !self.initialized {
            for system in &mut self.systems {
                system.initialize(world);
            }
            self.initialized = true;
        }

        // Reset stats
        let mut stats = ParallelExecutionStats {
            batch_count: self.batches.len(),
            system_count: self.systems.len(),
            ..Default::default()
        };

        // Execute each batch
        for batch in &self.batches {
            if batch.is_empty() {
                continue;
            }

            // Collect systems that should run
            let runnable: Vec<usize> = batch
                .system_ids
                .iter()
                .filter_map(|&id| {
                    let idx = self.system_indices[&id];
                    if self.systems[idx].should_run(world) {
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect();

            if runnable.is_empty() {
                continue;
            }

            if runnable.len() == 1 {
                // Single system - run directly without Rayon overhead
                self.systems[runnable[0]].run(world);
                stats.sequential_systems += 1;
            } else {
                // Multiple systems - run in parallel
                stats.parallel_systems += runnable.len();
                if runnable.len() > stats.max_parallelism {
                    stats.max_parallelism = runnable.len();
                }

                // SAFETY: We ensure through batch computation that systems in the same
                // batch do not have conflicting access patterns. Each system accesses
                // disjoint data, making concurrent mutation safe.
                //
                // We use raw pointers wrapped in UnsafePtr to satisfy Send bounds.
                // The safety is guaranteed by the non-overlapping access patterns
                // verified during batch construction.
                let systems_ptr = UnsafePtr(self.systems.as_mut_ptr());
                let world_ptr = UnsafePtr(world as *mut World);

                rayon::scope(|s| {
                    for &idx in &runnable {
                        s.spawn(move |_| {
                            // SAFETY: See above - each system in the batch accesses disjoint data
                            unsafe {
                                let system = &mut *systems_ptr.get().add(idx);
                                let world_ref = &mut *world_ptr.get();
                                system.run(world_ref);
                            }
                        });
                    }
                });
            }
        }

        self.last_stats = stats;
    }

    // =========================================================================
    // Conflict Detection (delegated to similar API as SystemStage)
    // =========================================================================

    /// Checks if any systems in this stage have conflicting access patterns.
    pub fn has_conflicts(&self) -> bool {
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                if self.systems[i].conflicts_with(&self.systems[j]) {
                    return true;
                }
            }
        }
        false
    }

    /// Finds all conflicting system pairs.
    pub fn find_conflicts(&self) -> Vec<SystemConflict> {
        let mut conflicts = Vec::new();
        for i in 0..self.systems.len() {
            for j in (i + 1)..self.systems.len() {
                let access_i = self.systems[i].component_access();
                let access_j = self.systems[j].component_access();
                if let Some(access_conflict) = access_i.get_conflicts(&access_j) {
                    conflicts.push(SystemConflict {
                        first_system_id: self.systems[i].id(),
                        first_system_name: self.systems[i].name(),
                        second_system_id: self.systems[j].id(),
                        second_system_name: self.systems[j].name(),
                        conflict: access_conflict,
                    });
                }
            }
        }
        conflicts
    }

    /// Returns all systems that are read-only.
    pub fn read_only_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| s.is_read_only())
            .map(|s| s.id())
            .collect()
    }

    /// Returns all systems that write.
    pub fn writing_systems(&self) -> Vec<SystemId> {
        self.systems
            .iter()
            .filter(|s| !s.is_read_only())
            .map(|s| s.id())
            .collect()
    }
}

impl Stage for ParallelSystemStage {
    #[inline]
    fn name(&self) -> &str {
        &self.name
    }

    fn run(&mut self, world: &mut World) {
        self.run_parallel(world);
    }

    fn initialize(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.initialize(world);
        }
        self.initialized = true;
    }

    #[inline]
    fn system_count(&self) -> usize {
        self.systems.len()
    }
}

impl Default for ParallelSystemStage {
    fn default() -> Self {
        Self::new("ParallelStage")
    }
}

impl fmt::Debug for ParallelSystemStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParallelSystemStage")
            .field("name", &self.name)
            .field("system_count", &self.systems.len())
            .field("batch_count", &self.batches.len())
            .field("initialized", &self.initialized)
            .field("dirty", &self.dirty)
            .field("config", &self.config)
            .field("systems", &self.system_names())
            .finish()
    }
}

// SAFETY: ParallelSystemStage follows the same safety reasoning as SystemStage.
// Additionally, parallel execution is safe because batch computation ensures
// non-conflicting access patterns between concurrent systems.
unsafe impl Send for ParallelSystemStage {}
unsafe impl Sync for ParallelSystemStage {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::collections::HashSet;

    // ========================================================================
    // CoreStage Tests
    // ========================================================================

    mod core_stage {
        use super::*;

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
        fn test_clone() {
            let stage = CoreStage::Update;
            let cloned = stage;
            assert_eq!(stage, cloned);
        }

        #[test]
        fn test_copy() {
            let stage = CoreStage::Update;
            let copied: CoreStage = stage;
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

            // Test sorting
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
            set.insert(CoreStage::Update); // Duplicate

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
            // Starting from PreUpdate, navigate through all stages
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
            // Starting from PostRender, navigate backwards
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

    // ========================================================================
    // StageLabel Tests
    // ========================================================================

    mod stage_label {
        use super::*;

        #[test]
        fn test_core_stage_label_id() {
            // Each variant should have a unique label_id
            let ids: Vec<_> = CoreStage::all().iter().map(|s| s.label_id()).collect();

            // All should be unique
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

            // Different from CoreStage
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
    }

    // ========================================================================
    // StageLabelId Tests
    // ========================================================================

    mod stage_label_id {
        use super::*;

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

    // ========================================================================
    // StagePosition Tests
    // ========================================================================

    mod stage_position {
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

    // ========================================================================
    // StageOrder Tests
    // ========================================================================

    mod stage_order {
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

    // ========================================================================
    // Thread Safety Tests
    // ========================================================================

    mod thread_safety {
        use super::*;

        #[test]
        fn test_core_stage_send() {
            fn assert_send<T: Send>() {}
            assert_send::<CoreStage>();
        }

        #[test]
        fn test_core_stage_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<CoreStage>();
        }

        #[test]
        fn test_stage_label_id_send() {
            fn assert_send<T: Send>() {}
            assert_send::<StageLabelId>();
        }

        #[test]
        fn test_stage_label_id_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<StageLabelId>();
        }

        #[test]
        fn test_stage_position_send() {
            fn assert_send<T: Send>() {}
            assert_send::<StagePosition>();
        }

        #[test]
        fn test_stage_position_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<StagePosition>();
        }
    }

    // ========================================================================
    // Integration Tests
    // ========================================================================

    mod integration {
        use super::*;

        #[test]
        fn test_stage_label_in_hashmap() {
            let mut stages: HashMap<StageLabelId, Vec<&str>> = HashMap::new();

            stages.insert(CoreStage::PreUpdate.into(), vec!["input_system"]);
            stages.insert(CoreStage::Update.into(), vec!["movement", "ai"]);
            stages.insert(CoreStage::Render.into(), vec!["sprite_render"]);

            assert_eq!(stages.len(), 3);

            let update_systems = stages.get(&CoreStage::Update.into()).unwrap();
            assert_eq!(update_systems, &vec!["movement", "ai"]);
        }

        #[test]
        fn test_custom_and_core_stages_together() {
            #[derive(Debug, Clone, Copy)]
            struct NetworkStage;

            impl StageLabel for NetworkStage {
                fn label_id(&self) -> TypeId {
                    TypeId::of::<Self>()
                }

                fn label_name(&self) -> &'static str {
                    "NetworkStage"
                }

                fn dyn_clone(&self) -> Box<dyn StageLabel> {
                    Box::new(*self)
                }
            }

            let stages: Vec<Box<dyn StageLabel>> = vec![
                Box::new(CoreStage::PreUpdate),
                Box::new(NetworkStage),
                Box::new(CoreStage::Update),
            ];

            assert_eq!(stages.len(), 3);
            assert_eq!(stages[0].label_name(), "PreUpdate");
            assert_eq!(stages[1].label_name(), "NetworkStage");
            assert_eq!(stages[2].label_name(), "Update");

            // Ensure they're distinguishable
            assert_ne!(stages[0].label_id(), stages[1].label_id());
            assert_ne!(stages[1].label_id(), stages[2].label_id());
        }

        #[test]
        fn test_stage_iteration_order() {
            // Simulate what a scheduler might do
            let execution_order: Vec<CoreStage> = CoreStage::all().to_vec();

            for window in execution_order.windows(2) {
                let current = window[0];
                let next = window[1];

                // Verify ordering is correct
                assert!(current < next);
                assert_eq!(current.next(), Some(next));
                assert_eq!(next.previous(), Some(current));
            }
        }

        #[test]
        fn test_stage_filtering() {
            // Filter stages by category
            let logic_stages: Vec<_> = CoreStage::all()
                .into_iter()
                .filter(|s| s.is_logic())
                .collect();
            assert_eq!(logic_stages.len(), 3);
            assert!(logic_stages.contains(&CoreStage::PreUpdate));
            assert!(logic_stages.contains(&CoreStage::Update));
            assert!(logic_stages.contains(&CoreStage::PostUpdate));

            let render_stages: Vec<_> = CoreStage::all()
                .into_iter()
                .filter(|s| s.is_render())
                .collect();
            assert_eq!(render_stages.len(), 3);
            assert!(render_stages.contains(&CoreStage::PreRender));
            assert!(render_stages.contains(&CoreStage::Render));
            assert!(render_stages.contains(&CoreStage::PostRender));
        }
    }

    // ========================================================================
    // Stage Trait Tests
    // ========================================================================

    mod stage_trait {
        use super::*;

        struct EmptyStage {
            name: String,
            run_count: u32,
        }

        impl Stage for EmptyStage {
            fn name(&self) -> &str {
                &self.name
            }

            fn run(&mut self, _world: &mut World) {
                self.run_count += 1;
            }

            fn system_count(&self) -> usize {
                0
            }
        }

        #[test]
        fn test_stage_trait_name() {
            let stage = EmptyStage {
                name: "TestStage".to_string(),
                run_count: 0,
            };
            assert_eq!(stage.name(), "TestStage");
        }

        #[test]
        fn test_stage_trait_run() {
            let mut stage = EmptyStage {
                name: "TestStage".to_string(),
                run_count: 0,
            };
            let mut world = World::new();

            assert_eq!(stage.run_count, 0);
            stage.run(&mut world);
            assert_eq!(stage.run_count, 1);
            stage.run(&mut world);
            assert_eq!(stage.run_count, 2);
        }

        #[test]
        fn test_stage_trait_is_empty() {
            let stage = EmptyStage {
                name: "TestStage".to_string(),
                run_count: 0,
            };
            assert!(stage.is_empty());
            assert_eq!(stage.system_count(), 0);
        }

        #[test]
        fn test_stage_trait_send_sync() {
            fn assert_send_sync<T: Send + Sync>() {}
            // Stage trait requires Send + Sync
            assert_send_sync::<EmptyStage>();
        }
    }

    // ========================================================================
    // SystemStage Tests
    // ========================================================================

    mod system_stage {
        use super::*;
        use crate::ecs::system::System;

        // Helper system that tracks execution
        struct CounterSystem {
            name: &'static str,
            run_count: std::sync::Arc<std::sync::atomic::AtomicU32>,
        }

        impl System for CounterSystem {
            fn name(&self) -> &'static str {
                self.name
            }

            fn run(&mut self, _world: &mut World) {
                self.run_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }

        // Simple system without shared state
        struct SimpleSystem {
            name: &'static str,
        }

        impl System for SimpleSystem {
            fn name(&self) -> &'static str {
                self.name
            }

            fn run(&mut self, _world: &mut World) {}
        }

        // System that spawns entities
        struct SpawnSystem;

        impl System for SpawnSystem {
            fn name(&self) -> &'static str {
                "SpawnSystem"
            }

            fn run(&mut self, world: &mut World) {
                world.spawn_empty();
            }
        }

        // Conditional system
        struct ConditionalSystem {
            should_run: bool,
            ran: std::sync::Arc<std::sync::atomic::AtomicBool>,
        }

        impl System for ConditionalSystem {
            fn name(&self) -> &'static str {
                "ConditionalSystem"
            }

            fn should_run(&self, _world: &World) -> bool {
                self.should_run
            }

            fn run(&mut self, _world: &mut World) {
                self.ran.store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }

        // ====================================================================
        // Construction Tests
        // ====================================================================

        #[test]
        fn test_new() {
            let stage = SystemStage::new("Update");
            assert_eq!(stage.name(), "Update");
            assert!(stage.is_empty());
            assert_eq!(stage.system_count(), 0);
            assert!(!stage.is_initialized());
        }

        #[test]
        fn test_new_with_string() {
            let name = String::from("DynamicName");
            let stage = SystemStage::new(name);
            assert_eq!(stage.name(), "DynamicName");
        }

        #[test]
        fn test_with_capacity() {
            let stage = SystemStage::with_capacity("Physics", 10);
            assert_eq!(stage.name(), "Physics");
            assert!(stage.is_empty());
        }

        #[test]
        fn test_from_core() {
            let stage = SystemStage::from_core(CoreStage::Update);
            assert_eq!(stage.name(), "Update");

            let stage = SystemStage::from_core(CoreStage::PreRender);
            assert_eq!(stage.name(), "PreRender");
        }

        #[test]
        fn test_default() {
            let stage = SystemStage::default();
            assert_eq!(stage.name(), "DefaultStage");
            assert!(stage.is_empty());
        }

        // ====================================================================
        // System Management Tests
        // ====================================================================

        #[test]
        fn test_add_system() {
            let mut stage = SystemStage::new("Update");
            let id = stage.add_system(SimpleSystem { name: "SystemA" });

            assert!(id.is_valid());
            assert_eq!(stage.system_count(), 1);
            assert!(!stage.is_empty());
        }

        #[test]
        fn test_add_multiple_systems() {
            let mut stage = SystemStage::new("Update");
            let id1 = stage.add_system(SimpleSystem { name: "SystemA" });
            let id2 = stage.add_system(SimpleSystem { name: "SystemB" });
            let id3 = stage.add_system(SimpleSystem { name: "SystemC" });

            assert_eq!(stage.system_count(), 3);
            assert_ne!(id1, id2);
            assert_ne!(id2, id3);
            assert_ne!(id1, id3);
        }

        #[test]
        fn test_remove_system() {
            let mut stage = SystemStage::new("Update");
            let id = stage.add_system(SimpleSystem { name: "SystemA" });

            assert!(stage.contains_system(id));
            assert!(stage.remove_system(id));
            assert!(!stage.contains_system(id));
            assert_eq!(stage.system_count(), 0);
        }

        #[test]
        fn test_remove_nonexistent_system() {
            let mut stage = SystemStage::new("Update");
            let id = SystemId::new(); // ID not in stage

            assert!(!stage.remove_system(id));
        }

        #[test]
        fn test_remove_twice() {
            let mut stage = SystemStage::new("Update");
            let id = stage.add_system(SimpleSystem { name: "SystemA" });

            assert!(stage.remove_system(id));
            assert!(!stage.remove_system(id)); // Second removal fails
        }

        #[test]
        fn test_remove_middle_system() {
            let mut stage = SystemStage::new("Update");
            let id1 = stage.add_system(SimpleSystem { name: "SystemA" });
            let id2 = stage.add_system(SimpleSystem { name: "SystemB" });
            let id3 = stage.add_system(SimpleSystem { name: "SystemC" });

            assert!(stage.remove_system(id2));
            assert_eq!(stage.system_count(), 2);

            // Verify remaining systems are still accessible
            assert!(stage.contains_system(id1));
            assert!(!stage.contains_system(id2));
            assert!(stage.contains_system(id3));
        }

        #[test]
        fn test_get_system() {
            let mut stage = SystemStage::new("Update");
            let id = stage.add_system(SimpleSystem { name: "MySystem" });

            let system = stage.get_system(id);
            assert!(system.is_some());
            assert_eq!(system.unwrap().name(), "MySystem");
        }

        #[test]
        fn test_get_system_not_found() {
            let stage = SystemStage::new("Update");
            let id = SystemId::new();

            assert!(stage.get_system(id).is_none());
        }

        #[test]
        fn test_get_system_mut() {
            let mut stage = SystemStage::new("Update");
            let id = stage.add_system(SimpleSystem { name: "MySystem" });

            let system = stage.get_system_mut(id);
            assert!(system.is_some());
        }

        #[test]
        fn test_contains_system() {
            let mut stage = SystemStage::new("Update");
            let id1 = stage.add_system(SimpleSystem { name: "SystemA" });
            let id2 = SystemId::new();

            assert!(stage.contains_system(id1));
            assert!(!stage.contains_system(id2));
        }

        #[test]
        fn test_system_ids() {
            let mut stage = SystemStage::new("Update");
            let id1 = stage.add_system(SimpleSystem { name: "SystemA" });
            let id2 = stage.add_system(SimpleSystem { name: "SystemB" });

            let ids: Vec<_> = stage.system_ids().collect();
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&id1));
            assert!(ids.contains(&id2));
        }

        #[test]
        fn test_systems_iterator() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });
            stage.add_system(SimpleSystem { name: "SystemB" });

            let names: Vec<_> = stage.systems().map(|s| s.name()).collect();
            assert_eq!(names, vec!["SystemA", "SystemB"]);
        }

        #[test]
        fn test_system_names() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });
            stage.add_system(SimpleSystem { name: "SystemB" });

            let names = stage.system_names();
            assert_eq!(names, vec!["SystemA", "SystemB"]);
        }

        #[test]
        fn test_clear() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });
            stage.add_system(SimpleSystem { name: "SystemB" });

            assert_eq!(stage.system_count(), 2);
            stage.clear();
            assert_eq!(stage.system_count(), 0);
            assert!(stage.is_empty());
        }

        // ====================================================================
        // Execution Tests
        // ====================================================================

        #[test]
        fn test_run_empty_stage() {
            let mut stage = SystemStage::new("Update");
            let mut world = World::new();

            // Should not panic
            stage.run(&mut world);
            assert!(stage.is_initialized());
        }

        #[test]
        fn test_run_single_system() {
            use std::sync::atomic::{AtomicU32, Ordering};
            use std::sync::Arc;

            let counter = Arc::new(AtomicU32::new(0));
            let mut stage = SystemStage::new("Update");
            stage.add_system(CounterSystem {
                name: "Counter",
                run_count: counter.clone(),
            });

            let mut world = World::new();

            assert_eq!(counter.load(Ordering::SeqCst), 0);
            stage.run(&mut world);
            assert_eq!(counter.load(Ordering::SeqCst), 1);
            stage.run(&mut world);
            assert_eq!(counter.load(Ordering::SeqCst), 2);
        }

        #[test]
        fn test_run_multiple_systems() {
            use std::sync::atomic::{AtomicU32, Ordering};
            use std::sync::Arc;

            let counter_a = Arc::new(AtomicU32::new(0));
            let counter_b = Arc::new(AtomicU32::new(0));

            let mut stage = SystemStage::new("Update");
            stage.add_system(CounterSystem {
                name: "CounterA",
                run_count: counter_a.clone(),
            });
            stage.add_system(CounterSystem {
                name: "CounterB",
                run_count: counter_b.clone(),
            });

            let mut world = World::new();
            stage.run(&mut world);

            assert_eq!(counter_a.load(Ordering::SeqCst), 1);
            assert_eq!(counter_b.load(Ordering::SeqCst), 1);
        }

        #[test]
        fn test_run_system_modifies_world() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SpawnSystem);

            let mut world = World::new();

            assert_eq!(world.entity_count(), 0);
            stage.run(&mut world);
            assert_eq!(world.entity_count(), 1);
            stage.run(&mut world);
            assert_eq!(world.entity_count(), 2);
        }

        #[test]
        fn test_run_respects_should_run() {
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::sync::Arc;

            let ran_yes = Arc::new(AtomicBool::new(false));
            let ran_no = Arc::new(AtomicBool::new(false));

            let mut stage = SystemStage::new("Update");
            stage.add_system(ConditionalSystem {
                should_run: true,
                ran: ran_yes.clone(),
            });
            stage.add_system(ConditionalSystem {
                should_run: false,
                ran: ran_no.clone(),
            });

            let mut world = World::new();
            stage.run(&mut world);

            assert!(ran_yes.load(Ordering::SeqCst));
            assert!(!ran_no.load(Ordering::SeqCst));
        }

        #[test]
        fn test_run_single_system_by_id() {
            use std::sync::atomic::{AtomicU32, Ordering};
            use std::sync::Arc;

            let counter_a = Arc::new(AtomicU32::new(0));
            let counter_b = Arc::new(AtomicU32::new(0));

            let mut stage = SystemStage::new("Update");
            let id_a = stage.add_system(CounterSystem {
                name: "CounterA",
                run_count: counter_a.clone(),
            });
            stage.add_system(CounterSystem {
                name: "CounterB",
                run_count: counter_b.clone(),
            });

            let mut world = World::new();

            // Run only system A
            let result = stage.run_system(id_a, &mut world);
            assert_eq!(result, Some(true));

            assert_eq!(counter_a.load(Ordering::SeqCst), 1);
            assert_eq!(counter_b.load(Ordering::SeqCst), 0);
        }

        #[test]
        fn test_run_system_not_found() {
            let mut stage = SystemStage::new("Update");
            let mut world = World::new();
            let unknown_id = SystemId::new();

            let result = stage.run_system(unknown_id, &mut world);
            assert_eq!(result, None);
        }

        #[test]
        fn test_run_system_skipped() {
            use std::sync::atomic::{AtomicBool, Ordering};
            use std::sync::Arc;

            let ran = Arc::new(AtomicBool::new(false));
            let mut stage = SystemStage::new("Update");
            let id = stage.add_system(ConditionalSystem {
                should_run: false,
                ran: ran.clone(),
            });

            let mut world = World::new();
            let result = stage.run_system(id, &mut world);

            assert_eq!(result, Some(false)); // Skipped
            assert!(!ran.load(Ordering::SeqCst));
        }

        // ====================================================================
        // Initialization Tests
        // ====================================================================

        #[test]
        fn test_initialization_happens_on_first_run() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });

            let mut world = World::new();

            assert!(!stage.is_initialized());
            stage.run(&mut world);
            assert!(stage.is_initialized());
        }

        #[test]
        fn test_initialization_only_once() {
            // If run is called multiple times, initialization should only happen once
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });

            let mut world = World::new();

            stage.run(&mut world);
            assert!(stage.is_initialized());

            stage.run(&mut world);
            assert!(stage.is_initialized());
        }

        #[test]
        fn test_reset_initialized() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });

            let mut world = World::new();
            stage.run(&mut world);

            assert!(stage.is_initialized());
            stage.reset_initialized();
            assert!(!stage.is_initialized());
        }

        #[test]
        fn test_clear_resets_initialized() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });

            let mut world = World::new();
            stage.run(&mut world);

            assert!(stage.is_initialized());
            stage.clear();
            assert!(!stage.is_initialized());
        }

        // ====================================================================
        // Trait Implementation Tests
        // ====================================================================

        #[test]
        fn test_stage_trait_implementation() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });

            // Use through trait
            let stage_ref: &dyn Stage = &stage;
            assert_eq!(stage_ref.name(), "Update");
            assert_eq!(stage_ref.system_count(), 1);
            assert!(!stage_ref.is_empty());
        }

        #[test]
        fn test_debug() {
            let mut stage = SystemStage::new("Update");
            stage.add_system(SimpleSystem { name: "SystemA" });
            stage.add_system(SimpleSystem { name: "SystemB" });

            let debug = format!("{:?}", stage);
            assert!(debug.contains("SystemStage"));
            assert!(debug.contains("Update"));
            assert!(debug.contains("SystemA"));
            assert!(debug.contains("SystemB"));
        }

        // ====================================================================
        // Thread Safety Tests
        // ====================================================================

        #[test]
        fn test_system_stage_send() {
            fn assert_send<T: Send>() {}
            assert_send::<SystemStage>();
        }

        #[test]
        fn test_system_stage_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<SystemStage>();
        }

        // ====================================================================
        // Edge Cases
        // ====================================================================

        #[test]
        fn test_add_many_systems() {
            let mut stage = SystemStage::new("Update");

            for i in 0..100 {
                // Use a struct that can have a dynamic name via field
                struct NumberedSystem(usize);
                impl System for NumberedSystem {
                    fn name(&self) -> &'static str {
                        "NumberedSystem"
                    }
                    fn run(&mut self, _world: &mut World) {}
                }
                stage.add_system(NumberedSystem(i));
            }

            assert_eq!(stage.system_count(), 100);
        }

        #[test]
        fn test_systems_run_in_order() {
            use std::sync::atomic::{AtomicUsize, Ordering};
            use std::sync::Arc;

            // Track execution order
            let order = Arc::new(AtomicUsize::new(0));

            struct OrderedSystem {
                expected_order: usize,
                order: Arc<AtomicUsize>,
                success: Arc<std::sync::atomic::AtomicBool>,
            }

            impl System for OrderedSystem {
                fn name(&self) -> &'static str {
                    "OrderedSystem"
                }

                fn run(&mut self, _world: &mut World) {
                    let current = self.order.fetch_add(1, Ordering::SeqCst);
                    if current == self.expected_order {
                        self.success.store(true, Ordering::SeqCst);
                    }
                }
            }

            let successes: Vec<_> = (0..5)
                .map(|_| Arc::new(std::sync::atomic::AtomicBool::new(false)))
                .collect();

            let mut stage = SystemStage::new("Update");
            for (i, success) in successes.iter().enumerate() {
                stage.add_system(OrderedSystem {
                    expected_order: i,
                    order: order.clone(),
                    success: success.clone(),
                });
            }

            let mut world = World::new();
            stage.run(&mut world);

            // All systems should have run in correct order
            for success in &successes {
                assert!(success.load(Ordering::SeqCst));
            }
        }

        #[test]
        fn test_boxed_stage() {
            let stage = SystemStage::new("Update");

            // Can be boxed as trait object
            let _boxed: Box<dyn Stage> = Box::new(stage);
        }

        #[test]
        fn test_multiple_stages() {
            let mut pre_update = SystemStage::from_core(CoreStage::PreUpdate);
            let mut update = SystemStage::from_core(CoreStage::Update);
            let mut post_update = SystemStage::from_core(CoreStage::PostUpdate);

            pre_update.add_system(SimpleSystem { name: "Input" });
            update.add_system(SimpleSystem { name: "Physics" });
            update.add_system(SimpleSystem { name: "AI" });
            post_update.add_system(SimpleSystem { name: "Cleanup" });

            assert_eq!(pre_update.system_count(), 1);
            assert_eq!(update.system_count(), 2);
            assert_eq!(post_update.system_count(), 1);

            let mut world = World::new();
            pre_update.run(&mut world);
            update.run(&mut world);
            post_update.run(&mut world);
        }
    }

    // ========================================================================
    // Conflict Detection Tests
    // ========================================================================

    mod conflict_detection {
        use super::*;
        use crate::ecs::component::ComponentId;
        use crate::ecs::query::Access;
        use crate::ecs::system::System;
        use crate::ecs::Component;

        // Test components
        #[derive(Debug, Clone, Copy)]
        struct Position {
            x: f32,
            y: f32,
        }
        impl Component for Position {}

        #[derive(Debug, Clone, Copy)]
        struct Velocity {
            x: f32,
            y: f32,
        }
        impl Component for Velocity {}

        #[derive(Debug, Clone, Copy)]
        struct Health(f32);
        impl Component for Health {}

        // Test systems
        struct PositionWriter;
        impl System for PositionWriter {
            fn name(&self) -> &'static str {
                "PositionWriter"
            }
            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_write(ComponentId::of::<Position>());
                access
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct PositionReader;
        impl System for PositionReader {
            fn name(&self) -> &'static str {
                "PositionReader"
            }
            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_read(ComponentId::of::<Position>());
                access
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct VelocityWriter;
        impl System for VelocityWriter {
            fn name(&self) -> &'static str {
                "VelocityWriter"
            }
            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_write(ComponentId::of::<Velocity>());
                access
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct VelocityReader;
        impl System for VelocityReader {
            fn name(&self) -> &'static str {
                "VelocityReader"
            }
            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_read(ComponentId::of::<Velocity>());
                access
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct MovementSystem;
        impl System for MovementSystem {
            fn name(&self) -> &'static str {
                "MovementSystem"
            }
            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_write(ComponentId::of::<Position>());
                access.add_read(ComponentId::of::<Velocity>());
                access
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct HealthWriter;
        impl System for HealthWriter {
            fn name(&self) -> &'static str {
                "HealthWriter"
            }
            fn component_access(&self) -> Access {
                let mut access = Access::new();
                access.add_write(ComponentId::of::<Health>());
                access
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct NoAccessSystem;
        impl System for NoAccessSystem {
            fn name(&self) -> &'static str {
                "NoAccessSystem"
            }
            fn run(&mut self, _: &mut World) {}
        }

        // =====================================================================
        // has_conflicts tests
        // =====================================================================

        #[test]
        fn test_has_conflicts_empty_stage() {
            let stage = SystemStage::new("Test");
            assert!(!stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_single_system() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            assert!(!stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_no_conflict_different_components() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter);
            assert!(!stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_no_conflict_both_readers() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionReader);
            stage.add_system(PositionReader); // Two readers is fine
            assert!(!stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_write_read() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionReader);
            assert!(stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_write_write() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionWriter);
            assert!(stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_movement_and_position_reader() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(MovementSystem); // Writes Position, reads Velocity
            stage.add_system(PositionReader); // Reads Position
            assert!(stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_movement_and_velocity_writer() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(MovementSystem); // Writes Position, reads Velocity
            stage.add_system(VelocityWriter); // Writes Velocity
            assert!(stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_three_systems_partial() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter); // No conflict with PositionWriter
            stage.add_system(PositionReader); // Conflicts with PositionWriter
            assert!(stage.has_conflicts());
        }

        #[test]
        fn test_has_conflicts_no_access_systems() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(NoAccessSystem);
            stage.add_system(NoAccessSystem);
            stage.add_system(PositionWriter);
            assert!(!stage.has_conflicts()); // NoAccessSystem doesn't conflict
        }

        // =====================================================================
        // find_conflicts tests
        // =====================================================================

        #[test]
        fn test_find_conflicts_empty() {
            let stage = SystemStage::new("Test");
            let conflicts = stage.find_conflicts();
            assert!(conflicts.is_empty());
        }

        #[test]
        fn test_find_conflicts_none() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter);
            let conflicts = stage.find_conflicts();
            assert!(conflicts.is_empty());
        }

        #[test]
        fn test_find_conflicts_one() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionReader);

            let conflicts = stage.find_conflicts();
            assert_eq!(conflicts.len(), 1);

            let conflict = &conflicts[0];
            assert_eq!(conflict.first_system_name, "PositionWriter");
            assert_eq!(conflict.second_system_name, "PositionReader");
            assert_eq!(conflict.component_conflict_count(), 1);
        }

        #[test]
        fn test_find_conflicts_multiple() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter);
            stage.add_system(VelocityWriter);

            let conflicts = stage.find_conflicts();
            // Two Position writers conflict, two Velocity writers conflict
            assert_eq!(conflicts.len(), 2);
        }

        #[test]
        fn test_find_conflicts_chain() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(MovementSystem); // Writes Position, reads Velocity
            stage.add_system(PositionReader); // Reads Position - conflicts
            stage.add_system(VelocityWriter); // Writes Velocity - conflicts

            let conflicts = stage.find_conflicts();
            // Movement conflicts with both Reader and Writer
            assert_eq!(conflicts.len(), 2);
        }

        #[test]
        fn test_find_conflicts_details() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionWriter);

            let conflicts = stage.find_conflicts();
            assert_eq!(conflicts.len(), 1);

            let conflict = &conflicts[0];
            assert!(conflict.is_write_write());
            assert_eq!(conflict.total_conflict_count(), 1);

            // Check that we can iterate over conflicting components
            let comp_ids: Vec<_> = conflict.conflict.conflicting_components().collect();
            assert_eq!(comp_ids.len(), 1);
            assert_eq!(comp_ids[0], ComponentId::of::<Position>());
        }

        // =====================================================================
        // find_conflicts_for_system tests
        // =====================================================================

        #[test]
        fn test_find_conflicts_for_system_none() {
            let mut stage = SystemStage::new("Test");
            let id = stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter);

            let conflicts = stage.find_conflicts_for_system(id);
            assert!(conflicts.is_empty());
        }

        #[test]
        fn test_find_conflicts_for_system_found() {
            let mut stage = SystemStage::new("Test");
            let writer_id = stage.add_system(PositionWriter);
            stage.add_system(PositionReader);
            stage.add_system(VelocityWriter);

            let conflicts = stage.find_conflicts_for_system(writer_id);
            assert_eq!(conflicts.len(), 1);
            assert_eq!(conflicts[0].second_system_name, "PositionReader");
        }

        #[test]
        fn test_find_conflicts_for_system_invalid_id() {
            let stage = SystemStage::new("Test");
            let fake_id = SystemId::from_raw(999);
            let conflicts = stage.find_conflicts_for_system(fake_id);
            assert!(conflicts.is_empty());
        }

        // =====================================================================
        // read_only_systems and writing_systems tests
        // =====================================================================

        #[test]
        fn test_read_only_systems_empty() {
            let stage = SystemStage::new("Test");
            assert!(stage.read_only_systems().is_empty());
        }

        #[test]
        fn test_read_only_systems() {
            let mut stage = SystemStage::new("Test");
            let reader_id = stage.add_system(PositionReader);
            stage.add_system(PositionWriter);

            let read_only = stage.read_only_systems();
            assert_eq!(read_only.len(), 1);
            assert_eq!(read_only[0], reader_id);
        }

        #[test]
        fn test_writing_systems() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionReader);
            let writer_id = stage.add_system(PositionWriter);

            let writers = stage.writing_systems();
            assert_eq!(writers.len(), 1);
            assert_eq!(writers[0], writer_id);
        }

        #[test]
        fn test_no_access_is_read_only() {
            let mut stage = SystemStage::new("Test");
            let id = stage.add_system(NoAccessSystem);

            let read_only = stage.read_only_systems();
            assert!(read_only.contains(&id));
        }

        // =====================================================================
        // compute_parallel_groups tests
        // =====================================================================

        #[test]
        fn test_compute_parallel_groups_empty() {
            let stage = SystemStage::new("Test");
            let groups = stage.compute_parallel_groups();
            assert!(groups.is_empty());
        }

        #[test]
        fn test_compute_parallel_groups_no_conflicts() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter);
            stage.add_system(HealthWriter);

            let groups = stage.compute_parallel_groups();
            // All systems can run in parallel
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].len(), 3);
        }

        #[test]
        fn test_compute_parallel_groups_all_conflict() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionWriter);
            stage.add_system(PositionWriter);

            let groups = stage.compute_parallel_groups();
            // Each system in its own group
            assert_eq!(groups.len(), 3);
            assert!(groups.iter().all(|g| g.len() == 1));
        }

        #[test]
        fn test_compute_parallel_groups_mixed() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter); // Can run with PositionWriter
            stage.add_system(PositionReader); // Conflicts with PositionWriter

            let groups = stage.compute_parallel_groups();
            // Group 1: PositionWriter + VelocityWriter
            // Group 2: PositionReader
            assert_eq!(groups.len(), 2);
        }

        #[test]
        fn test_compute_parallel_groups_readers() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionReader);
            stage.add_system(PositionReader);
            stage.add_system(PositionReader);

            let groups = stage.compute_parallel_groups();
            // All readers can run together
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0].len(), 3);
        }

        // =====================================================================
        // conflict_count tests
        // =====================================================================

        #[test]
        fn test_conflict_count_empty() {
            let stage = SystemStage::new("Test");
            assert_eq!(stage.conflict_count(), 0);
        }

        #[test]
        fn test_conflict_count_none() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(VelocityWriter);
            assert_eq!(stage.conflict_count(), 0);
        }

        #[test]
        fn test_conflict_count_some() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionReader);
            stage.add_system(PositionWriter);

            // Writer1 conflicts with Reader, Writer1 conflicts with Writer2,
            // Reader conflicts with Writer2 = 3 conflicts
            assert_eq!(stage.conflict_count(), 3);
        }

        // =====================================================================
        // combined_access tests
        // =====================================================================

        #[test]
        fn test_combined_access_empty() {
            let stage = SystemStage::new("Test");
            let access = stage.combined_access();
            assert!(access.is_empty());
        }

        #[test]
        fn test_combined_access_single() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);

            let access = stage.combined_access();
            assert!(access.writes().contains(&ComponentId::of::<Position>()));
        }

        #[test]
        fn test_combined_access_multiple() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(MovementSystem);
            stage.add_system(HealthWriter);

            let access = stage.combined_access();
            assert!(access.writes().contains(&ComponentId::of::<Position>()));
            assert!(access.writes().contains(&ComponentId::of::<Health>()));
            // Velocity is read by MovementSystem
            assert!(access
                .reads()
                .any(|&id| id == ComponentId::of::<Velocity>()));
        }

        // =====================================================================
        // SystemConflict tests
        // =====================================================================

        #[test]
        fn test_system_conflict_display() {
            let mut stage = SystemStage::new("Test");
            stage.add_system(PositionWriter);
            stage.add_system(PositionReader);

            let conflicts = stage.find_conflicts();
            let display = format!("{}", conflicts[0]);

            assert!(display.contains("PositionWriter"));
            assert!(display.contains("PositionReader"));
            assert!(display.contains("Conflict"));
        }

        #[test]
        fn test_system_conflict_accessors() {
            let mut stage = SystemStage::new("Test");
            let writer_id = stage.add_system(PositionWriter);
            let reader_id = stage.add_system(PositionReader);

            let conflicts = stage.find_conflicts();
            let conflict = &conflicts[0];

            assert_eq!(conflict.system_ids(), (writer_id, reader_id));
            assert_eq!(
                conflict.system_names(),
                ("PositionWriter", "PositionReader")
            );
            assert!(!conflict.is_write_write()); // It's write-read
        }

        // =====================================================================
        // Integration tests
        // =====================================================================

        #[test]
        fn test_conflict_detection_stress() {
            let mut stage = SystemStage::new("Test");

            // Add 10 systems that all write to Position
            for _ in 0..10 {
                stage.add_system(PositionWriter);
            }

            // Should have 10 * 9 / 2 = 45 conflicts
            assert_eq!(stage.conflict_count(), 45);

            // Each system is in its own group
            let groups = stage.compute_parallel_groups();
            assert_eq!(groups.len(), 10);
        }

        #[test]
        fn test_conflict_detection_with_removal() {
            let mut stage = SystemStage::new("Test");
            let writer1 = stage.add_system(PositionWriter);
            let writer2 = stage.add_system(PositionWriter);

            assert!(stage.has_conflicts());
            assert_eq!(stage.conflict_count(), 1);

            // Remove one writer
            stage.remove_system(writer1);

            assert!(!stage.has_conflicts());
            assert_eq!(stage.conflict_count(), 0);

            // Remove second writer
            stage.remove_system(writer2);
            assert!(!stage.has_conflicts());
        }
    }

    // ========================================================================
    // System Ordering Tests
    // ========================================================================

    mod system_ordering {
        use super::*;

        // ====================================================================
        // SystemOrdering Enum Tests
        // ====================================================================

        #[test]
        fn test_ordering_before() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            let ordering = SystemOrdering::before(a, b);

            assert_eq!(ordering.first(), a);
            assert_eq!(ordering.second(), b);
            assert_eq!(ordering.as_edge(), (a, b));
        }

        #[test]
        fn test_ordering_after() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            let ordering = SystemOrdering::after(a, b);

            // After means: a runs after b, so b is first, a is second
            assert_eq!(ordering.first(), b);
            assert_eq!(ordering.second(), a);
            assert_eq!(ordering.as_edge(), (b, a));
        }

        #[test]
        fn test_ordering_involves() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);

            let ordering = SystemOrdering::before(a, b);

            assert!(ordering.involves(a));
            assert!(ordering.involves(b));
            assert!(!ordering.involves(c));
        }

        #[test]
        fn test_ordering_display() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            let before = SystemOrdering::before(a, b);
            let after = SystemOrdering::after(a, b);

            let before_str = format!("{}", before);
            let after_str = format!("{}", after);

            assert!(before_str.contains("before"));
            assert!(after_str.contains("after"));
        }

        #[test]
        fn test_ordering_equality() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            let o1 = SystemOrdering::before(a, b);
            let o2 = SystemOrdering::before(a, b);
            let o3 = SystemOrdering::before(b, a);

            assert_eq!(o1, o2);
            assert_ne!(o1, o3);
        }

        #[test]
        fn test_ordering_hash() {
            use std::collections::HashSet;

            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            let mut set = HashSet::new();
            set.insert(SystemOrdering::before(a, b));
            set.insert(SystemOrdering::before(a, b)); // Duplicate
            set.insert(SystemOrdering::before(b, a)); // Different

            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_ordering_clone() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            let o1 = SystemOrdering::before(a, b);
            let o2 = o1;

            assert_eq!(o1, o2);
        }

        // ====================================================================
        // OrderingCycleError Tests
        // ====================================================================

        #[test]
        fn test_cycle_error_new() {
            let ids = vec![
                SystemId::from_raw(1),
                SystemId::from_raw(2),
                SystemId::from_raw(3),
            ];
            let names = vec!["SystemA", "SystemB", "SystemC"];

            let err = OrderingCycleError::new(ids.clone(), names.clone());

            assert_eq!(err.cycle.len(), 3);
            assert_eq!(err.names.len(), 3);
        }

        #[test]
        fn test_cycle_error_describe() {
            let ids = vec![SystemId::from_raw(1), SystemId::from_raw(2)];
            let names = vec!["A", "B"];

            let err = OrderingCycleError::new(ids, names);
            let desc = err.describe();

            assert!(desc.contains("A"));
            assert!(desc.contains("B"));
            assert!(desc.contains("->"));
        }

        #[test]
        fn test_cycle_error_display() {
            let ids = vec![SystemId::from_raw(1)];
            let names = vec!["Test"];

            let err = OrderingCycleError::new(ids, names);
            let display = format!("{}", err);

            assert!(display.contains("cycle"));
        }

        #[test]
        fn test_cycle_error_empty() {
            let err = OrderingCycleError::new(Vec::new(), Vec::new());
            let desc = err.describe();
            assert!(desc.contains("Empty"));
        }

        // ====================================================================
        // TopologicalSorter Tests
        // ====================================================================

        #[test]
        fn test_sorter_new() {
            let sorter = TopologicalSorter::new();
            assert!(sorter.is_empty());
            assert_eq!(sorter.system_count(), 0);
            assert_eq!(sorter.edge_count(), 0);
        }

        #[test]
        fn test_sorter_with_capacity() {
            let sorter = TopologicalSorter::with_capacity(10, 20);
            assert!(sorter.is_empty());
        }

        #[test]
        fn test_sorter_add_system() {
            let mut sorter = TopologicalSorter::new();
            let id = SystemId::from_raw(1);

            sorter.add_system(id, "TestSystem");

            assert_eq!(sorter.system_count(), 1);
            assert!(!sorter.is_empty());
        }

        #[test]
        fn test_sorter_add_system_duplicate() {
            let mut sorter = TopologicalSorter::new();
            let id = SystemId::from_raw(1);

            sorter.add_system(id, "TestSystem");
            sorter.add_system(id, "TestSystem"); // Duplicate - ignored

            assert_eq!(sorter.system_count(), 1);
        }

        #[test]
        fn test_sorter_add_ordering() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_ordering(a, b);

            assert_eq!(sorter.edge_count(), 1);
        }

        #[test]
        fn test_sorter_add_ordering_missing_system() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            sorter.add_system(a, "A");
            // b not added
            sorter.add_ordering(a, b);

            // Edge should not be added
            assert_eq!(sorter.edge_count(), 0);
        }

        #[test]
        fn test_sorter_add_ordering_self() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);

            sorter.add_system(a, "A");
            sorter.add_ordering(a, a); // Self-ordering - ignored

            assert_eq!(sorter.edge_count(), 0);
        }

        #[test]
        fn test_sorter_add_ordering_duplicate() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_ordering(a, b);
            sorter.add_ordering(a, b); // Duplicate - ignored

            assert_eq!(sorter.edge_count(), 1);
        }

        #[test]
        fn test_sorter_clear() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_ordering(a, b);

            sorter.clear();

            assert!(sorter.is_empty());
            assert_eq!(sorter.edge_count(), 0);
        }

        #[test]
        fn test_sorter_sort_empty() {
            let sorter = TopologicalSorter::new();
            let result = sorter.sort();
            assert!(result.is_ok());
            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn test_sorter_sort_single() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);

            sorter.add_system(a, "A");

            let result = sorter.sort().unwrap();
            assert_eq!(result, vec![a]);
        }

        #[test]
        fn test_sorter_sort_no_constraints() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");

            let result = sorter.sort().unwrap();
            assert_eq!(result.len(), 2);
            // Order can be either A, B or B, A
            assert!(result.contains(&a));
            assert!(result.contains(&b));
        }

        #[test]
        fn test_sorter_sort_linear_chain() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_system(c, "C");

            // A -> B -> C
            sorter.add_ordering(a, b);
            sorter.add_ordering(b, c);

            let result = sorter.sort().unwrap();
            assert_eq!(result, vec![a, b, c]);
        }

        #[test]
        fn test_sorter_sort_diamond() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);
            let d = SystemId::from_raw(4);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_system(c, "C");
            sorter.add_system(d, "D");

            //     A
            //    / \
            //   B   C
            //    \ /
            //     D
            sorter.add_ordering(a, b);
            sorter.add_ordering(a, c);
            sorter.add_ordering(b, d);
            sorter.add_ordering(c, d);

            let result = sorter.sort().unwrap();

            // A must be first, D must be last
            assert_eq!(result[0], a);
            assert_eq!(result[3], d);

            // B and C can be in any order
            let middle: Vec<_> = result[1..3].to_vec();
            assert!(middle.contains(&b));
            assert!(middle.contains(&c));
        }

        #[test]
        fn test_sorter_sort_cycle_detection() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_system(c, "C");

            // A -> B -> C -> A (cycle!)
            sorter.add_ordering(a, b);
            sorter.add_ordering(b, c);
            sorter.add_ordering(c, a);

            let result = sorter.sort();
            assert!(result.is_err());

            let err = result.unwrap_err();
            assert!(!err.cycle.is_empty());
        }

        #[test]
        fn test_sorter_sort_self_cycle() {
            // Self-cycles are filtered out by add_ordering
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);

            sorter.add_system(a, "A");
            sorter.add_ordering(a, a);

            // Should succeed since self-ordering was ignored
            let result = sorter.sort();
            assert!(result.is_ok());
        }

        #[test]
        fn test_sorter_would_cycle() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);

            sorter.add_system(a, "A");
            sorter.add_system(b, "B");
            sorter.add_ordering(a, b);
            sorter.add_ordering(b, a);

            assert!(sorter.would_cycle());
        }

        #[test]
        fn test_sorter_clone() {
            let mut sorter = TopologicalSorter::new();
            let a = SystemId::from_raw(1);

            sorter.add_system(a, "A");

            let cloned = sorter.clone();
            assert_eq!(cloned.system_count(), sorter.system_count());
        }

        // ====================================================================
        // SystemStage Ordering API Tests
        // ====================================================================

        mod stage_ordering_api {
            use super::*;
            use crate::ecs::system::System;

            // Helper systems for testing
            struct SimpleSystem {
                name: &'static str,
            }

            impl System for SimpleSystem {
                fn name(&self) -> &'static str {
                    self.name
                }

                fn run(&mut self, _world: &mut World) {}
            }

            #[test]
            fn test_add_ordering_basic() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                assert!(stage.add_ordering(a, b));
                assert_eq!(stage.ordering_count(), 1);
            }

            #[test]
            fn test_add_ordering_nonexistent_system() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let nonexistent = SystemId::from_raw(9999);

                assert!(!stage.add_ordering(a, nonexistent));
                assert!(!stage.add_ordering(nonexistent, a));
                assert_eq!(stage.ordering_count(), 0);
            }

            #[test]
            fn test_add_ordering_self() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });

                assert!(!stage.add_ordering(a, a));
                assert_eq!(stage.ordering_count(), 0);
            }

            #[test]
            fn test_add_ordering_duplicate() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                assert!(stage.add_ordering(a, b));
                assert!(stage.add_ordering(a, b)); // Duplicate - returns true but doesn't add
                assert_eq!(stage.ordering_count(), 1);
            }

            #[test]
            fn test_set_before() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                assert!(stage.set_before(a, b));
                assert_eq!(stage.ordering_count(), 1);
            }

            #[test]
            fn test_set_after() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                assert!(stage.set_after(a, b)); // A runs after B
                assert_eq!(stage.ordering_count(), 1);

                // Should create ordering B -> A
                let orderings: Vec<_> = stage.orderings().collect();
                assert_eq!(orderings[0].first(), b);
                assert_eq!(orderings[0].second(), a);
            }

            #[test]
            fn test_remove_orderings_for() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });
                let c = stage.add_system(SimpleSystem { name: "C" });

                stage.add_ordering(a, b);
                stage.add_ordering(b, c);
                stage.add_ordering(a, c);

                assert_eq!(stage.ordering_count(), 3);

                // Remove orderings involving A
                let removed = stage.remove_orderings_for(a);
                assert_eq!(removed, 2); // A->B and A->C
                assert_eq!(stage.ordering_count(), 1); // Only B->C remains
            }

            #[test]
            fn test_clear_orderings() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                stage.add_ordering(a, b);
                stage.add_ordering(b, a);

                stage.clear_orderings();
                assert_eq!(stage.ordering_count(), 0);
            }

            #[test]
            fn test_orderings_iterator() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });
                let c = stage.add_system(SimpleSystem { name: "C" });

                stage.add_ordering(a, b);
                stage.add_ordering(b, c);

                let orderings: Vec<_> = stage.orderings().collect();
                assert_eq!(orderings.len(), 2);
            }

            #[test]
            fn test_is_order_dirty() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                assert!(!stage.is_order_dirty());

                stage.add_ordering(a, b);
                assert!(stage.is_order_dirty());

                stage.rebuild_order().unwrap();
                assert!(!stage.is_order_dirty());
            }

            #[test]
            fn test_rebuild_order_no_orderings() {
                let mut stage = SystemStage::new("Test");
                stage.add_system(SimpleSystem { name: "A" });
                stage.add_system(SimpleSystem { name: "B" });

                // No orderings - should succeed and do nothing
                assert!(stage.rebuild_order().is_ok());
            }

            #[test]
            fn test_rebuild_order_reverses_add_order() {
                let mut stage = SystemStage::new("Test");
                let b = stage.add_system(SimpleSystem { name: "B" }); // Added first
                let a = stage.add_system(SimpleSystem { name: "A" }); // Added second

                // Require A to run before B (opposite of add order)
                stage.add_ordering(a, b);
                stage.rebuild_order().unwrap();

                let names = stage.system_names();
                assert_eq!(names[0], "A");
                assert_eq!(names[1], "B");
            }

            #[test]
            fn test_rebuild_order_complex_chain() {
                let mut stage = SystemStage::new("Test");

                // Add in random order
                let c = stage.add_system(SimpleSystem { name: "C" });
                let a = stage.add_system(SimpleSystem { name: "A" });
                let d = stage.add_system(SimpleSystem { name: "D" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                // Define chain: A -> B -> C -> D
                stage.add_ordering(a, b);
                stage.add_ordering(b, c);
                stage.add_ordering(c, d);

                stage.rebuild_order().unwrap();

                let names = stage.system_names();
                assert_eq!(names, vec!["A", "B", "C", "D"]);
            }

            #[test]
            fn test_rebuild_order_cycle_error() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });
                let c = stage.add_system(SimpleSystem { name: "C" });

                // Create cycle: A -> B -> C -> A
                stage.add_ordering(a, b);
                stage.add_ordering(b, c);
                stage.add_ordering(c, a);

                let result = stage.rebuild_order();
                assert!(result.is_err());

                let err = result.unwrap_err();
                assert!(!err.cycle.is_empty());
            }

            #[test]
            fn test_would_ordering_cycle() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                stage.add_ordering(a, b);

                // Adding B -> A would create cycle
                assert!(stage.would_ordering_cycle(b, a));

                // Adding A -> B again wouldn't create new cycle (already exists)
                assert!(!stage.would_ordering_cycle(a, b));
            }

            #[test]
            fn test_orderings_for() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });
                let c = stage.add_system(SimpleSystem { name: "C" });

                stage.add_ordering(a, b);
                stage.add_ordering(a, c);
                stage.add_ordering(b, c);

                let orderings_for_a = stage.orderings_for(a);
                assert_eq!(orderings_for_a.len(), 2); // A->B and A->C

                let orderings_for_b = stage.orderings_for(b);
                assert_eq!(orderings_for_b.len(), 2); // A->B and B->C

                let orderings_for_c = stage.orderings_for(c);
                assert_eq!(orderings_for_c.len(), 2); // A->C and B->C
            }

            #[test]
            fn test_run_auto_rebuilds_order() {
                use std::sync::atomic::{AtomicUsize, Ordering};
                use std::sync::Arc;

                struct OrderTracker {
                    name: &'static str,
                    order: Arc<AtomicUsize>,
                    expected: usize,
                    actual: Arc<std::sync::atomic::AtomicUsize>,
                }

                impl System for OrderTracker {
                    fn name(&self) -> &'static str {
                        self.name
                    }

                    fn run(&mut self, _: &mut World) {
                        let current = self.order.fetch_add(1, Ordering::SeqCst);
                        self.actual.store(current, Ordering::SeqCst);
                    }
                }

                let order = Arc::new(AtomicUsize::new(0));
                let actual_a = Arc::new(std::sync::atomic::AtomicUsize::new(999));
                let actual_b = Arc::new(std::sync::atomic::AtomicUsize::new(999));

                let mut stage = SystemStage::new("Test");

                // Add B first, A second
                let b = stage.add_system(OrderTracker {
                    name: "B",
                    order: order.clone(),
                    expected: 1,
                    actual: actual_b.clone(),
                });
                let a = stage.add_system(OrderTracker {
                    name: "A",
                    order: order.clone(),
                    expected: 0,
                    actual: actual_a.clone(),
                });

                // Require A before B
                stage.add_ordering(a, b);

                let mut world = World::new();
                stage.run(&mut world); // Should auto-rebuild order

                // A should have run first (position 0)
                assert_eq!(actual_a.load(Ordering::SeqCst), 0);
                // B should have run second (position 1)
                assert_eq!(actual_b.load(Ordering::SeqCst), 1);
            }

            #[test]
            fn test_clear_also_clears_orderings() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });

                stage.add_ordering(a, b);
                assert_eq!(stage.ordering_count(), 1);

                stage.clear();
                assert_eq!(stage.ordering_count(), 0);
            }

            #[test]
            fn test_debug_shows_ordering_info() {
                let mut stage = SystemStage::new("Test");
                let a = stage.add_system(SimpleSystem { name: "A" });
                let b = stage.add_system(SimpleSystem { name: "B" });
                stage.add_ordering(a, b);

                let debug = format!("{:?}", stage);
                assert!(debug.contains("ordering_count"));
                assert!(debug.contains("order_dirty"));
            }
        }

        // ====================================================================
        // Stress Tests
        // ====================================================================

        mod stress_tests {
            use super::*;

            #[test]
            fn test_sorter_many_systems() {
                let mut sorter = TopologicalSorter::new();

                // Add 100 systems in a chain
                for i in 0..100 {
                    sorter.add_system(SystemId::from_raw(i), "System");
                }

                for i in 0..99 {
                    sorter.add_ordering(SystemId::from_raw(i), SystemId::from_raw(i + 1));
                }

                let result = sorter.sort().unwrap();
                assert_eq!(result.len(), 100);

                // Verify order
                for i in 0..100 {
                    assert_eq!(result[i as usize], SystemId::from_raw(i));
                }
            }

            #[test]
            fn test_sorter_complex_dag() {
                // Create a complex DAG with multiple valid orderings
                let mut sorter = TopologicalSorter::new();

                for i in 0..10 {
                    sorter.add_system(SystemId::from_raw(i), "System");
                }

                // Create edges that form a valid DAG
                let edges = [
                    (0, 1),
                    (0, 2),
                    (0, 3),
                    (1, 4),
                    (2, 4),
                    (3, 5),
                    (4, 6),
                    (5, 6),
                    (6, 7),
                    (6, 8),
                    (6, 9),
                ];

                for (from, to) in edges {
                    sorter.add_ordering(SystemId::from_raw(from), SystemId::from_raw(to));
                }

                let result = sorter.sort().unwrap();
                assert_eq!(result.len(), 10);

                // Verify all edges are respected
                for (from, to) in edges {
                    let from_pos = result
                        .iter()
                        .position(|id| *id == SystemId::from_raw(from))
                        .unwrap();
                    let to_pos = result
                        .iter()
                        .position(|id| *id == SystemId::from_raw(to))
                        .unwrap();
                    assert!(from_pos < to_pos, "Edge {}->{} violated", from, to);
                }
            }
        }
    }

    // ========================================================================
    // Parallel Execution Tests
    // ========================================================================

    mod parallel_execution {
        use super::*;
        use crate::ecs::component::ComponentId;
        use crate::ecs::query::Access;
        use crate::ecs::system::System;
        use crate::ecs::Component;
        use std::sync::atomic::{AtomicU32, Ordering as AtomicOrdering};
        use std::sync::Arc;

        // Test components
        #[derive(Debug, Clone, Copy)]
        struct Position {
            x: f32,
            y: f32,
        }
        impl Component for Position {}

        #[derive(Debug, Clone, Copy)]
        struct Velocity {
            x: f32,
            y: f32,
        }
        impl Component for Velocity {}

        #[derive(Debug, Clone, Copy)]
        struct Health(f32);
        impl Component for Health {}

        // =================================================================
        // ParallelExecutionConfig Tests
        // =================================================================

        #[test]
        fn test_config_default() {
            let config = ParallelExecutionConfig::default();
            assert_eq!(config.max_threads, 0);
            assert!(config.auto_rebuild);
            assert!(config.respect_ordering);
        }

        #[test]
        fn test_config_with_max_threads() {
            let config = ParallelExecutionConfig::with_max_threads(4);
            assert_eq!(config.max_threads, 4);
            assert!(config.auto_rebuild);
            assert!(config.respect_ordering);
        }

        #[test]
        fn test_config_ignore_ordering() {
            let config = ParallelExecutionConfig::ignore_ordering();
            assert!(!config.respect_ordering);
        }

        // =================================================================
        // ParallelBatch Tests
        // =================================================================

        #[test]
        fn test_batch_new() {
            let batch = ParallelBatch::new();
            assert!(batch.is_empty());
            assert_eq!(batch.len(), 0);
            assert!(batch.all_read_only);
        }

        #[test]
        fn test_batch_with_capacity() {
            let batch = ParallelBatch::with_capacity(10);
            assert!(batch.is_empty());
        }

        #[test]
        fn test_batch_add() {
            let mut batch = ParallelBatch::new();
            let id = SystemId::from_raw(1);

            batch.add(id, true);
            assert_eq!(batch.len(), 1);
            assert!(batch.all_read_only);

            batch.add(SystemId::from_raw(2), false);
            assert_eq!(batch.len(), 2);
            assert!(!batch.all_read_only);
        }

        #[test]
        fn test_batch_can_parallelize() {
            let mut batch = ParallelBatch::new();
            assert!(!batch.can_parallelize());

            batch.add(SystemId::from_raw(1), true);
            assert!(!batch.can_parallelize());

            batch.add(SystemId::from_raw(2), true);
            assert!(batch.can_parallelize());
        }

        #[test]
        fn test_batch_default() {
            let batch = ParallelBatch::default();
            assert!(batch.is_empty());
            assert!(batch.all_read_only);
        }

        // =================================================================
        // ParallelExecutionStats Tests
        // =================================================================

        #[test]
        fn test_stats_default() {
            let stats = ParallelExecutionStats::default();
            assert_eq!(stats.batch_count, 0);
            assert_eq!(stats.system_count, 0);
            assert_eq!(stats.parallel_systems, 0);
            assert_eq!(stats.sequential_systems, 0);
            assert_eq!(stats.max_parallelism, 0);
        }

        #[test]
        fn test_stats_parallelism_ratio() {
            let mut stats = ParallelExecutionStats::default();
            assert_eq!(stats.parallelism_ratio(), 0.0);

            stats.system_count = 10;
            stats.parallel_systems = 5;
            assert_eq!(stats.parallelism_ratio(), 0.5);

            stats.parallel_systems = 10;
            assert_eq!(stats.parallelism_ratio(), 1.0);
        }

        // =================================================================
        // ParallelSystemStage Tests
        // =================================================================

        #[test]
        fn test_parallel_stage_new() {
            let stage = ParallelSystemStage::new("Test");
            assert_eq!(stage.name(), "Test");
            assert!(stage.is_empty());
            assert!(!stage.is_initialized());
            assert!(!stage.is_dirty());
        }

        #[test]
        fn test_parallel_stage_with_capacity() {
            let stage = ParallelSystemStage::with_capacity("Test", 10);
            assert_eq!(stage.name(), "Test");
            assert_eq!(stage.system_count(), 0);
        }

        #[test]
        fn test_parallel_stage_from_core() {
            let stage = ParallelSystemStage::from_core(CoreStage::Update);
            assert_eq!(stage.name(), "Update");
        }

        #[test]
        fn test_parallel_stage_with_config() {
            let config = ParallelExecutionConfig::with_max_threads(8);
            let stage = ParallelSystemStage::with_config("Test", config);
            assert_eq!(stage.config().max_threads, 8);
        }

        #[test]
        fn test_parallel_stage_default() {
            let stage = ParallelSystemStage::default();
            assert_eq!(stage.name(), "ParallelStage");
        }

        #[test]
        fn test_parallel_stage_add_system() {
            struct SimpleSystem;
            impl System for SimpleSystem {
                fn name(&self) -> &'static str {
                    "SimpleSystem"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            let id = stage.add_system(SimpleSystem);

            assert!(id.is_valid());
            assert_eq!(stage.system_count(), 1);
            assert!(stage.contains_system(id));
            assert!(stage.is_dirty());
        }

        #[test]
        fn test_parallel_stage_remove_system() {
            struct SimpleSystem;
            impl System for SimpleSystem {
                fn name(&self) -> &'static str {
                    "SimpleSystem"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            let id = stage.add_system(SimpleSystem);

            assert!(stage.remove_system(id));
            assert!(!stage.contains_system(id));
            assert_eq!(stage.system_count(), 0);
        }

        #[test]
        fn test_parallel_stage_clear() {
            struct SimpleSystem;
            impl System for SimpleSystem {
                fn name(&self) -> &'static str {
                    "SimpleSystem"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(SimpleSystem);
            stage.add_system(SimpleSystem);

            stage.clear();
            assert!(stage.is_empty());
            assert!(!stage.is_dirty());
        }

        #[test]
        fn test_parallel_stage_config_mut() {
            let mut stage = ParallelSystemStage::new("Test");
            stage.config_mut().max_threads = 16;

            assert_eq!(stage.config().max_threads, 16);
            assert!(stage.is_dirty()); // Config change marks dirty
        }

        // =================================================================
        // Batch Computation Tests
        // =================================================================

        #[test]
        fn test_rebuild_batches_empty() {
            let mut stage = ParallelSystemStage::new("Test");
            stage.rebuild_batches().unwrap();
            assert_eq!(stage.batch_count(), 0);
        }

        #[test]
        fn test_rebuild_batches_single_system() {
            struct SimpleSystem;
            impl System for SimpleSystem {
                fn name(&self) -> &'static str {
                    "SimpleSystem"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(SimpleSystem);
            stage.rebuild_batches().unwrap();

            assert_eq!(stage.batch_count(), 1);
            assert_eq!(stage.batches()[0].len(), 1);
        }

        #[test]
        fn test_rebuild_batches_no_conflicts() {
            // Two systems with different access - should be in same batch
            struct PositionSystem;
            impl System for PositionSystem {
                fn name(&self) -> &'static str {
                    "PositionSystem"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct VelocitySystem;
            impl System for VelocitySystem {
                fn name(&self) -> &'static str {
                    "VelocitySystem"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Velocity>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(PositionSystem);
            stage.add_system(VelocitySystem);
            stage.rebuild_batches().unwrap();

            // Both systems should be in the same batch (no conflicts)
            assert_eq!(stage.batch_count(), 1);
            assert_eq!(stage.batches()[0].len(), 2);
        }

        #[test]
        fn test_rebuild_batches_with_conflicts() {
            // Two systems with conflicting access - should be in different batches
            struct WriterA;
            impl System for WriterA {
                fn name(&self) -> &'static str {
                    "WriterA"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct WriterB;
            impl System for WriterB {
                fn name(&self) -> &'static str {
                    "WriterB"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(WriterA);
            stage.add_system(WriterB);
            stage.rebuild_batches().unwrap();

            // Systems should be in different batches (write-write conflict)
            assert_eq!(stage.batch_count(), 2);
        }

        #[test]
        fn test_rebuild_batches_with_ordering() {
            struct SystemA;
            impl System for SystemA {
                fn name(&self) -> &'static str {
                    "SystemA"
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct SystemB;
            impl System for SystemB {
                fn name(&self) -> &'static str {
                    "SystemB"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            let id_a = stage.add_system(SystemA);
            let id_b = stage.add_system(SystemB);

            // Add ordering: A must run before B
            stage.add_ordering(id_a, id_b);
            stage.rebuild_batches().unwrap();

            // Should be in different batches due to ordering constraint
            assert_eq!(stage.batch_count(), 2);
        }

        #[test]
        fn test_rebuild_batches_cycle_error() {
            struct SystemA;
            impl System for SystemA {
                fn name(&self) -> &'static str {
                    "SystemA"
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct SystemB;
            impl System for SystemB {
                fn name(&self) -> &'static str {
                    "SystemB"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            let id_a = stage.add_system(SystemA);
            let id_b = stage.add_system(SystemB);

            // Create cycle: A before B, B before A
            stage.add_ordering(id_a, id_b);
            stage.orderings.push(SystemOrdering::before(id_b, id_a));

            let result = stage.rebuild_batches();
            assert!(result.is_err());
        }

        // =================================================================
        // Parallel Execution Tests
        // =================================================================

        #[test]
        fn test_run_empty_stage() {
            let mut stage = ParallelSystemStage::new("Test");
            let mut world = World::new();

            stage.run(&mut world); // Should not panic
            assert!(stage.is_initialized());
        }

        #[test]
        fn test_run_single_system() {
            struct CounterSystem {
                count: Arc<AtomicU32>,
            }
            impl System for CounterSystem {
                fn name(&self) -> &'static str {
                    "CounterSystem"
                }
                fn run(&mut self, _: &mut World) {
                    self.count.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            let counter = Arc::new(AtomicU32::new(0));
            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(CounterSystem {
                count: counter.clone(),
            });

            let mut world = World::new();
            stage.run(&mut world);

            assert_eq!(counter.load(AtomicOrdering::SeqCst), 1);
        }

        #[test]
        fn test_run_parallel_systems() {
            // Two systems that can run in parallel
            struct SystemA {
                counter: Arc<AtomicU32>,
            }
            impl System for SystemA {
                fn name(&self) -> &'static str {
                    "SystemA"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {
                    self.counter.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            struct SystemB {
                counter: Arc<AtomicU32>,
            }
            impl System for SystemB {
                fn name(&self) -> &'static str {
                    "SystemB"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Velocity>());
                    access
                }
                fn run(&mut self, _: &mut World) {
                    self.counter.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            let counter = Arc::new(AtomicU32::new(0));
            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(SystemA {
                counter: counter.clone(),
            });
            stage.add_system(SystemB {
                counter: counter.clone(),
            });

            let mut world = World::new();
            stage.run(&mut world);

            assert_eq!(counter.load(AtomicOrdering::SeqCst), 2);

            // Check stats - both should be in same batch
            let stats = stage.last_stats();
            assert_eq!(stats.batch_count, 1);
            assert_eq!(stats.system_count, 2);
            assert_eq!(stats.parallel_systems, 2);
        }

        #[test]
        fn test_run_sequential_batches() {
            // Two systems with conflicting access - run in different batches
            struct WriteSystemA {
                order: Arc<AtomicU32>,
            }
            impl System for WriteSystemA {
                fn name(&self) -> &'static str {
                    "WriteSystemA"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {
                    self.order.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            struct WriteSystemB {
                order: Arc<AtomicU32>,
            }
            impl System for WriteSystemB {
                fn name(&self) -> &'static str {
                    "WriteSystemB"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {
                    self.order.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            let order = Arc::new(AtomicU32::new(0));
            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(WriteSystemA {
                order: order.clone(),
            });
            stage.add_system(WriteSystemB {
                order: order.clone(),
            });

            let mut world = World::new();
            stage.run(&mut world);

            assert_eq!(order.load(AtomicOrdering::SeqCst), 2);
            assert_eq!(stage.batch_count(), 2);
        }

        #[test]
        fn test_run_respects_should_run() {
            struct SkipSystem {
                ran: Arc<AtomicU32>,
            }
            impl System for SkipSystem {
                fn name(&self) -> &'static str {
                    "SkipSystem"
                }
                fn should_run(&self, _: &World) -> bool {
                    false
                }
                fn run(&mut self, _: &mut World) {
                    self.ran.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            let ran = Arc::new(AtomicU32::new(0));
            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(SkipSystem { ran: ran.clone() });

            let mut world = World::new();
            stage.run(&mut world);

            assert_eq!(ran.load(AtomicOrdering::SeqCst), 0);
        }

        #[test]
        fn test_run_initializes_systems() {
            struct InitSystem {
                initialized: Arc<AtomicU32>,
            }
            impl System for InitSystem {
                fn name(&self) -> &'static str {
                    "InitSystem"
                }
                fn initialize(&mut self, _: &mut World) {
                    self.initialized.fetch_add(1, AtomicOrdering::SeqCst);
                }
                fn run(&mut self, _: &mut World) {}
            }

            let init = Arc::new(AtomicU32::new(0));
            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(InitSystem {
                initialized: init.clone(),
            });

            let mut world = World::new();
            stage.run(&mut world);
            stage.run(&mut world);

            // Should only initialize once
            assert_eq!(init.load(AtomicOrdering::SeqCst), 1);
        }

        #[test]
        fn test_run_with_ordering_constraints() {
            // Test that ordering is respected
            struct OrderedSystem {
                id: u32,
                execution_order: Arc<std::sync::Mutex<Vec<u32>>>,
            }
            impl System for OrderedSystem {
                fn name(&self) -> &'static str {
                    "OrderedSystem"
                }
                fn run(&mut self, _: &mut World) {
                    self.execution_order.lock().unwrap().push(self.id);
                }
            }

            let order = Arc::new(std::sync::Mutex::new(Vec::new()));
            let mut stage = ParallelSystemStage::new("Test");

            let id_a = stage.add_system(OrderedSystem {
                id: 1,
                execution_order: order.clone(),
            });
            let id_b = stage.add_system(OrderedSystem {
                id: 2,
                execution_order: order.clone(),
            });
            let id_c = stage.add_system(OrderedSystem {
                id: 3,
                execution_order: order.clone(),
            });

            // Enforce order: A -> B -> C
            stage.add_ordering(id_a, id_b);
            stage.add_ordering(id_b, id_c);

            let mut world = World::new();
            stage.run(&mut world);

            let execution_order = order.lock().unwrap().clone();
            assert_eq!(execution_order, vec![1, 2, 3]);
        }

        #[test]
        fn test_stage_trait_implementation() {
            struct SimpleSystem;
            impl System for SimpleSystem {
                fn name(&self) -> &'static str {
                    "SimpleSystem"
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("TestStage");
            stage.add_system(SimpleSystem);

            // Test via Stage trait
            let stage: &mut dyn Stage = &mut stage;

            assert_eq!(stage.name(), "TestStage");
            assert_eq!(stage.system_count(), 1);
            assert!(!stage.is_empty());

            let mut world = World::new();
            stage.run(&mut world);
        }

        #[test]
        fn test_debug_format() {
            let stage = ParallelSystemStage::new("Test");
            let debug = format!("{:?}", stage);

            assert!(debug.contains("ParallelSystemStage"));
            assert!(debug.contains("Test"));
        }

        #[test]
        fn test_send_sync() {
            fn assert_send<T: Send>() {}
            fn assert_sync<T: Sync>() {}

            assert_send::<ParallelSystemStage>();
            assert_sync::<ParallelSystemStage>();
            assert_send::<ParallelExecutionConfig>();
            assert_sync::<ParallelExecutionConfig>();
            assert_send::<ParallelBatch>();
            assert_sync::<ParallelBatch>();
            assert_send::<ParallelExecutionStats>();
            assert_sync::<ParallelExecutionStats>();
        }

        #[test]
        fn test_many_parallel_systems() {
            // Test with many systems to verify parallel execution works at scale
            struct CounterSystem {
                counter: Arc<AtomicU32>,
                id: usize,
            }
            impl System for CounterSystem {
                fn name(&self) -> &'static str {
                    "CounterSystem"
                }
                fn run(&mut self, _: &mut World) {
                    self.counter.fetch_add(1, AtomicOrdering::SeqCst);
                }
            }

            let counter = Arc::new(AtomicU32::new(0));
            let mut stage = ParallelSystemStage::new("Test");

            // Add 100 systems with no access (all can run in parallel)
            for i in 0..100 {
                stage.add_system(CounterSystem {
                    counter: counter.clone(),
                    id: i,
                });
            }

            let mut world = World::new();
            stage.run(&mut world);

            assert_eq!(counter.load(AtomicOrdering::SeqCst), 100);

            // All should be in the same batch
            assert_eq!(stage.batch_count(), 1);
            assert_eq!(stage.batches()[0].len(), 100);
        }

        #[test]
        fn test_conflict_detection() {
            struct WriterA;
            impl System for WriterA {
                fn name(&self) -> &'static str {
                    "WriterA"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct ReaderA;
            impl System for ReaderA {
                fn name(&self) -> &'static str {
                    "ReaderA"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_read(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            stage.add_system(WriterA);
            stage.add_system(ReaderA);

            assert!(stage.has_conflicts());
            let conflicts = stage.find_conflicts();
            assert_eq!(conflicts.len(), 1);
        }

        #[test]
        fn test_read_only_and_writing_systems() {
            struct Reader;
            impl System for Reader {
                fn name(&self) -> &'static str {
                    "Reader"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_read(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            struct Writer;
            impl System for Writer {
                fn name(&self) -> &'static str {
                    "Writer"
                }
                fn component_access(&self) -> Access {
                    let mut access = Access::new();
                    access.add_write(ComponentId::of::<Position>());
                    access
                }
                fn run(&mut self, _: &mut World) {}
            }

            let mut stage = ParallelSystemStage::new("Test");
            let reader_id = stage.add_system(Reader);
            let writer_id = stage.add_system(Writer);

            let read_only = stage.read_only_systems();
            let writing = stage.writing_systems();

            assert_eq!(read_only.len(), 1);
            assert!(read_only.contains(&reader_id));
            assert_eq!(writing.len(), 1);
            assert!(writing.contains(&writer_id));
        }
    }

    // ========================================================================
    // SystemLabel Tests
    // ========================================================================

    mod system_label {
        use super::*;

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

        #[test]
        fn test_custom_label_name() {
            let label = CustomLabel;
            assert_eq!(label.label_name(), "CustomLabel");
        }

        #[test]
        fn test_custom_label_id() {
            let label = CustomLabel;
            assert_eq!(label.label_id(), TypeId::of::<CustomLabel>());
        }

        #[test]
        fn test_custom_label_dyn_clone() {
            let label = CustomLabel;
            let cloned = label.dyn_clone();
            assert_eq!(cloned.label_name(), "CustomLabel");
        }

        #[test]
        fn test_dyn_eq_same() {
            let a = CustomLabel;
            let b = CustomLabel;
            assert!(a.dyn_eq(&b));
        }

        #[test]
        fn test_dyn_eq_different() {
            let a = CustomLabel;
            let b = AnotherLabel;
            assert!(!a.dyn_eq(&b));
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
            use std::hash::{Hash, Hasher};

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
        fn test_system_label_send() {
            fn assert_send<T: Send>() {}
            assert_send::<CustomLabel>();
        }

        #[test]
        fn test_system_label_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<CustomLabel>();
        }
    }

    // ========================================================================
    // SystemLabelId Tests
    // ========================================================================

    mod system_label_id {
        use super::*;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct TestLabel;

        impl SystemLabel for TestLabel {
            fn label_id(&self) -> TypeId {
                TypeId::of::<Self>()
            }
            fn label_name(&self) -> &'static str {
                "TestLabel"
            }
            fn dyn_clone(&self) -> Box<dyn SystemLabel> {
                Box::new(*self)
            }
        }

        #[test]
        fn test_of() {
            let id = SystemLabelId::of(TestLabel);
            assert_eq!(id.name(), "TestLabel");
        }

        #[test]
        fn test_name() {
            let id = SystemLabelId::of(CoreSystemLabel::Physics);
            assert_eq!(id.name(), "Physics");
        }

        #[test]
        fn test_type_id() {
            let id = SystemLabelId::of(TestLabel);
            assert_eq!(id.type_id(), TypeId::of::<TestLabel>());
        }

        #[test]
        fn test_inner() {
            let id = SystemLabelId::of(CoreSystemLabel::Input);
            assert_eq!(id.inner().label_name(), "Input");
        }

        #[test]
        fn test_equality_same() {
            let a = SystemLabelId::of(TestLabel);
            let b = SystemLabelId::of(TestLabel);
            assert_eq!(a, b);
        }

        #[test]
        fn test_equality_different() {
            let a = SystemLabelId::of(TestLabel);
            let b = SystemLabelId::of(CoreSystemLabel::Physics);
            assert_ne!(a, b);
        }

        #[test]
        fn test_hash() {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let a = SystemLabelId::of(TestLabel);
            let b = SystemLabelId::of(TestLabel);

            let mut hasher_a = DefaultHasher::new();
            let mut hasher_b = DefaultHasher::new();
            a.hash(&mut hasher_a);
            b.hash(&mut hasher_b);

            assert_eq!(hasher_a.finish(), hasher_b.finish());
        }

        #[test]
        fn test_debug() {
            let id = SystemLabelId::of(TestLabel);
            let debug = format!("{:?}", id);
            assert!(debug.contains("TestLabel"));
        }

        #[test]
        fn test_display() {
            let id = SystemLabelId::of(CoreSystemLabel::Audio);
            assert_eq!(format!("{}", id), "Audio");
        }

        #[test]
        fn test_clone() {
            let id = SystemLabelId::of(TestLabel);
            let cloned = id.clone();
            assert_eq!(id, cloned);
        }

        #[test]
        fn test_in_hashmap() {
            let mut map = HashMap::new();
            let id = SystemLabelId::of(TestLabel);
            map.insert(id.clone(), "value");
            assert_eq!(map.get(&id), Some(&"value"));
        }
    }

    // ========================================================================
    // CoreSystemLabel Tests
    // ========================================================================

    mod core_system_label {
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
        fn test_send() {
            fn assert_send<T: Send>() {}
            assert_send::<CoreSystemLabel>();
        }

        #[test]
        fn test_sync() {
            fn assert_sync<T: Sync>() {}
            assert_sync::<CoreSystemLabel>();
        }
    }

    // ========================================================================
    // SystemSet Tests
    // ========================================================================

    mod system_set {
        use super::*;

        #[test]
        fn test_new() {
            let set = SystemSet::new("PhysicsSet");
            assert_eq!(set.name(), "PhysicsSet");
            assert!(set.is_empty());
        }

        #[test]
        fn test_with_capacity() {
            let set = SystemSet::with_capacity("Test", 10);
            assert_eq!(set.name(), "Test");
            assert!(set.is_empty());
        }

        #[test]
        fn test_add() {
            let mut set = SystemSet::new("Test");
            let id = SystemId::from_raw(1);
            assert!(set.add(id));
            assert_eq!(set.len(), 1);
            assert!(set.contains(id));
        }

        #[test]
        fn test_add_duplicate() {
            let mut set = SystemSet::new("Test");
            let id = SystemId::from_raw(1);
            assert!(set.add(id));
            assert!(!set.add(id)); // Should return false for duplicate
            assert_eq!(set.len(), 1);
        }

        #[test]
        fn test_remove() {
            let mut set = SystemSet::new("Test");
            let id = SystemId::from_raw(1);
            set.add(id);
            assert!(set.remove(id));
            assert!(!set.contains(id));
            assert!(set.is_empty());
        }

        #[test]
        fn test_remove_nonexistent() {
            let mut set = SystemSet::new("Test");
            let id = SystemId::from_raw(1);
            assert!(!set.remove(id));
        }

        #[test]
        fn test_contains() {
            let mut set = SystemSet::new("Test");
            let id1 = SystemId::from_raw(1);
            let id2 = SystemId::from_raw(2);
            set.add(id1);
            assert!(set.contains(id1));
            assert!(!set.contains(id2));
        }

        #[test]
        fn test_len() {
            let mut set = SystemSet::new("Test");
            assert_eq!(set.len(), 0);
            set.add(SystemId::from_raw(1));
            assert_eq!(set.len(), 1);
            set.add(SystemId::from_raw(2));
            assert_eq!(set.len(), 2);
        }

        #[test]
        fn test_is_empty() {
            let mut set = SystemSet::new("Test");
            assert!(set.is_empty());
            set.add(SystemId::from_raw(1));
            assert!(!set.is_empty());
        }

        #[test]
        fn test_iter() {
            let mut set = SystemSet::new("Test");
            set.add(SystemId::from_raw(1));
            set.add(SystemId::from_raw(2));
            let ids: Vec<_> = set.iter().collect();
            assert_eq!(ids.len(), 2);
        }

        #[test]
        fn test_clear() {
            let mut set = SystemSet::new("Test");
            set.add(SystemId::from_raw(1));
            set.add(SystemId::from_raw(2));
            set.clear();
            assert!(set.is_empty());
        }

        #[test]
        fn test_default() {
            let set = SystemSet::default();
            assert_eq!(set.name(), "DefaultSet");
        }

        #[test]
        fn test_debug() {
            let set = SystemSet::new("Test");
            let debug = format!("{:?}", set);
            assert!(debug.contains("Test"));
        }

        #[test]
        fn test_clone() {
            let mut set = SystemSet::new("Test");
            set.add(SystemId::from_raw(1));
            let cloned = set.clone();
            assert_eq!(cloned.len(), 1);
            assert!(cloned.contains(SystemId::from_raw(1)));
        }
    }

    // ========================================================================
    // SystemSetConfig Tests
    // ========================================================================

    mod system_set_config {
        use super::*;

        #[test]
        fn test_new() {
            let config = SystemSetConfig::new();
            assert!(config.enabled);
            assert!(config.before_labels.is_empty());
            assert!(config.after_labels.is_empty());
        }

        #[test]
        fn test_default() {
            let config = SystemSetConfig::default();
            assert!(config.enabled);
        }

        #[test]
        fn test_before() {
            let config = SystemSetConfig::new().before(CoreSystemLabel::Physics);
            assert_eq!(config.before_labels.len(), 1);
            assert_eq!(config.before_labels[0].name(), "Physics");
        }

        #[test]
        fn test_after() {
            let config = SystemSetConfig::new().after(CoreSystemLabel::Input);
            assert_eq!(config.after_labels.len(), 1);
            assert_eq!(config.after_labels[0].name(), "Input");
        }

        #[test]
        fn test_enabled() {
            let config = SystemSetConfig::new().enabled(false);
            assert!(!config.enabled);
        }

        #[test]
        fn test_chained_config() {
            let config = SystemSetConfig::new()
                .before(CoreSystemLabel::Physics)
                .after(CoreSystemLabel::Input)
                .enabled(true);
            assert_eq!(config.before_labels.len(), 1);
            assert_eq!(config.after_labels.len(), 1);
            assert!(config.enabled);
        }

        #[test]
        fn test_clone() {
            let config = SystemSetConfig::new().before(CoreSystemLabel::Physics);
            let cloned = config.clone();
            assert_eq!(cloned.before_labels.len(), 1);
        }
    }

    // ========================================================================
    // ChainedSystems Tests
    // ========================================================================

    mod chained_systems {
        use super::*;

        #[test]
        fn test_new() {
            let chain = ChainedSystems::new("TestChain");
            assert_eq!(chain.name(), "TestChain");
            assert!(chain.is_empty());
        }

        #[test]
        fn test_with_capacity() {
            let chain = ChainedSystems::with_capacity("Test", 10);
            assert_eq!(chain.name(), "Test");
            assert!(chain.is_empty());
        }

        #[test]
        fn test_add() {
            let mut chain = ChainedSystems::new("Test");
            chain.add(SystemId::from_raw(1));
            chain.add(SystemId::from_raw(2));
            assert_eq!(chain.len(), 2);
        }

        #[test]
        fn test_add_after() {
            let mut chain = ChainedSystems::new("Test");
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);

            chain.add(a);
            chain.add(c);
            chain.add_after(b, a); // b after a, so order is a, b, c

            let ids: Vec<_> = chain.iter().collect();
            assert_eq!(ids, vec![a, b, c]);
        }

        #[test]
        fn test_add_after_not_found() {
            let mut chain = ChainedSystems::new("Test");
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);

            chain.add(a);
            chain.add_after(b, c); // c not in chain, so b added at end

            let ids: Vec<_> = chain.iter().collect();
            assert_eq!(ids, vec![a, b]);
        }

        #[test]
        fn test_len() {
            let mut chain = ChainedSystems::new("Test");
            assert_eq!(chain.len(), 0);
            chain.add(SystemId::from_raw(1));
            assert_eq!(chain.len(), 1);
        }

        #[test]
        fn test_is_empty() {
            let mut chain = ChainedSystems::new("Test");
            assert!(chain.is_empty());
            chain.add(SystemId::from_raw(1));
            assert!(!chain.is_empty());
        }

        #[test]
        fn test_iter() {
            let mut chain = ChainedSystems::new("Test");
            chain.add(SystemId::from_raw(1));
            chain.add(SystemId::from_raw(2));
            let ids: Vec<_> = chain.iter().collect();
            assert_eq!(ids.len(), 2);
        }

        #[test]
        fn test_to_orderings_empty() {
            let chain = ChainedSystems::new("Test");
            let orderings = chain.to_orderings();
            assert!(orderings.is_empty());
        }

        #[test]
        fn test_to_orderings_single() {
            let mut chain = ChainedSystems::new("Test");
            chain.add(SystemId::from_raw(1));
            let orderings = chain.to_orderings();
            assert!(orderings.is_empty()); // No pairs
        }

        #[test]
        fn test_to_orderings_multiple() {
            let mut chain = ChainedSystems::new("Test");
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);

            chain.add(a);
            chain.add(b);
            chain.add(c);

            let orderings = chain.to_orderings();
            assert_eq!(orderings.len(), 2);

            // Check orderings are a->b and b->c
            assert_eq!(orderings[0].first(), a);
            assert_eq!(orderings[0].second(), b);
            assert_eq!(orderings[1].first(), b);
            assert_eq!(orderings[1].second(), c);
        }

        #[test]
        fn test_clear() {
            let mut chain = ChainedSystems::new("Test");
            chain.add(SystemId::from_raw(1));
            chain.clear();
            assert!(chain.is_empty());
        }

        #[test]
        fn test_default() {
            let chain = ChainedSystems::default();
            assert_eq!(chain.name(), "Chain");
        }

        #[test]
        fn test_clone() {
            let mut chain = ChainedSystems::new("Test");
            chain.add(SystemId::from_raw(1));
            let cloned = chain.clone();
            assert_eq!(cloned.len(), 1);
        }
    }

    // ========================================================================
    // chain Function Tests
    // ========================================================================

    mod chain_function {
        use super::*;

        #[test]
        fn test_chain_empty() {
            let orderings = chain([]);
            assert!(orderings.is_empty());
        }

        #[test]
        fn test_chain_single() {
            let orderings = chain([SystemId::from_raw(1)]);
            assert!(orderings.is_empty());
        }

        #[test]
        fn test_chain_two() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let orderings = chain([a, b]);
            assert_eq!(orderings.len(), 1);
            assert_eq!(orderings[0].first(), a);
            assert_eq!(orderings[0].second(), b);
        }

        #[test]
        fn test_chain_three() {
            let a = SystemId::from_raw(1);
            let b = SystemId::from_raw(2);
            let c = SystemId::from_raw(3);
            let orderings = chain([a, b, c]);
            assert_eq!(orderings.len(), 2);
        }

        #[test]
        fn test_chain_from_vec() {
            let ids = vec![
                SystemId::from_raw(1),
                SystemId::from_raw(2),
                SystemId::from_raw(3),
            ];
            let orderings = chain(ids);
            assert_eq!(orderings.len(), 2);
        }
    }

    // ========================================================================
    // LabeledOrderingConstraint Tests
    // ========================================================================

    mod labeled_ordering_constraint {
        use super::*;

        #[test]
        fn test_before_label() {
            let constraint = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
            assert!(constraint.is_label_based());
            assert!(!constraint.is_system_based());
        }

        #[test]
        fn test_after_label() {
            let constraint = LabeledOrderingConstraint::after_label(CoreSystemLabel::Input);
            assert!(constraint.is_label_based());
        }

        #[test]
        fn test_before_system() {
            let constraint = LabeledOrderingConstraint::before_system(SystemId::from_raw(1));
            assert!(constraint.is_system_based());
            assert!(!constraint.is_label_based());
        }

        #[test]
        fn test_after_system() {
            let constraint = LabeledOrderingConstraint::after_system(SystemId::from_raw(1));
            assert!(constraint.is_system_based());
        }

        #[test]
        fn test_display_before_label() {
            let constraint = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
            let display = format!("{}", constraint);
            assert!(display.contains("before label"));
            assert!(display.contains("Physics"));
        }

        #[test]
        fn test_display_after_label() {
            let constraint = LabeledOrderingConstraint::after_label(CoreSystemLabel::Input);
            let display = format!("{}", constraint);
            assert!(display.contains("after label"));
            assert!(display.contains("Input"));
        }

        #[test]
        fn test_display_before_system() {
            let constraint = LabeledOrderingConstraint::before_system(SystemId::from_raw(42));
            let display = format!("{}", constraint);
            assert!(display.contains("before system"));
            assert!(display.contains("42"));
        }

        #[test]
        fn test_display_after_system() {
            let constraint = LabeledOrderingConstraint::after_system(SystemId::from_raw(99));
            let display = format!("{}", constraint);
            assert!(display.contains("after system"));
            assert!(display.contains("99"));
        }

        #[test]
        fn test_clone() {
            let constraint = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
            let cloned = constraint.clone();
            assert!(cloned.is_label_based());
        }

        #[test]
        fn test_debug() {
            let constraint = LabeledOrderingConstraint::before_label(CoreSystemLabel::Physics);
            let debug = format!("{:?}", constraint);
            assert!(debug.contains("BeforeLabel"));
        }
    }

    // ========================================================================
    // SystemStage chain_systems and add_chain Tests
    // ========================================================================

    mod stage_chain_methods {
        use super::*;
        use crate::ecs::system::System;

        struct SysA;
        impl System for SysA {
            fn name(&self) -> &'static str {
                "A"
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct SysB;
        impl System for SysB {
            fn name(&self) -> &'static str {
                "B"
            }
            fn run(&mut self, _: &mut World) {}
        }

        struct SysC;
        impl System for SysC {
            fn name(&self) -> &'static str {
                "C"
            }
            fn run(&mut self, _: &mut World) {}
        }

        #[test]
        fn test_chain_systems_empty() {
            let mut stage = SystemStage::new("Test");
            let count = stage.chain_systems([]);
            assert_eq!(count, 0);
            assert_eq!(stage.ordering_count(), 0);
        }

        #[test]
        fn test_chain_systems_single() {
            let mut stage = SystemStage::new("Test");
            let a = stage.add_system(SysA);
            let count = stage.chain_systems([a]);
            assert_eq!(count, 0);
            assert_eq!(stage.ordering_count(), 0);
        }

        #[test]
        fn test_chain_systems_two() {
            let mut stage = SystemStage::new("Test");
            let a = stage.add_system(SysA);
            let b = stage.add_system(SysB);
            let count = stage.chain_systems([a, b]);
            assert_eq!(count, 1);
            assert_eq!(stage.ordering_count(), 1);
        }

        #[test]
        fn test_chain_systems_three() {
            let mut stage = SystemStage::new("Test");
            let a = stage.add_system(SysA);
            let b = stage.add_system(SysB);
            let c = stage.add_system(SysC);
            let count = stage.chain_systems([a, b, c]);
            assert_eq!(count, 2);
            assert_eq!(stage.ordering_count(), 2);
        }

        #[test]
        fn test_chain_systems_enforces_order() {
            let mut stage = SystemStage::new("Test");
            // Add in reverse order
            let c = stage.add_system(SysC);
            let b = stage.add_system(SysB);
            let a = stage.add_system(SysA);

            // Chain them: a -> b -> c
            stage.chain_systems([a, b, c]);
            stage.rebuild_order().expect("No cycles");

            // Check the order
            let names = stage.system_names();
            assert_eq!(names[0], "A");
            assert_eq!(names[1], "B");
            assert_eq!(names[2], "C");
        }

        #[test]
        fn test_add_chain_empty() {
            let mut stage = SystemStage::new("Test");
            let chain = ChainedSystems::new("Empty");
            let count = stage.add_chain(&chain);
            assert_eq!(count, 0);
        }

        #[test]
        fn test_add_chain_with_systems() {
            let mut stage = SystemStage::new("Test");
            let a = stage.add_system(SysA);
            let b = stage.add_system(SysB);
            let c = stage.add_system(SysC);

            let mut chain = ChainedSystems::new("TestChain");
            chain.add(a);
            chain.add(b);
            chain.add(c);

            let count = stage.add_chain(&chain);
            assert_eq!(count, 2);
            assert_eq!(stage.ordering_count(), 2);
        }

        #[test]
        fn test_add_chain_enforces_order() {
            let mut stage = SystemStage::new("Test");
            // Add in reverse order
            let c = stage.add_system(SysC);
            let b = stage.add_system(SysB);
            let a = stage.add_system(SysA);

            let mut chain = ChainedSystems::new("TestChain");
            chain.add(a);
            chain.add(b);
            chain.add(c);

            stage.add_chain(&chain);
            stage.rebuild_order().expect("No cycles");

            // Check the order
            let names = stage.system_names();
            assert_eq!(names[0], "A");
            assert_eq!(names[1], "B");
            assert_eq!(names[2], "C");
        }

        #[test]
        fn test_chain_marks_dirty() {
            let mut stage = SystemStage::new("Test");
            let a = stage.add_system(SysA);
            let b = stage.add_system(SysB);

            // Clear dirty flag first
            let _ = stage.rebuild_order();
            assert!(!stage.is_order_dirty());

            stage.chain_systems([a, b]);
            assert!(stage.is_order_dirty());
        }
    }
}
