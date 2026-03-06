//! Topological sorting of systems and cycle detection.

use std::collections::{HashMap, VecDeque};
use std::fmt;

use crate::ecs::system::SystemId;

use super::system_ordering::SystemOrdering;

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
        let mut queue: VecDeque<usize> = VecDeque::new();
        for (i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(i);
            }
        }

        // Process nodes
        let mut result = Vec::with_capacity(n);
        while let Some(idx) = queue.pop_front() {
            result.push(self.systems[idx].0);

            for &neighbor in &adj[idx] {
                in_degree[neighbor] -= 1;
                if in_degree[neighbor] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        // Check for cycle
        if result.len() != n {
            let cycle = self.find_cycle(&adj, &in_degree);
            return Err(cycle);
        }

        Ok(result)
    }

    /// Finds a cycle in the graph for error reporting.
    fn find_cycle(&self, adj: &[Vec<usize>], in_degree: &[usize]) -> OrderingCycleError {
        let start = in_degree.iter().position(|&d| d > 0).unwrap_or(0);

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

    /// Checks if sorting would produce a cycle.
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

#[cfg(test)]
#[path = "tests/topological_sort_tests.rs"]
#[cfg(test)]
mod tests;
