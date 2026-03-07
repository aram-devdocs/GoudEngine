//! Named system sets with ordering groups.
//!
//! Allows grouping systems into named sets and defining ordering constraints
//! between entire sets rather than individual systems.

use std::any::TypeId;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

use crate::ecs::system::SystemId;

use super::stage_label::DynHasherWrapper;
use super::system_label::SystemLabel;
use super::system_ordering::SystemOrdering;
use super::system_set::{SystemSet, SystemSetConfig};

/// Default system set names matching common game loop phases.
///
/// These mirror [`CoreStage`](super::core_stage::CoreStage) but operate at the
/// system-set level within a single stage, enabling finer-grained ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefaultSystemSet {
    /// Runs before the main update logic.
    PreUpdate,
    /// Main update logic.
    Update,
    /// Runs after the main update logic.
    PostUpdate,
    /// Runs before rendering.
    PreRender,
    /// Rendering phase.
    Render,
    /// Runs after rendering.
    PostRender,
}

impl DefaultSystemSet {
    /// Returns the string name for this set variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            DefaultSystemSet::PreUpdate => "PreUpdate",
            DefaultSystemSet::Update => "Update",
            DefaultSystemSet::PostUpdate => "PostUpdate",
            DefaultSystemSet::PreRender => "PreRender",
            DefaultSystemSet::Render => "Render",
            DefaultSystemSet::PostRender => "PostRender",
        }
    }

    /// Returns all default system set variants.
    pub fn all() -> [DefaultSystemSet; 6] {
        [
            DefaultSystemSet::PreUpdate,
            DefaultSystemSet::Update,
            DefaultSystemSet::PostUpdate,
            DefaultSystemSet::PreRender,
            DefaultSystemSet::Render,
            DefaultSystemSet::PostRender,
        ]
    }
}

impl fmt::Display for DefaultSystemSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl SystemLabel for DefaultSystemSet {
    fn label_id(&self) -> TypeId {
        // Use a per-variant discriminated type so each variant is unique.
        TypeId::of::<(DefaultSystemSet, u8)>()
    }

    fn label_name(&self) -> &'static str {
        self.as_str()
    }

    fn dyn_clone(&self) -> Box<dyn SystemLabel> {
        Box::new(*self)
    }

    fn dyn_eq(&self, other: &dyn SystemLabel) -> bool {
        if other.label_id() == TypeId::of::<(DefaultSystemSet, u8)>() {
            self.label_name() == other.label_name()
        } else {
            false
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        TypeId::of::<DefaultSystemSet>().hash(&mut DynHasherWrapper(state));
        self.as_str().hash(&mut DynHasherWrapper(state));
    }
}

/// A label backed by a static string, used to reference named sets in
/// [`SystemSetConfig::before`] and [`SystemSetConfig::after`].
///
/// # Example
///
/// ```ignore
/// use goud_engine::ecs::schedule::named_system_sets::SetNameLabel;
/// use goud_engine::ecs::schedule::SystemSetConfig;
///
/// let config = SystemSetConfig::new().before(SetNameLabel("Rendering"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetNameLabel(pub &'static str);

impl SystemLabel for SetNameLabel {
    fn label_id(&self) -> TypeId {
        TypeId::of::<SetNameLabel>()
    }

    fn label_name(&self) -> &'static str {
        self.0
    }

    fn dyn_clone(&self) -> Box<dyn SystemLabel> {
        Box::new(*self)
    }

    fn dyn_eq(&self, other: &dyn SystemLabel) -> bool {
        self.label_name() == other.label_name()
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        TypeId::of::<SetNameLabel>().hash(&mut DynHasherWrapper(state));
        self.0.hash(&mut DynHasherWrapper(state));
    }
}

/// A collection of named system sets with ordering configuration.
///
/// `NamedSystemSets` manages a registry of [`SystemSet`]s identified by string
/// names, each with an optional [`SystemSetConfig`] that defines ordering
/// constraints relative to other sets.
///
/// # Ordering Resolution
///
/// When [`resolve_orderings`](Self::resolve_orderings) is called, set-level
/// constraints are expanded into system-level [`SystemOrdering`] pairs. For
/// example, if set "A" is configured to run before set "B", every system in A
/// gets a `Before` ordering relative to every system in B.
#[derive(Debug, Default)]
pub struct NamedSystemSets {
    /// Named system sets.
    sets: HashMap<String, SystemSet>,
    /// Per-set configuration (ordering constraints, enabled flag).
    configs: HashMap<String, SystemSetConfig>,
}

impl NamedSystemSets {
    /// Creates a new empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a named set. No-op if the set already exists.
    pub fn register_set(&mut self, name: &str) {
        self.sets
            .entry(name.to_string())
            .or_insert_with(|| SystemSet::new(name));
    }

    /// Adds a system to a named set.
    ///
    /// Returns `true` if the system was newly added, `false` if already present.
    ///
    /// # Panics
    ///
    /// Panics if the set has not been registered.
    pub fn add_to_set(&mut self, name: &str, system_id: SystemId) -> bool {
        let set = self
            .sets
            .get_mut(name)
            .unwrap_or_else(|| panic!("System set '{}' has not been registered", name));
        set.add(system_id)
    }

    /// Applies a configuration to a named set.
    ///
    /// # Panics
    ///
    /// Panics if the set has not been registered.
    pub fn configure_set(&mut self, name: &str, config: SystemSetConfig) {
        assert!(
            self.sets.contains_key(name),
            "System set '{}' has not been registered",
            name
        );
        self.configs.insert(name.to_string(), config);
    }

    /// Returns a reference to the named set, if it exists.
    pub fn get_set(&self, name: &str) -> Option<&SystemSet> {
        self.sets.get(name)
    }

    /// Returns the configuration for a named set, if any.
    pub fn get_config(&self, name: &str) -> Option<&SystemSetConfig> {
        self.configs.get(name)
    }

    /// Returns the names of all registered sets.
    pub fn set_names(&self) -> Vec<&str> {
        self.sets.keys().map(|s| s.as_str()).collect()
    }

    /// Converts set-level ordering constraints into system-level orderings.
    ///
    /// For each configured set, its `before_labels` and `after_labels` are
    /// matched against set names. The result is a cross-product of orderings
    /// between systems in the source set and systems in each target set.
    pub fn resolve_orderings(&self) -> Vec<SystemOrdering> {
        let mut orderings = Vec::new();

        for (set_name, config) in &self.configs {
            let Some(source_set) = self.sets.get(set_name) else {
                continue;
            };
            if source_set.is_empty() {
                continue;
            }

            // "before" labels: every system in source must run before
            // every system in each target set whose name matches the label.
            for label_id in &config.before_labels {
                let target_name = label_id.name();
                let Some(target_set) = self.sets.get(target_name) else {
                    continue;
                };
                for src in source_set.iter() {
                    for tgt in target_set.iter() {
                        orderings.push(SystemOrdering::before(src, tgt));
                    }
                }
            }

            // "after" labels: every system in source must run after
            // every system in each target set whose name matches the label.
            for label_id in &config.after_labels {
                let target_name = label_id.name();
                let Some(target_set) = self.sets.get(target_name) else {
                    continue;
                };
                for src in source_set.iter() {
                    for tgt in target_set.iter() {
                        orderings.push(SystemOrdering::after(src, tgt));
                    }
                }
            }
        }

        orderings
    }
}

#[cfg(test)]
#[path = "tests/named_system_sets_tests.rs"]
mod tests;
