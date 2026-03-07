//! System scheduling and execution ordering.
//!
//! This module provides the infrastructure for organizing systems into stages,
//! defining execution order, and running systems with automatic ordering
//! resolution.
//!
//! # Architecture
//!
//! - **Stage Label**: Trait for identifying stages (`StageLabel`, `StageLabelId`)
//! - **Core Stage**: Built-in game loop stages (`CoreStage`)
//! - **Stage Position**: Stage ordering and relative positioning
//! - **System Ordering**: Ordering constraints between systems
//! - **System Label**: Trait for labeling systems (`SystemLabel`, `SystemLabelId`)
//! - **Core System Label**: Built-in system labels (`CoreSystemLabel`)
//! - **System Set**: Grouping systems, chains, and label-based ordering
//! - **Topological Sort**: Dependency-aware ordering resolution
//! - **Stage**: Trait for system containers (`Stage`)
//! - **System Stage**: Sequential execution stage (`SystemStage`)
//! - **System Conflict**: Conflict detection types
//! - **Parallel**: Parallel execution stage (`ParallelSystemStage`)

mod conflict_utils;
mod core_stage;
mod core_system_label;
pub mod named_system_sets;
mod parallel;
mod parallel_conflicts;
mod parallel_types;
mod stage;
mod stage_label;
mod stage_position;
mod system_conflict;
mod system_label;
mod system_ordering;
mod system_set;
mod system_stage;
mod system_stage_conflicts;
mod topological_sort;

// Re-export everything to preserve the public API.
pub use core_stage::CoreStage;
pub use core_system_label::CoreSystemLabel;
pub use named_system_sets::{DefaultSystemSet, NamedSystemSets};
pub use parallel::ParallelSystemStage;
pub use parallel_types::{ParallelBatch, ParallelExecutionConfig, ParallelExecutionStats};
pub use stage::Stage;
pub use stage_label::{StageLabel, StageLabelId};
pub use stage_position::{StageOrder, StagePosition};
pub use system_conflict::SystemConflict;
pub use system_label::{SystemLabel, SystemLabelId};
pub use system_ordering::SystemOrdering;
pub use system_set::{
    chain, ChainedSystems, LabeledOrderingConstraint, SystemSet, SystemSetConfig,
};
pub use system_stage::SystemStage;
pub use topological_sort::{OrderingCycleError, TopologicalSorter};

#[cfg(test)]
mod tests;
