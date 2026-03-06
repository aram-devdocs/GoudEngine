//! Tests for AABB utility functions.

#[cfg(test)]
mod aabb_tests {
    use crate::core::math::{Rect, Vec2};
    use crate::ecs::components::collider::aabb;
    use crate::ecs::components::{ColliderShape, Transform2D};

    #[test]
    fn test_aabb_compute_world_aabb_circle_no_rotation() {
        let shape = ColliderShape::Circle { radius: 2.0 };
        let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(10.0, 20.0));
        assert_eq!(world_aabb.width, 4.0);
        assert_eq!(world_aabb.height, 4.0);
    }

    #[test]
    fn test_aabb_compute_world_aabb_circle_with_scale() {
        let shape = ColliderShape::Circle { radius: 1.0 };
        let mut transform = Transform2D::from_position(Vec2::new(5.0, 5.0));
        transform.set_scale_uniform(2.0);

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(5.0, 5.0));
        assert_eq!(world_aabb.width, 4.0); // 2 * radius * scale
        assert_eq!(world_aabb.height, 4.0);
    }

    #[test]
    fn test_aabb_compute_world_aabb_box_no_rotation() {
        let shape = ColliderShape::Aabb {
            half_extents: Vec2::new(3.0, 2.0),
        };
        let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(10.0, 20.0));
        assert_eq!(world_aabb.width, 6.0);
        assert_eq!(world_aabb.height, 4.0);
    }

    #[test]
    fn test_aabb_compute_world_aabb_box_with_rotation() {
        let shape = ColliderShape::Obb {
            half_extents: Vec2::new(2.0, 1.0),
        };
        let mut transform = Transform2D::from_position(Vec2::new(0.0, 0.0));
        transform.set_rotation(std::f32::consts::PI / 4.0); // 45 degrees

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);

        // At 45 degrees, a 4x2 box should have AABB approximately sqrt(20) ≈ 4.47 on each side
        // But corners at (±2, ±1) rotated 45° gives max extent ≈ 2.12
        assert!(world_aabb.width > 4.0 && world_aabb.width < 4.5);
        assert!(world_aabb.height > 4.0 && world_aabb.height < 4.5);
        assert_eq!(world_aabb.center(), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_aabb_compute_world_aabb_capsule() {
        let shape = ColliderShape::Capsule {
            half_height: 1.0,
            radius: 0.5,
        };
        let transform = Transform2D::from_position(Vec2::new(5.0, 10.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);
        assert_eq!(world_aabb.center(), Vec2::new(5.0, 10.0));
        assert_eq!(world_aabb.width, 1.0); // 2 * radius
        assert_eq!(world_aabb.height, 3.0); // 2 * (half_height + radius)
    }

    #[test]
    fn test_aabb_compute_world_aabb_polygon() {
        let vertices = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(0.0, 3.0),
        ];
        let shape = ColliderShape::Polygon { vertices };
        let transform = Transform2D::from_position(Vec2::new(10.0, 20.0));

        let world_aabb = aabb::compute_world_aabb(&shape, &transform);

        // Local AABB is (-1, -1) to (2, 3), size (3, 4)
        // Translated to (10, 20) should give center at (10.5, 22)
        let expected_min = Vec2::new(9.0, 19.0);
        let expected_max = Vec2::new(12.0, 23.0);

        assert!((world_aabb.x - expected_min.x).abs() < 0.001);
        assert!((world_aabb.y - expected_min.y).abs() < 0.001);
        assert!((world_aabb.max().x - expected_max.x).abs() < 0.001);
        assert!((world_aabb.max().y - expected_max.y).abs() < 0.001);
    }

    #[test]
    fn test_aabb_overlaps() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);
        let c = Rect::new(20.0, 20.0, 10.0, 10.0);

        assert!(aabb::overlaps(&a, &b));
        assert!(aabb::overlaps(&b, &a));
        assert!(!aabb::overlaps(&a, &c));
    }

    #[test]
    fn test_aabb_intersection() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 10.0, 10.0);

        let intersection = aabb::intersection(&a, &b);
        assert!(intersection.is_some());
        let rect = intersection.unwrap();
        assert_eq!(rect.x, 5.0);
        assert_eq!(rect.y, 5.0);
        assert_eq!(rect.width, 5.0);
        assert_eq!(rect.height, 5.0);
    }

    #[test]
    fn test_aabb_intersection_none() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(20.0, 20.0, 10.0, 10.0);

        assert!(aabb::intersection(&a, &b).is_none());
    }

    #[test]
    fn test_aabb_expand() {
        let aabb_rect = Rect::new(5.0, 5.0, 10.0, 10.0);
        let expanded = aabb::expand(&aabb_rect, 2.0);

        assert_eq!(expanded.x, 3.0);
        assert_eq!(expanded.y, 3.0);
        assert_eq!(expanded.width, 14.0);
        assert_eq!(expanded.height, 14.0);
    }

    #[test]
    fn test_aabb_expand_negative_margin() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let shrunk = aabb::expand(&aabb_rect, -1.0);

        assert_eq!(shrunk.x, 1.0);
        assert_eq!(shrunk.y, 1.0);
        assert_eq!(shrunk.width, 8.0);
        assert_eq!(shrunk.height, 8.0);
    }

    #[test]
    fn test_aabb_expand_zero_margin() {
        let aabb_rect = Rect::new(5.0, 5.0, 10.0, 10.0);
        let expanded = aabb::expand(&aabb_rect, 0.0);

        assert_eq!(expanded, aabb_rect);
    }

    #[test]
    fn test_aabb_merge() {
        let a = Rect::new(0.0, 0.0, 5.0, 5.0);
        let b = Rect::new(3.0, 3.0, 5.0, 5.0);
        let merged = aabb::merge(&a, &b);

        assert_eq!(merged.x, 0.0);
        assert_eq!(merged.y, 0.0);
        assert_eq!(merged.width, 8.0);
        assert_eq!(merged.height, 8.0);
    }

    #[test]
    fn test_aabb_merge_disjoint() {
        let a = Rect::new(0.0, 0.0, 5.0, 5.0);
        let b = Rect::new(10.0, 10.0, 5.0, 5.0);
        let merged = aabb::merge(&a, &b);

        assert_eq!(merged.x, 0.0);
        assert_eq!(merged.y, 0.0);
        assert_eq!(merged.width, 15.0);
        assert_eq!(merged.height, 15.0);
    }

    #[test]
    fn test_aabb_contains_point() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);

        assert!(aabb::contains_point(&aabb_rect, Vec2::new(5.0, 5.0)));
        assert!(aabb::contains_point(&aabb_rect, Vec2::new(0.0, 0.0)));
        assert!(!aabb::contains_point(&aabb_rect, Vec2::new(-1.0, 5.0)));
        assert!(!aabb::contains_point(&aabb_rect, Vec2::new(5.0, 11.0)));
    }

    #[test]
    fn test_aabb_raycast_hit() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(-5.0, 5.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        let hit = aabb::raycast(&aabb_rect, ray_origin, ray_direction, 100.0);
        assert!(hit.is_some());
        let t = hit.unwrap();
        assert!((t - 5.0).abs() < 0.001); // Should hit at t=5
    }

    #[test]
    fn test_aabb_raycast_miss() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(-5.0, 15.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        let hit = aabb::raycast(&aabb_rect, ray_origin, ray_direction, 100.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_aabb_raycast_from_inside() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(5.0, 5.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        let hit = aabb::raycast(&aabb_rect, ray_origin, ray_direction, 100.0);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap(), 0.0); // Ray starts inside
    }

    #[test]
    fn test_aabb_raycast_max_distance() {
        let aabb_rect = Rect::new(100.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(0.0, 5.0);
        let ray_direction = Vec2::new(1.0, 0.0);

        // Should not hit because max_distance is too short
        let hit = aabb::raycast(&aabb_rect, ray_origin, ray_direction, 50.0);
        assert!(hit.is_none());

        // Should hit with longer max_distance
        let hit = aabb::raycast(&aabb_rect, ray_origin, ray_direction, 200.0);
        assert!(hit.is_some());
    }

    #[test]
    fn test_aabb_raycast_diagonal() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let ray_origin = Vec2::new(-5.0, -5.0);
        let ray_direction = Vec2::new(1.0, 1.0).normalize();

        let hit = aabb::raycast(&aabb_rect, ray_origin, ray_direction, 100.0);
        assert!(hit.is_some());

        // Hit point should be near (0, 0)
        let t = hit.unwrap();
        let hit_point = ray_origin + ray_direction * t;
        assert!((hit_point.x - 0.0).abs() < 0.1);
        assert!((hit_point.y - 0.0).abs() < 0.1);
    }

    #[test]
    fn test_aabb_closest_point_outside() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(-5.0, 5.0);

        let closest = aabb::closest_point(&aabb_rect, point);
        assert_eq!(closest, Vec2::new(0.0, 5.0));
    }

    #[test]
    fn test_aabb_closest_point_inside() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(5.0, 5.0);

        let closest = aabb::closest_point(&aabb_rect, point);
        assert_eq!(closest, point); // Point is inside, returns itself
    }

    #[test]
    fn test_aabb_closest_point_corner() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(-5.0, -5.0);

        let closest = aabb::closest_point(&aabb_rect, point);
        assert_eq!(closest, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn test_aabb_distance_squared_to_point_outside() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(-3.0, 0.0);

        let dist_sq = aabb::distance_squared_to_point(&aabb_rect, point);
        assert_eq!(dist_sq, 9.0); // Distance is 3.0, squared is 9.0
    }

    #[test]
    fn test_aabb_distance_squared_to_point_inside() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let point = Vec2::new(5.0, 5.0);

        let dist_sq = aabb::distance_squared_to_point(&aabb_rect, point);
        assert_eq!(dist_sq, 0.0);
    }

    #[test]
    fn test_aabb_area() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(aabb::area(&aabb_rect), 50.0);
    }

    #[test]
    fn test_aabb_perimeter() {
        let aabb_rect = Rect::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(aabb::perimeter(&aabb_rect), 30.0); // 2 * (10 + 5)
    }
}
