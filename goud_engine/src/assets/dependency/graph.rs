//! Dependency graph for tracking relationships between assets.
//!
//! The graph maintains both forward edges (what an asset depends on) and
//! reverse edges (what depends on an asset) to support cascade reloading.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;

// =============================================================================
// CycleError
// =============================================================================

/// Error returned when adding a dependency would create a cycle.
#[derive(Debug, Clone)]
pub struct CycleError {
    /// The asset that would create the cycle.
    pub from: String,
    /// The dependency target that would close the cycle.
    pub to: String,
}

impl fmt::Display for CycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "adding dependency '{}' -> '{}' would create a cycle",
            self.from, self.to
        )
    }
}

impl std::error::Error for CycleError {}

// =============================================================================
// DependencyGraph
// =============================================================================

/// Tracks dependency relationships between assets.
///
/// Maintains two maps:
/// - `dependencies`: forward edges (asset -> what it depends on)
/// - `dependents`: reverse edges (asset -> who depends on it)
///
/// Supports cycle detection when adding edges and BFS cascade ordering
/// for reload propagation.
///
/// # Example
///
/// ```
/// use goud_engine::assets::dependency::DependencyGraph;
///
/// let mut graph = DependencyGraph::new();
/// graph.add_dependency("shader.glsl", "common.glsl").unwrap();
/// graph.add_dependency("material.json", "shader.glsl").unwrap();
///
/// // When common.glsl changes, reload shader.glsl then material.json
/// let order = graph.get_cascade_order("common.glsl");
/// assert_eq!(order, vec!["shader.glsl", "material.json"]);
/// ```
pub struct DependencyGraph {
    /// Forward edges: asset -> set of assets it depends on.
    dependencies: HashMap<String, HashSet<String>>,
    /// Reverse edges: asset -> set of assets that depend on it.
    dependents: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Adds a dependency: `asset` depends on `depends_on`.
    ///
    /// Returns `Err(CycleError)` if adding this edge would create a cycle.
    ///
    /// # Arguments
    ///
    /// * `asset` - The asset that has the dependency
    /// * `depends_on` - The asset being depended upon
    pub fn add_dependency(&mut self, asset: &str, depends_on: &str) -> Result<(), CycleError> {
        // Self-dependency is a trivial cycle
        if asset == depends_on {
            return Err(CycleError {
                from: asset.to_string(),
                to: depends_on.to_string(),
            });
        }

        // Check if adding this edge would create a cycle:
        // A cycle exists if `depends_on` can already reach `asset` through
        // the existing forward edges.
        if self.can_reach(depends_on, asset) {
            return Err(CycleError {
                from: asset.to_string(),
                to: depends_on.to_string(),
            });
        }

        // Add forward edge
        self.dependencies
            .entry(asset.to_string())
            .or_default()
            .insert(depends_on.to_string());

        // Add reverse edge
        self.dependents
            .entry(depends_on.to_string())
            .or_default()
            .insert(asset.to_string());

        Ok(())
    }

    /// Returns true if `from` can reach `to` via forward dependency edges.
    ///
    /// Uses BFS to traverse the dependency graph.
    fn can_reach(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(from.to_string());

        while let Some(current) = queue.pop_front() {
            if current == to {
                return true;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            if let Some(deps) = self.dependencies.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        false
    }

    /// Returns the cascade reload order for a changed asset.
    ///
    /// Performs a BFS over reverse edges (dependents) to produce a
    /// topological ordering: assets closer to the changed asset are
    /// reloaded first.
    ///
    /// The changed asset itself is NOT included in the output.
    ///
    /// # Arguments
    ///
    /// * `changed_path` - The path of the asset that changed
    pub fn get_cascade_order(&self, changed_path: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        visited.insert(changed_path.to_string());

        // Seed with direct dependents
        if let Some(deps) = self.dependents.get(changed_path) {
            for dep in deps {
                if visited.insert(dep.clone()) {
                    queue.push_back(dep.clone());
                }
            }
        }

        // BFS through dependents
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            if let Some(deps) = self.dependents.get(&current) {
                for dep in deps {
                    if visited.insert(dep.clone()) {
                        queue.push_back(dep.clone());
                    }
                }
            }
        }

        result
    }

    /// Removes an asset and all its edges from the graph.
    ///
    /// Cleans up both forward and reverse edges so no dangling
    /// references remain.
    pub fn remove_asset(&mut self, path: &str) {
        // Remove forward edges: for each dependency of this asset,
        // remove this asset from their dependents set.
        if let Some(deps) = self.dependencies.remove(path) {
            for dep in &deps {
                if let Some(rev) = self.dependents.get_mut(dep) {
                    rev.remove(path);
                    if rev.is_empty() {
                        self.dependents.remove(dep);
                    }
                }
            }
        }

        // Remove reverse edges: for each dependent of this asset,
        // remove this asset from their dependencies set.
        if let Some(rev_deps) = self.dependents.remove(path) {
            for rev in &rev_deps {
                if let Some(fwd) = self.dependencies.get_mut(rev) {
                    fwd.remove(path);
                    if fwd.is_empty() {
                        self.dependencies.remove(rev);
                    }
                }
            }
        }
    }

    /// Returns the set of assets that `asset` directly depends on.
    pub fn get_dependencies(&self, asset: &str) -> Option<&HashSet<String>> {
        self.dependencies.get(asset)
    }

    /// Returns the set of assets that directly depend on `asset`.
    pub fn get_dependents(&self, asset: &str) -> Option<&HashSet<String>> {
        self.dependents.get(asset)
    }

    /// Returns true if the graph contains any edges for the given asset.
    pub fn contains(&self, asset: &str) -> bool {
        self.dependencies.contains_key(asset) || self.dependents.contains_key(asset)
    }

    /// Returns the total number of assets tracked in the graph.
    pub fn asset_count(&self) -> usize {
        let mut all: HashSet<&str> = HashSet::new();
        for key in self.dependencies.keys() {
            all.insert(key.as_str());
        }
        for key in self.dependents.keys() {
            all.insert(key.as_str());
        }
        all.len()
    }

    /// Clears all dependency relationships.
    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DependencyGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DependencyGraph")
            .field("assets", &self.asset_count())
            .field("forward_edges", &self.dependencies.len())
            .field("reverse_edges", &self.dependents.len())
            .finish()
    }
}


