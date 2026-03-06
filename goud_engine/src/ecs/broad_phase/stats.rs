//! Statistics types for spatial hash performance analysis.

/// Statistics for spatial hash performance analysis.
#[derive(Clone, Debug, Default)]
pub struct SpatialHashStats {
    /// Total number of entities in the hash.
    pub entity_count: usize,

    /// Number of occupied cells.
    pub cell_count: usize,

    /// Total number of entity-cell mappings.
    pub total_cell_entries: usize,

    /// Maximum entities in a single cell.
    pub max_entities_per_cell: usize,

    /// Average entities per occupied cell.
    pub avg_entities_per_cell: f32,

    /// Number of potential collision pairs found in last query.
    pub last_query_pairs: usize,
}

impl std::fmt::Display for SpatialHashStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpatialHash Stats:\n\
             - Entities: {}\n\
             - Cells: {}\n\
             - Total entries: {}\n\
             - Max per cell: {}\n\
             - Avg per cell: {:.2}\n\
             - Last query pairs: {}",
            self.entity_count,
            self.cell_count,
            self.total_cell_entries,
            self.max_entities_per_cell,
            self.avg_entities_per_cell,
            self.last_query_pairs
        )
    }
}
