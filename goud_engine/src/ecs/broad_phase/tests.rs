//! Tests for the broad phase spatial hash.

#[cfg(test)]
mod tests {
    use crate::core::math::{Rect, Vec2};
    use crate::ecs::broad_phase::{SpatialHash, SpatialHashStats};
    use crate::ecs::Entity;

    // Helper to create test entities
    fn entity(id: u32) -> Entity {
        Entity::new(id, 0)
    }

    // =========================================================================
    // Construction Tests
    // =========================================================================

    #[test]
    fn test_new() {
        let hash = SpatialHash::new(64.0);
        assert_eq!(hash.cell_size(), 64.0);
        assert_eq!(hash.entity_count(), 0);
        assert_eq!(hash.cell_count(), 0);
        assert!(hash.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let hash = SpatialHash::with_capacity(64.0, 100);
        assert_eq!(hash.cell_size(), 64.0);
        assert!(hash.is_empty());
    }

    #[test]
    #[should_panic(expected = "Cell size must be positive and finite")]
    fn test_new_invalid_cell_size() {
        let _ = SpatialHash::new(0.0);
    }

    #[test]
    #[should_panic(expected = "Cell size must be positive and finite")]
    fn test_new_negative_cell_size() {
        let _ = SpatialHash::new(-10.0);
    }

    // =========================================================================
    // Insertion and Removal Tests
    // =========================================================================

    #[test]
    fn test_insert_single() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);
        let aabb = Rect::new(0.0, 0.0, 32.0, 32.0);

        hash.insert(entity, aabb);

        assert_eq!(hash.entity_count(), 1);
        assert!(hash.contains(entity));
        assert_eq!(hash.get_aabb(entity), Some(aabb));
    }

    #[test]
    fn test_insert_multiple() {
        let mut hash = SpatialHash::new(64.0);

        for i in 0..10 {
            let entity = entity(i);
            let aabb = Rect::new(i as f32 * 100.0, 0.0, 32.0, 32.0);
            hash.insert(entity, aabb);
        }

        assert_eq!(hash.entity_count(), 10);
    }

    #[test]
    fn test_insert_overwrites() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity, Rect::new(100.0, 100.0, 32.0, 32.0));

        assert_eq!(hash.entity_count(), 1);
        assert_eq!(
            hash.get_aabb(entity),
            Some(Rect::new(100.0, 100.0, 32.0, 32.0))
        );
    }

    #[test]
    fn test_remove_present() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.remove(entity));
        assert!(!hash.contains(entity));
        assert_eq!(hash.entity_count(), 0);
    }

    #[test]
    fn test_remove_absent() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        assert!(!hash.remove(entity));
    }

    #[test]
    fn test_remove_twice() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.remove(entity));
        assert!(!hash.remove(entity));
    }

    #[test]
    fn test_update_same_cells() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.update(entity, Rect::new(10.0, 10.0, 32.0, 32.0)));

        assert_eq!(
            hash.get_aabb(entity),
            Some(Rect::new(10.0, 10.0, 32.0, 32.0))
        );
    }

    #[test]
    fn test_update_different_cells() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));
        assert!(hash.update(entity, Rect::new(100.0, 100.0, 32.0, 32.0)));

        assert_eq!(
            hash.get_aabb(entity),
            Some(Rect::new(100.0, 100.0, 32.0, 32.0))
        );
    }

    #[test]
    fn test_update_absent() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        assert!(!hash.update(entity, Rect::new(0.0, 0.0, 32.0, 32.0)));
    }

    #[test]
    fn test_clear() {
        let mut hash = SpatialHash::new(64.0);

        for i in 0..10 {
            hash.insert(entity(i), Rect::new(i as f32 * 10.0, 0.0, 32.0, 32.0));
        }

        hash.clear();
        assert_eq!(hash.entity_count(), 0);
        assert_eq!(hash.cell_count(), 0);
        assert!(hash.is_empty());
    }

    // =========================================================================
    // Query Tests
    // =========================================================================

    #[test]
    fn test_query_pairs_empty() {
        let mut hash = SpatialHash::new(64.0);
        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_query_pairs_single() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 0); // No pairs with single entity
    }

    #[test]
    fn test_query_pairs_nearby() {
        let mut hash = SpatialHash::new(64.0);

        // Two entities in the same cell
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(30.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 1);
        assert!(pairs.contains(&(entity(0), entity(1))));
    }

    #[test]
    fn test_query_pairs_far_apart() {
        let mut hash = SpatialHash::new(64.0);

        // Two entities in different cells
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(200.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 0); // Too far apart
    }

    #[test]
    fn test_query_pairs_multiple() {
        let mut hash = SpatialHash::new(64.0);

        // Three entities in same cell
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(20.0, 0.0, 32.0, 32.0));
        hash.insert(entity(2), Rect::new(40.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 3); // (0,1), (0,2), (1,2)
    }

    #[test]
    fn test_query_pairs_no_duplicates() {
        let mut hash = SpatialHash::new(64.0);

        // Entity spanning multiple cells
        hash.insert(entity(0), Rect::new(0.0, 0.0, 128.0, 32.0)); // Spans 2 cells
        hash.insert(entity(1), Rect::new(20.0, 0.0, 32.0, 32.0));

        let pairs = hash.query_pairs();
        assert_eq!(pairs.len(), 1); // Only one pair despite multiple cells
    }

    #[test]
    fn test_query_point_hit() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let nearby = hash.query_point(Vec2::new(10.0, 10.0));
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&entity(0)));
    }

    #[test]
    fn test_query_point_miss() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let nearby = hash.query_point(Vec2::new(200.0, 200.0));
        assert_eq!(nearby.len(), 0);
    }

    #[test]
    fn test_query_aabb_overlapping() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));
        hash.insert(entity(1), Rect::new(100.0, 100.0, 32.0, 32.0));

        let nearby = hash.query_aabb(Rect::new(-10.0, -10.0, 50.0, 50.0));
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&entity(0)));
    }

    #[test]
    fn test_query_circle() {
        let mut hash = SpatialHash::new(64.0);
        hash.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let nearby = hash.query_circle(Vec2::new(16.0, 16.0), 20.0);
        assert_eq!(nearby.len(), 1);
        assert!(nearby.contains(&entity(0)));
    }

    // =========================================================================
    // Statistics Tests
    // =========================================================================

    #[test]
    fn test_stats_empty() {
        let hash = SpatialHash::new(64.0);
        let stats = hash.stats();

        assert_eq!(stats.entity_count, 0);
        assert_eq!(stats.cell_count, 0);
        assert_eq!(stats.total_cell_entries, 0);
        assert_eq!(stats.max_entities_per_cell, 0);
        assert_eq!(stats.avg_entities_per_cell, 0.0);
    }

    #[test]
    fn test_stats_after_insert() {
        let mut hash = SpatialHash::new(64.0);

        // Insert 3 entities in same cell
        for i in 0..3 {
            hash.insert(entity(i), Rect::new(i as f32 * 10.0, 0.0, 32.0, 32.0));
        }

        let stats = hash.stats();
        assert_eq!(stats.entity_count, 3);
        assert!(stats.cell_count > 0);
        assert!(stats.total_cell_entries >= 3);
    }

    #[test]
    fn test_stats_display() {
        let hash = SpatialHash::new(64.0);
        let stats = hash.stats();
        let display = format!("{}", stats);
        assert!(display.contains("SpatialHash Stats"));
        assert!(display.contains("Entities: 0"));
    }

    // =========================================================================
    // Large Entity Tests
    // =========================================================================

    #[test]
    fn test_large_entity_spans_multiple_cells() {
        let mut hash = SpatialHash::new(64.0);

        // Entity spanning 4 cells (2x2)
        let entity = entity(0);
        hash.insert(entity, Rect::new(0.0, 0.0, 128.0, 128.0));

        assert_eq!(hash.entity_count(), 1);
        assert!(hash.cell_count() >= 4); // Should occupy at least 4 cells
    }

    #[test]
    fn test_tiny_entity_single_cell() {
        let mut hash = SpatialHash::new(64.0);

        // Very small entity
        let entity = entity(0);
        hash.insert(entity, Rect::new(10.0, 10.0, 1.0, 1.0));

        assert_eq!(hash.entity_count(), 1);
        assert_eq!(hash.cell_count(), 1); // Should occupy exactly 1 cell
    }

    // =========================================================================
    // Stress Tests
    // =========================================================================

    #[test]
    fn test_stress_many_entities() {
        let mut hash = SpatialHash::new(64.0);

        // Insert 1000 entities
        for i in 0..1000 {
            let x = (i % 32) as f32 * 50.0;
            let y = (i / 32) as f32 * 50.0;
            hash.insert(entity(i), Rect::new(x, y, 32.0, 32.0));
        }

        assert_eq!(hash.entity_count(), 1000);

        // Query should complete
        let pairs = hash.query_pairs();
        assert!(pairs.len() > 0);
    }

    #[test]
    fn test_stress_update_cycle() {
        let mut hash = SpatialHash::new(64.0);
        let entity = entity(0);

        hash.insert(entity, Rect::new(0.0, 0.0, 32.0, 32.0));

        // Update 100 times
        for i in 0..100 {
            let x = (i % 10) as f32 * 20.0;
            let y = (i / 10) as f32 * 20.0;
            hash.update(entity, Rect::new(x, y, 32.0, 32.0));
        }

        assert_eq!(hash.entity_count(), 1);
    }

    // =========================================================================
    // Display Tests
    // =========================================================================

    #[test]
    fn test_display() {
        let hash = SpatialHash::new(64.0);
        let display = format!("{}", hash);
        assert!(display.contains("SpatialHash"));
        assert!(display.contains("64.0"));
    }

    // =========================================================================
    // Clone and Debug Tests
    // =========================================================================

    #[test]
    fn test_clone() {
        let mut hash1 = SpatialHash::new(64.0);
        hash1.insert(entity(0), Rect::new(0.0, 0.0, 32.0, 32.0));

        let hash2 = hash1.clone();
        assert_eq!(hash2.entity_count(), hash1.entity_count());
        assert_eq!(hash2.cell_size(), hash1.cell_size());
    }

    #[test]
    fn test_debug() {
        let hash = SpatialHash::new(64.0);
        let debug_str = format!("{:?}", hash);
        assert!(debug_str.contains("SpatialHash"));
    }

    // =========================================================================
    // SpatialHashStats unused import silencer
    // =========================================================================
    // The import is used only to verify it's re-exported correctly.
    #[test]
    fn test_stats_type_accessible() {
        let _: SpatialHashStats = SpatialHashStats::default();
    }
}
